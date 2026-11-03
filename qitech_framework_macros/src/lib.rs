use std::fs;
use std::path;

use heck::ToSnakeCase;
use proc_macro::TokenStream;
use qitech_framework_core::schema::FloatSemantic;
use qitech_framework_core::schema::MachineSchema;
use qitech_framework_core::schema::ScalarPropertyDefinition;
use qitech_framework_core::schema::ScalarPropertyKind;
use quote::quote;
use syn::Data;
use syn::DeriveInput;
use syn::Expr;
use syn::ExprMethodCall;
use syn::GenericArgument;
use syn::Ident;
use syn::ItemFn;
use syn::Lit;
use syn::Path;
use syn::parse_macro_input;
use syn::visit_mut;
use syn::visit_mut::VisitMut;

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
                qitech_framework::__private::ScalarValue::Enum(
                    match value {
                        #(#to_scalar,)*
                    }
                    .to_string()
                )
            }

            fn from_scalar(
                value: qitech_framework::__private::ScalarValue
            ) -> Result<Self::Type, qitech_framework::__private::ScalarValueTypeMismatchError> {
                match value {
                    qitech_framework::__private::ScalarValue::Enum(v) => {
                        match v.as_str() {
                            #(#from_scalar,)*
                            _ => Err(qitech_framework::__private::ScalarValueTypeMismatchError),
                        }
                    }

                    _ => Err(qitech_framework::__private::ScalarValueTypeMismatchError),
                }
            }

            fn validate_scalar_property_definition(
                definition: &qitech_framework::__private::ScalarPropertyDefinition,
                ignore_nullable: bool,
            ) -> bool {
                if !ignore_nullable && definition.nullable {
                    return false;
                }

                true
            }

            fn validate_measurement_definition(
                definition: &qitech_framework::__private::MeasurementDefinition,
                ignore_nullable: bool,
            ) -> bool {
                if !ignore_nullable && definition.nullable {
                    return false;
                }

                true
            }

            fn apply_constraints(
                constraints: &<Self::Type as qitech_framework::__private::PropertyType>::Constraints,
                value: &Self::Type,
            ) -> Result<(), qitech_framework::__private::ConstraintViolationError> {
                // TODO: initialize with constraints so this doesnt lock out from the start
                Ok(())
                // if constraints.allowed.contains(value) {
                //     Ok(())
                // } else {
                //     Err(
                //         qitech_framework::__private::ConstraintViolationError::ForbiddenVariant {
                //             value: Self::into_scalar(value.clone()),
                //         }
                //     )
                // }
            }

            fn as_constraints(
                constraints: &<Self::Type as qitech_framework::__private::PropertyType>::Constraints,
            ) -> qitech_framework::__private::Constraints {
                qitech_framework::__private::Constraints::Enum {
                    allowed: constraints
                        .allowed
                        .iter()
                        .cloned()
                        .map(|v| Self::into_scalar(v).r#enum().expect("Cannot be null"))
                        .collect(),
                }
            }
        }

        impl qitech_framework::__private::PropertyType for #name {
            type Constraints = qitech_framework::__private::EnumConstraints<#name>;
        }
    };

    TokenStream::from(expanded)
}
