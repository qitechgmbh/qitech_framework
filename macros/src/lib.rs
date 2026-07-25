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