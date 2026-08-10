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

            fn validate_scalar_property_definition(definition: &qitech_framework::__private::ScalarPropertyDefinition) -> bool {
                true
            }

            fn validate_measurement_definition(definition: &qitech_framework::__private::MeasurementDefinition) -> bool {
                true
            }

            fn apply_constraints(
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

            fn as_constraints(
                constraints: &<Self::Type as qitech_framework::__private::PropertyType>::Constraints,
            ) -> qitech_framework::__private::Constraints {
                qitech_framework::__private::Constraints::Enum {
                    allowed: constraints
                        .allowed
                        .iter()
                        .cloned()
                        .map(Self::into_scalar)
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

#[proc_macro_derive(Machine, attributes(machine))]
pub fn machine(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let schema_path = find_machine_schema_path(&name);
    let schema = load_machine_schema(&name);

    let vendor_id = schema.identification.vendor_id;
    let machine_id = schema.identification.machine_id;

    let schema_path = schema_path.to_string_lossy();

    quote! {
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
    }
    .into()
}

// --- build helper ---
#[proc_macro_attribute]
pub fn machine_build(args: TokenStream, input: TokenStream) -> TokenStream {
    let machine = parse_macro_input!(args as Path);
    let mut function = parse_macro_input!(input as ItemFn);

    let ident = match machine.segments.last() {
        Some(segment) => &segment.ident,
        None => {
            return syn::Error::new_spanned(machine, "expected machine type")
                .to_compile_error()
                .into();
        }
    };

    let schema = load_machine_schema(ident);

    let mut visitor = ConfigRewriter {
        schema,
        error: None,
    };

    visitor.visit_block_mut(&mut function.block);

    if let Some(error) = visitor.error {
        return error.to_compile_error().into();
    }

    quote! {
        #function
    }
    .into()
}

struct ConfigRewriter {
    schema: MachineSchema,
    error: Option<syn::Error>,
}

impl VisitMut for ConfigRewriter {
    fn visit_expr_method_call_mut(&mut self, node: &mut ExprMethodCall) {
        if self.error.is_none() && node.method == "config" {
            let Some(arg) = node.args.first() else {
                self.error = Some(syn::Error::new_spanned(
                    node,
                    "config() requires a string literal resource path",
                ));
                return;
            };

            let Expr::Lit(expr_lit) = arg else {
                self.error = Some(syn::Error::new_spanned(
                    arg,
                    "config() requires a string literal resource path",
                ));
                return;
            };

            let Lit::Str(value) = &expr_lit.lit else {
                self.error = Some(syn::Error::new_spanned(
                    arg,
                    "config() requires a string literal resource path",
                ));
                return;
            };

            let key = value.value();

            let Some(property) = self.schema.config_properties.get(&key) else {
                self.error = Some(syn::Error::new_spanned(
                    arg,
                    format!("unknown config property `{key}`"),
                ));
                return;
            };

            match &node.turbofish {
                Some(turbofish) => {
                    let Some(GenericArgument::Type(user_type)) = turbofish.args.first() else {
                        self.error = Some(syn::Error::new_spanned(
                            turbofish,
                            "config() expects a type argument",
                        ));
                        return;
                    };

                    if let Err(err) = validate_rust_type(property, user_type) {
                        self.error = Some(err);
                        return;
                    }
                }

                None => {
                    // These cannot be inferred because the Rust representation
                    // is user-defined.
                    if matches!(
                        property.kind,
                        ScalarPropertyKind::Enum { .. } | ScalarPropertyKind::String
                    ) {
                        self.error = Some(syn::Error::new_spanned(
                            node,
                            format!(
                                "cannot infer type for config property `{key}`; \
                                 specify it explicitly"
                            ),
                        ));
                        return;
                    }
                }
            }
        }

        visit_mut::visit_expr_method_call_mut(self, node);
    }
}

// --- utils ---
fn load_machine_schema(name: &Ident) -> MachineSchema {
    let schema_path = find_machine_schema_path(name);

    let schema_string = fs::read_to_string(&schema_path).unwrap_or_else(|e| {
        panic!(
            "Failed reading schema for `{name}` at {}: {e}",
            schema_path.display()
        )
    });

    MachineSchema::parse_str(&schema_string).unwrap_or_else(|e| {
        panic!(
            "Invalid schema for `{name}` at {}: {e}",
            schema_path.display()
        )
    })
}

fn find_machine_schema_path(name: &Ident) -> path::PathBuf {
    let schema_name = name.to_string().to_snake_case();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR missing");

    let schema_dir = find_schema_dir();

    let path = path::Path::new(&manifest_dir)
        .join(schema_dir)
        .join(format!("{schema_name}.yaml"));

    if !path.exists() {
        panic!("Could not find schema for `{name}` at {}", path.display());
    }

    path
}

fn find_schema_dir() -> String {
    let manifest = fs::read_to_string("Cargo.toml").expect("failed reading Cargo.toml");

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

// --- type gen ---
fn validate_rust_type(
    property: &ScalarPropertyDefinition,
    user_type: &syn::Type,
) -> Result<(), syn::Error> {
    let inner_type = if property.nullable {
        match unwrap_option(user_type) {
            Some(inner) => inner,
            None => {
                return Err(syn::Error::new_spanned(
                    user_type,
                    "nullable property requires Option<T>",
                ));
            }
        }
    } else {
        if is_option(user_type) {
            return Err(syn::Error::new_spanned(
                user_type,
                "non-nullable property cannot use Option<T>",
            ));
        }

        user_type
    };

    if let ScalarPropertyKind::Float { semantic } = &property.kind
        && let FloatSemantic::Quantity(quantity) = semantic
    {
        let expected = quantity.as_str();

        if !type_ends_with(inner_type, expected) {
            return Err(syn::Error::new_spanned(
                inner_type,
                format!("expected quantity type `{expected}`"),
            ));
        }
    }

    Ok(())
}

fn type_ends_with(ty: &syn::Type, expected: &str) -> bool {
    let syn::Type::Path(type_path) = ty else {
        return false;
    };

    let Some(segment) = type_path.path.segments.last() else {
        return false;
    };

    segment.ident == expected
}

fn unwrap_option(ty: &syn::Type) -> Option<&syn::Type> {
    let syn::Type::Path(type_path) = ty else {
        return None;
    };

    let segment = type_path.path.segments.last()?;

    if segment.ident == "Option" {
        let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
            return None;
        };

        let syn::GenericArgument::Type(inner) = args.args.first()? else {
            return None;
        };

        Some(inner)
    } else {
        None
    }
}

fn is_option(ty: &syn::Type) -> bool {
    unwrap_option(ty).is_some()
}
