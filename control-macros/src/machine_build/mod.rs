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

    let mut function: syn::ItemFn = syn::parse2(item)?;

    let first_arg = function
        .sig
        .inputs
        .first()
        .ok_or_else(|| {
            Error::new_spanned(
                &function.sig,
                "build(...) requires BuildContext argument",
            )
        })?;

    let FnArg::Typed(arg) = first_arg else {
        return Err(Error::new_spanned(
            first_arg,
            "build(...) must not have self receiver",
        ));
    };

    let Pat::Ident(name) = arg.pat.as_ref() else {
        return Err(Error::new_spanned(
            &arg.pat,
            "context argument must be named",
        ));
    };

    let mut error = None;

    let mut visitor = BuildVisitor {
        ctx: name.ident.clone(),
        schema: &schema,
        error: &mut error,
    };

    visitor.visit_block_mut(&mut function.block);

    if let Some(e) = error {
        return Err(e);
    }

    Ok(quote::quote! {
        #function
    })
}

struct BuildVisitor<'a> {
    ctx: syn::Ident,
    schema: &'a control_core::schema::latest::Schema,
    error: &'a mut Option<syn::Error>,
}

impl<'a> VisitMut for BuildVisitor<'a> {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        if let Expr::Macro(macro_expr) = expr {
            if macro_expr.mac.path.is_ident("config_property") {
                match self.process_config_property(macro_expr) {
                    Ok(v) => *expr = v,
                    Err(e) => *self.error = Some(e),
                }

                return;
            }

            if macro_expr.mac.path.is_ident("state_property") {
                match self.process_state_property(macro_expr) {
                    Ok(v) => *expr = v,
                    Err(e) => *self.error = Some(e),
                }

                return;
            }
        }
        

        visit_mut::visit_expr_mut(self, expr);
    }
}

impl BuildVisitor<'_> {
    fn process_config_property(
        &mut self, 
        macro_expr: &mut ExprMacro,
    ) -> Result<Expr> {
        let args: ConfigPropertyArgs =
            syn::parse2(macro_expr.mac.tokens.clone())?;

        let resource_name = args.name.value();

        let info = match self.schema.find_config_property(&resource_name) {
            Some(v) => v,
            None => return Err(Error::new_spanned(
                args.name,
                format!(
                    "unknown config property '{}'",
                    resource_name
                ),
            ))
        };

        let options = match args.validate {
            Some(default_value) => {
                quote::quote! {
                    ConfigPropertyOptions {
                        default_value: #default_value,
                    }
                }
            }

            None => {
                quote::quote! {
                    ConfigPropertyOptions {
                        ..Default::default()
                    }
                }
            }
        };

        let ctx = &self.ctx;
        let ty = schema_utils::state_value_to_type(info)?;
        let name = args.name;

        // --- replace macro call with actual content ---
        Ok(syn::parse_quote! {
            #ctx.register_state_property::<#ty>(
                #name,
                #options
            )?
        })
    }

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
        let ty = schema_utils::state_value_to_type(info)?;
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

struct ConfigPropertyArgs {
    name: LitStr,
    validate: Option<Expr>,
    on_changed: Option<Expr>,
}

impl Parse for ConfigPropertyArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse::<LitStr>()?;

        let mut validate = None;
        let mut on_changed = None;

        while input.peek(Token![,]) {
            input.parse::<Token![,]>()?;

            let key = input.parse::<Ident>()?;

            input.parse::<Token![=]>()?;
            let value = input.parse::<Expr>()?;

            match key.to_string().as_str() {
                "validate" => validate = Some(value),
                "on_changed" => on_changed = Some(value),
                _ => {
                    return Err(Error::new_spanned(
                        key,
                        "unknown config_property option",
                    ));
                }
            }
        }

        Ok(Self {
            name,
            validate,
            on_changed,
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