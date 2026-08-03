use heck::ToSnakeCase;
use proc_macro::TokenStream;
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
