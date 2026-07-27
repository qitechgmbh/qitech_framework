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
