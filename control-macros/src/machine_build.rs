use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    ExprMacro, FnArg, ImplItem, ItemImpl, Lit, Meta, Pat, Token, parse::Parser,
    punctuated::Punctuated, token::Comma,
};

use syn::visit_mut::{self, VisitMut};

pub fn expand(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    // ----------------------------
    // Parse #[machine_build(...)]
    // ----------------------------

    let args = Punctuated::<Meta, Comma>::parse_terminated.parse2(attr)?;

    let machine = args
        .iter()
        .find_map(|arg| {
            let Meta::NameValue(v) = arg else {
                return None;
            };

            if !v.path.is_ident("machine") {
                return None;
            }

            let syn::Expr::Lit(syn::ExprLit {
                lit: Lit::Str(s), ..
            }) = &v.value
            else {
                return None;
            };

            Some(s.value())
        })
        .ok_or_else(|| {
            syn::Error::new(proc_macro2::Span::call_site(), "missing machine attribute")
        })?;

    // ----------------------------
    // Load schema
    // ----------------------------

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").map_err(|_| {
        syn::Error::new(proc_macro2::Span::call_site(), "missing CARGO_MANIFEST_DIR")
    })?;

    let schema_path = format!("{manifest_dir}/schemas/{machine}.yaml");

    let yaml = std::fs::read_to_string(&schema_path).map_err(|e| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("failed reading {}: {}", schema_path, e),
        )
    })?;

    let schema = control_core::schema::parse_latest(&yaml).map_err(|e| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("invalid schema: {}", e),
        )
    })?;

    eprintln!(
        "loaded machine {} schema version {}",
        machine, schema.qms_version
    );

    // ----------------------------
    // Parse impl block
    // ----------------------------

    let mut item_impl: ItemImpl = syn::parse2(item)?;

    // ----------------------------
    // Find build()
    // ----------------------------

    let mut ctx_ident = None;

    for item in &mut item_impl.items {
        let ImplItem::Fn(method) = item else {
            continue;
        };

        if method.sig.ident != "build" {
            continue;
        }

        let Some(first_arg) = method.sig.inputs.first() else {
            return Err(syn::Error::new_spanned(
                &method.sig,
                "build requires BuildContext argument",
            ));
        };

        let FnArg::Typed(arg) = first_arg else {
            return Err(syn::Error::new_spanned(
                first_arg,
                "build must not have self receiver",
            ));
        };

        let Pat::Ident(name) = arg.pat.as_ref() else {
            return Err(syn::Error::new_spanned(
                &arg.pat,
                "context argument must be named",
            ));
        };

        ctx_ident = Some(name.ident.clone());

        // transform body
        let mut visitor = BuildVisitor {
            ctx: name.ident.clone(),
        };

        visitor.visit_block_mut(&mut method.block);
    }

    let Some(_) = ctx_ident else {
        return Err(syn::Error::new_spanned(
            &item_impl,
            "missing build() function",
        ));
    };

    // ----------------------------
    // Emit modified impl
    // ----------------------------

    Ok(quote! {
        #item_impl
    })
}

struct BuildVisitor {
    ctx: syn::Ident,
}

impl VisitMut for BuildVisitor {
    fn visit_expr_macro_mut(&mut self, node: &mut ExprMacro) {
        if node.mac.path.is_ident("state_property") {
            let Ok(name) = syn::parse2::<syn::LitStr>(node.mac.tokens.clone()) else {
                return;
            };

            let resource_name = name.value();

            eprintln!("found state property {}", resource_name);

            // temporary replacement
            //
            // state_property!("foo")
            //
            // becomes
            //
            // ctx.state_property("foo")

            let ctx = &self.ctx;

            *node = syn::parse_quote! {
                println!("Hello World!")
            };

            return;
        }

        visit_mut::visit_expr_macro_mut(self, node);
    }
}
