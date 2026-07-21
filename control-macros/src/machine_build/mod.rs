use std::env;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Error, Expr, Ident, LitStr, Result, Token};
use syn::{
    ExprMacro, FnArg, ImplItem, ItemImpl, Lit, Meta, Pat, parse::Parser,
    punctuated::Punctuated, token::Comma,
};

use crate::schema_utils;
use syn::visit_mut::{self, VisitMut};

mod state_property;

pub fn expand(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    // --- parse schema ---
    let schema = syn::parse2::<crate::Schema>(attr)?.0;

    // --- parse impl block ---
    let mut item_impl: ItemImpl = syn::parse2(item)?;

    // --- find build(...) ---
    let mut ctx_ident = None;

    for item in &mut item_impl.items {
        let ImplItem::Fn(method) = item else {
            continue;
        };

        if method.sig.ident != "build" {
            continue;
        }

        let Some(first_arg) = method.sig.inputs.first() else {
            return Err(Error::new_spanned(
                &method.sig,
                "build(...) requires BuildContext argument",
            ));
        };

        let FnArg::Typed(arg) = first_arg else {
            return Err(Error::new_spanned(
                first_arg,
                "build(...) must not have self receiver",
            ));
        };

        eprintln!("schema ok");

        let Pat::Ident(name) = arg.pat.as_ref() else {
            return Err(Error::new_spanned(
                &arg.pat,
                "context argument must be named",
            ));
        };

        ctx_ident = Some(name.ident.clone());

        // --- transform body ---
        let mut error = None;
        let mut visitor = BuildVisitor {
            ctx: name.ident.clone(),
            schema: &schema,
            error: &mut error,
        };

        visitor.visit_block_mut(&mut method.block);

        if let Some(e) = error {
            // failed to parse an item inside build(...)
            return Err(e);
        }
    }

    let Some(_) = ctx_ident else {
        return Err(Error::new_spanned(
            &item_impl,
            "missing build(...) function",
        ));
    };

    // --- emit modified impl ---
    Ok(quote! {
        #item_impl
    })
}

struct BuildVisitor<'a> {
    ctx: syn::Ident,
    schema: &'a control_core::schema::latest::Schema,
    error: &'a mut Option<syn::Error>,
}

impl<'a> VisitMut for BuildVisitor<'a> {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        if let Expr::Macro(macro_expr) = expr && macro_expr.mac.path.is_ident("state_property") {
            match self.process_state_property(macro_expr) {
                Ok(v) => *expr = v,
                Err(e) => *self.error = Some(e),
            }

            return;
        }

        visit_mut::visit_expr_mut(self, expr);
    }
}

impl BuildVisitor<'_> {
    fn process_state_property(
        &mut self, 
        macro_expr: &mut ExprMacro,
    ) -> Result<Expr> {
        let args: StatePropertyArgs =
            syn::parse2(macro_expr.mac.tokens.clone())?;

        let resource_name = args.name.value();

        let info = match self.schema.find_state_property(&resource_name) {
            Some(v) => v,
            None => return Err(Error::new_spanned(
                args.name,
                format!(
                    "unknown state property '{}'",
                    resource_name
                ),
            ))
        };

        let options = match args.initial_value {
            Some(initial_value) => {
                quote::quote! {
                    StatePropertyOptions {
                        initial_value: #initial_value,
                    }
                }
            }

            None => {
                quote::quote! {
                    StatePropertyOptions {
                        ..Default::default()
                    }
                }
            }
        };

        let ctx = &self.ctx;
        let ty = schema_utils::value_to_type(info)?;
        let name = args.name;

        // --- replace macro call with actual content ---
        Ok(syn::parse_quote! {
            #ctx.register_state_property::<#ty>(
                #name,
                #options
            )?
        })
    }
}

struct StatePropertyArgs {
    name: LitStr,
    initial_value: Option<Expr>,
}

impl Parse for StatePropertyArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse::<LitStr>()?;

        let mut initial_value = None;

        while input.peek(Token![,]) {
            input.parse::<Token![,]>()?;

            let key = input.parse::<Ident>()?;

            input.parse::<Token![=]>()?;
            let value = input.parse::<Expr>()?;

            match key.to_string().as_str() {
                "initial_value" => initial_value = Some(value),
                _ => {
                    return Err(Error::new_spanned(
                        key,
                        "unknown state_property option",
                    ));
                }
            }
        }

        Ok(Self {
            name,
            initial_value,
        })
    }
}