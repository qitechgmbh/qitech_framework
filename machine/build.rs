use std::env;
use std::fs;
use std::path::Path;

#[derive(serde::Deserialize)]
struct UomFile {
    quantity: Vec<UomEntry>,
}

#[derive(serde::Deserialize)]
struct UomEntry {
    name: String,
    units: Vec<String>,
}

fn main() {
    let toml_path = "../units.toml";
    println!("cargo:rerun-if-changed={toml_path}");

    let raw = fs::read_to_string(toml_path)
        .unwrap_or_else(|e| panic!("failed to read {toml_path}: {e}"));

    let parsed: UomFile =
        toml::from_str(&raw).unwrap_or_else(|e| panic!("failed to parse {toml_path}: {e}"));

    let mut body = String::new();
    body.push_str("macro_rules! with_uom { ($callback:ident) => { ");

    for UomEntry { name, units } in &parsed.quantity {
        let module = pascal_to_snake(name);

        for unit in units {
            let unit = pascal_to_snake(unit);

            let module_path = format!("qitech_lib::units::{module}");
            let quantity = format!("qitech_lib::units::{name}");
            let unit = format!("{module_path}::{unit}");
            let unit_trait = format!("{module_path}::Unit");
            let conversion_trait = format!("{module_path}::Conversion<f64>");

            body.push_str(&format!(
                "$callback!({quantity}, {unit}, {unit_trait}, {conversion_trait});\n"
            ));
        }
    }

    body.push_str("};\n}\n");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let dest = Path::new(&out_dir).join("with_uom.rs");
    fs::write(&dest, body).unwrap_or_else(|e| panic!("failed to write {dest:?}: {e}"));
}

fn pascal_to_snake(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 8);
    for (i, ch) in input.chars().enumerate() {
        if ch.is_uppercase() {
            if i != 0 {
                out.push('_');
            }
            out.extend(ch.to_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}