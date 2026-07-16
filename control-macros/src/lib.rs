use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    Ident, Token,
};

struct RowInput {
    table: Ident,
    fields: Vec<Ident>,
}

impl Parse for RowInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let table: Ident = input.parse()?;

        let content;
        syn::braced!(content in input);

        let fields = content
            .parse_terminated(Ident::parse, Token![,])?
            .into_iter()
            .collect();

        Ok(Self { table, fields })
    }
}

#[proc_macro]
pub fn row(input: TokenStream) -> TokenStream {
    let RowInput { table, fields } = parse_macro_input!(input as RowInput);

    let table_name = table.to_string();

    let struct_name = syn::Ident::new(
        &format!("{}Row", to_upper_camel(&table_name)),
        table.span(),
    );

    let generated_fields = fields.iter().map(|field| {
        let ty = field_type(&table_name, &field.to_string());

        quote! {
            pub #field: #ty,
        }
    });

    quote! {
        // #[derive(Debug, serde::Serialize, serde::Deserialize, clickhouse::Row)]
        #[derive(Debug)]
        pub struct #struct_name {
            #(#generated_fields)*
        }
    }
    .into()
}

fn field_type(table: &str, field: &str) -> proc_macro2::TokenStream {
    match (table, field) {
        ("logs", "timestamp") => quote!(chrono::DateTime<chrono::Utc>),
        ("logs", "level") => quote!(i8),
        ("logs", "origin") => quote!(u64),
        ("logs", "message") => quote!(String),
        ("logs", "attributes") => quote!(Vec<(String, String)>),

        _ => panic!("unknown field {table}.{field}"),
    }
}

fn to_upper_camel(value: &str) -> String {
    value
        .split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => {
                    first.to_uppercase().collect::<String>() + chars.as_str()
                }
                None => String::new(),
            }
        })
        .collect()
}