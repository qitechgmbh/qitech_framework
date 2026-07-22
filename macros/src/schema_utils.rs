use std::{env, fs};

use control_core::schema;
use proc_macro2::Span;
use syn::{
    Error, Expr, Ident, Lit, Meta, MetaNameValue, ExprLit, Result, Token, parse::{Parse, ParseStream}, punctuated::Punctuated, token::Comma,
};

pub struct Schema(pub control_core::schema::latest::Schema);

macro_rules! fail_callsite {
    ($($arg:tt)*) => {
        Error::new(
            Span::call_site(),
            format!($($arg)*),
        )
    };
}

impl Parse for Schema {
    fn parse(input: ParseStream) -> Result<Self> {
        let machine = input.parse::<Ident>()?;
        let mut schema_dir = None;

        // if we find any ',' parse them
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            let metas = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;
            process_metas(metas, &mut schema_dir)?;
        }

        // --- assemble the final path ---
        let schema_dir = schema_dir.unwrap_or(format!("{}/schemas", manifest_dir()?));
        let schema_path = format!("{schema_dir}/{machine}.yaml");

        // --- read the file ---
        let yaml = fs::read_to_string(&schema_path).map_err(|e| {
            fail_callsite!("failed reading {}: {}", schema_path, e)
        })?;

        // --- parse the schema ---
        let schema = control_core::schema::parse_latest(&yaml)
            .map_err(|e| fail_callsite!("invalid schema: {}", e))?;

        // --- ok ---
        Ok(Self(schema))
    }
}

fn process_metas(metas: Punctuated<Meta, Comma>, schema_dir: &mut Option<String>) -> Result<()> {
    for meta in metas {
        match meta {
            Meta::NameValue(nv) if nv.path.is_ident("schema_dir") => {
                *schema_dir = process_schema_dir(nv)?;
            }

            other => {
                return Err(Error::new_spanned(other, "unknown machine_build argument"));
            }
        }
    }

    Ok(())
}

fn process_schema_dir(nv: MetaNameValue) -> Result<Option<String>> {
    let Expr::Lit(ExprLit {
        lit: Lit::Str(s),
        ..
    }) = nv.value
    else {
        return Err(Error::new_spanned(
            nv,
            "schema_dir must be a string literal",
        ));
    };

    Ok(Some(s.value()))
}

fn manifest_dir() -> Result<String> {
    env::var("CARGO_MANIFEST_DIR")
        .map_err(|_| fail_callsite!("missing CARGO_MANIFEST_DIR"))
}

// --- utils ---
pub fn config_value_to_type(
    value: &schema::latest::config::Value,
) -> syn::Result<syn::Type> {
    use schema::latest::config::*;

    match value {
        Value::Boolean(BooleanValue { nullable, default, persistent }) => {
            if *nullable {
                Ok(syn::parse_quote! { Option<bool> })
            } else {
                Ok(syn::parse_quote! { bool })
            }
        }

        Value::Float(FloatValue { nullable, default, range, persistent }) => {
            if *nullable {
                Ok(syn::parse_quote! { Option<f64> })
            } else {
                Ok(syn::parse_quote! { f64 })
            }
        }

        Value::Integer(IntegerValue { nullable, default, range, persistent }) => {
            if *nullable {
                Ok(syn::parse_quote! { Option<i64> })
            } else {
                Ok(syn::parse_quote! { i64 })
            }
        }

        Value::Quantity { value, unit } => {
            let ty: syn::Type = syn::parse_quote! {
                qitech_lib::units::length::millimeter
            };

            if value.nullable {
                Ok(syn::parse_quote! { Option<i64> })
            } else {
                Ok(syn::parse_quote! { i64 })
            }
        }

        _ => Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "unsupported state property type",
        )),
    }
}

pub fn state_value_to_type(
    value: &schema::latest::state::Value,
) -> syn::Result<syn::Type> {
    use schema::latest::state::{ScalarValue, Value};

    match value {
        Value::Boolean(ScalarValue { nullable }) => {
            if *nullable {
                Ok(syn::parse_quote! { Option<bool> })
            } else {
                Ok(syn::parse_quote! { bool })
            }
        }

        Value::Float(ScalarValue { nullable }) => {
            if *nullable {
                Ok(syn::parse_quote! { Option<f64> })
            } else {
                Ok(syn::parse_quote! { f64 })
            }
        }

        Value::Integer(ScalarValue { nullable }) => {
            if *nullable {
                Ok(syn::parse_quote! { Option<i64> })
            } else {
                Ok(syn::parse_quote! { i64 })
            }
        }

        Value::Quantity { value, unit } => {
            let ty: syn::Type = syn::parse_quote! {
                qitech_lib::units::length::millimeter
            };

            if value.nullable {
                Ok(syn::parse_quote! { Option<i64> })
            } else {
                Ok(syn::parse_quote! { i64 })
            }
        }

        _ => Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "unsupported state property type",
        )),
    }
}