// use proc_macro::TokenStream;
//
// mod schema_utils;
// use schema_utils::Schema;
//
// mod machine_build;
//
// #[proc_macro_attribute]
// pub fn machine_build(
//     attr: TokenStream,
//     item: TokenStream,
// ) -> TokenStream {
//     machine_build::expand(attr.into(), item.into())
//         .unwrap_or_else(syn::Error::into_compile_error)
//         .into()
// }

/*

        #[derive(Clone, Default, Deserialize)]
        #[serde(rename_all = "snake_case")]
        enum MyEnum {
            #[default]
            MyVariant,
            MyVariant2,
        }

        impl TypeWrapper for MyEnum {
            type Type = MyEnum;
            type Input = MyEnum;

            fn into_scalar(value: &Self::Type) -> ScalarValue {
                ScalarValue::Enum(Some(match value {
                    MyEnum::MyVariant => "my_variant",
                    MyEnum::MyVariant2 => "my_variant",
                }.to_string()))
            }

            fn convert_input(input: Self::Input) -> Self::Type {
                input
            }

            fn deserialize_json(raw: &str) -> serde_json::Result<Self> {
                match raw {
                    "my_variant" => Ok(Self::MyVariant),
                    "my_variant2" => Ok(Self::MyVariant2),
                    _ => Err(serde_json::Error::custom(format!(
                        "unknown variant: {raw}"
                    ))),
                }
            }
        }

        impl BoundedMeta for MyEnum {
            type Bound = u64;
            fn as_bound(&self) -> Option<Self::Bound> { None }
        }

        let my_enum = ctx.config::<MyEnum>("enum").register()?;
*/

use heck::ToSnakeCase;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

#[proc_macro_derive(EnumProperty)]
pub fn enum_property(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let Data::Enum(data) = input.data else {
        panic!("EnumProperty can only be derived for enums");
    };

    let variants: Vec<_> = data
        .variants
        .iter()
        .map(|v| {
            let ident = &v.ident;
            let snake = ident.to_string().to_snake_case();

            quote! {
                #name::#ident => #snake
            }
        })
        .collect();

    let from_variants: Vec<_> = data
        .variants
        .iter()
        .map(|v| {
            let ident = &v.ident;
            let snake = ident.to_string().to_snake_case();

            quote! {
                #snake => Some(Self::#ident)
            }
        })
        .collect();

    let expanded = quote! {
        impl qitech_framework::machine::TypeWrapper for #name {
            type Type = #name;
            type Input = #name;
            type Constraints = qitech_framework::machine::resource::EnumConfigPropertyConstraints<String>;

            fn convert_input(input: Self::Input) -> Self::Type {
                input
            }

            fn into_scalar(value: Self::Type) -> qitech_framework::ScalarValue {
                qitech_framework::ScalarValue::Enum(Some(
                    match value {
                        #(#variants,)*
                    }
                    .to_string()
                ))
            }

            fn from_scalar(value: qitech_framework::ScalarValue) -> Option<Self::Type> {
                match value {
                    qitech_framework::ScalarValue::Enum(Some(v)) => match v.as_str() {
                        #(#from_variants,)*
                        _ => None,
                    },
                    _ => None,
                }
            }

            fn into_constraints(
                constraints: Self::Constraints
            ) -> qitech_framework::machine::ConfigPropertyWriteConstraints {
                qitech_framework::machine::ConfigPropertyWriteConstraints::Enum {
                    allowed: constraints.allowed,
                }
            }
        }
    };

    TokenStream::from(expanded)
}