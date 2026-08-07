use std::fs;
use std::path::Path;

use heck::ToSnakeCase;
use proc_macro::TokenStream;
use qitech_framework_core::schema::MachineSchema;
use quote::quote;
use syn::Data;
use syn::DeriveInput;
use syn::parse_macro_input;

#[proc_macro_derive(EnumProperty)]
pub fn enum_property(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let Data::Enum(data) = input.data else {
        panic!("EnumProperty can only be derived for enums");
    };

    let to_scalar = data.variants.iter().map(|variant| {
        let ident = &variant.ident;
        let value = ident.to_string().to_string();

        quote! {
            Self::#ident => #value
        }
    });

    let from_scalar = data.variants.iter().map(|variant| {
        let ident = &variant.ident;
        let value = ident.to_string().to_snake_case();

        quote! {
            #value => Ok(Self::#ident)
        }
    });

    let expanded = quote! {
        impl qitech_framework::__private::PropertyAdapter for #name {
            type Type = #name;
            type Input = #name;

            fn convert_input(input: Self::Input) -> Self::Type {
                input
            }

            fn into_scalar(value: Self::Type) -> qitech_framework::__private::ScalarValue {
                qitech_framework::__private::ScalarValue::Enum(Some(
                    match value {
                        #(#to_scalar,)*
                    }
                    .to_string()
                ))
            }

            fn from_scalar(
                value: qitech_framework::__private::ScalarValue
            ) -> Result<Self::Type, qitech_framework::__private::ScalarValueTypeMismatchError> {
                match value {
                    qitech_framework::__private::ScalarValue::Enum(Some(v)) => {
                        match v.as_str() {
                            #(#from_scalar,)*
                            _ => Err(qitech_framework::__private::ScalarValueTypeMismatchError),
                        }
                    }

                    _ => Err(qitech_framework::__private::ScalarValueTypeMismatchError),
                }
            }

            fn validate_constraints(
                constraints: &<Self::Type as qitech_framework::__private::PropertyType>::Constraints,
                value: &Self::Type,
            ) -> Result<(), qitech_framework::__private::ConstraintViolationError> {
                if constraints.allowed.contains(value) {
                    Ok(())
                } else {
                    Err(
                        qitech_framework::__private::ConstraintViolationError::ForbiddenVariant {
                            value: Self::into_scalar(value.clone()),
                        }
                    )
                }
            }

            fn as_parameter_constraints(
                constraints: &<Self::Type as qitech_framework::__private::PropertyType>::Constraints,
            ) -> qitech_framework::__private::Constraints {
                qitech_framework::__private::Constraints::Enum {
                    allowed: constraints
                        .allowed
                        .iter()
                        .cloned()
                        .map(Self::into_scalar)
                        .collect(),
                    nullable: false,
                }
            }
        }

        impl qitech_framework::__private::PropertyType for #name {
            type Constraints = qitech_framework::__private::EnumConstraints<#name>;
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Machine, attributes(machine))]
pub fn machine(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let schema_name = name.to_string().to_snake_case();
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR missing");
    let schema_dir = find_schema_dir();

    let schema_path = Path::new(&manifest_dir)
        .join(schema_dir)
        .join(format!("{schema_name}.yaml"));

    if !schema_path.exists() {
        panic!(
            "Could not find schema for `{name}` at {}",
            schema_path.display()
        );
    }

    let schema_string = fs::read_to_string(&schema_path).expect("failed reading schema");
    let schema = MachineSchema::parse_str(&schema_string).expect("invalid machine schema");

    let vendor_id = schema.identification.vendor_id;
    let machine_id = schema.identification.machine_id;
    let schema_path = schema_path.to_string_lossy();

    let expanded = quote! {
        impl qitech_framework::__private::MachineDescriptor for #name {
            const SCHEMA: &'static str =
                include_str!(#schema_path);

            const IDENTIFICATION:
                qitech_framework::machine::MachineIdentification =
                qitech_framework::machine::MachineIdentification {
                    vendor_id: #vendor_id,
                    machine_id: #machine_id,
                };
        }
    };

    expanded.into()
}

fn find_schema_dir() -> String {
    let manifest = std::fs::read_to_string("Cargo.toml").expect("failed reading Cargo.toml");

    let value: toml::Value = toml::from_str(&manifest).expect("invalid Cargo.toml");

    value
        .get("package")
        .and_then(|p| p.get("metadata"))
        .and_then(|m| m.get("qitech"))
        .and_then(|q| q.get("schema-dir"))
        .and_then(|s| s.as_str())
        .unwrap_or("schemas")
        .to_string()
}
