use std::{env, fs};

use proc_macro2::Span;
use qitech_framework_common::{
    MachineSchema, schema::{
        ConfigPropertyValue, ConfigPropertyValueKind, FloatSemantic, StatePropertyValue, StatePropertyValueKind,
    },
};
use syn::{
    Error, Expr, ExprLit, Ident, Lit, Meta, MetaNameValue, Result, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Comma,
};

pub struct Schema(pub MachineSchema);

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
        let yaml = fs::read_to_string(&schema_path)
            .map_err(|e| fail_callsite!("failed reading {}: {}", schema_path, e))?;

        // --- parse the schema ---
        let schema = MachineSchema::from_yaml_str(&yaml)
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
        lit: Lit::Str(s), ..
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
    env::var("CARGO_MANIFEST_DIR").map_err(|_| fail_callsite!("missing CARGO_MANIFEST_DIR"))
}

// --- utils ---
pub fn config_property_value_to_type(value: &ConfigPropertyValue) -> syn::Result<syn::Type> {
    use ConfigPropertyValueKind::*;

    let ty: syn::Type = match value.kind {
        Boolean { .. } => syn::parse_quote!(bool),
        Float { semantic, .. } => match semantic {
            FloatSemantic::Quantity(quantity) => syn::parse_str(quantity.as_str())?,
            _ => syn::parse_quote!(f64),
        },
        Integer { .. } => syn::parse_quote!(i64),
        _ => {
            return Err(Error::new(
                Span::call_site(),
                "unsupported config property type",
            ));
        }
    };

    Ok(if value.nullable {
        syn::parse_quote!(Option<#ty>)
    } else {
        ty
    })
}

pub fn state_property_value_to_type(value: &StatePropertyValue) -> syn::Result<syn::Type> {
    use StatePropertyValueKind::*;

    let ty: syn::Type = match value.kind {
        Boolean { .. } => syn::parse_quote!(bool),
        Float { semantic, .. } => match semantic {
            FloatSemantic::Quantity(quantity) => syn::parse_str(quantity.as_str())?,
            _ => syn::parse_quote!(f64),
        },
        Integer { .. } => syn::parse_quote!(i64),
        _ => {
            return Err(Error::new(
                Span::call_site(),
                "unsupported config property type",
            ));
        }
    };

    Ok(if value.nullable {
        syn::parse_quote!(Option<#ty>)
    } else {
        ty
    })
}