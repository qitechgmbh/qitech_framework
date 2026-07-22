use std::{collections::BTreeMap, env, fs::File, io::{self, BufWriter, Write}, path::Path};
use serde::Deserialize;

const ENV_VAR_OUT_DIR: &str = "OUT_DIR";

const VENDORS_PATH: &str = "vendors.toml";
const VENDORS_DATA: &str = include_str!("vendors.toml");
const VENDORS_EXPORT_FILE_NAME: &str = "vendors.rs";

const QUANTITIES_PATH: &str = "quantities.toml";
const QUANTITIES_DATA: &str = include_str!("quantities.toml");
const WITH_UOM_EXPORT_FILE_NAME: &str = "with_uom.rs";
const PARSE_TYPE_EXPORT_FILE_NAME: &str = "parse_type.rs";

/// Generates code related to vendors
fn main() -> io::Result<()> {
    let out_dir = env::var(ENV_VAR_OUT_DIR).unwrap();
    create_vendors(&out_dir)?;
    create_with_uom(&out_dir)?;
    create_parse_value_type(&out_dir)?;
    Ok(())
}

fn create_vendors(out_dir: &String) -> io::Result<()> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Entry { id: u16, name: String }

    println!("cargo:rerun-if-changed={VENDORS_PATH}");

    let out_path = Path::new(&out_dir).join(VENDORS_EXPORT_FILE_NAME);   
    let mut file = BufWriter::new(File::create(&out_path)?);

    let entries = toml::from_str::<BTreeMap<String, Entry>>(VENDORS_DATA).unwrap();

    // constants
    writeln!(file, "pub struct Entry {{ pub id: u16, pub name: &'static str }}\n")?;
    for (abbr, Entry { id, name }) in &entries {
        let abbr = abbr.to_uppercase();
        writeln!(file, "pub const {abbr}: Entry = Entry {{ id: {id}, name: \"{name}\" }};",)?;
    }
    writeln!(file)?;

    writeln!(file, "mod private {{")?;

    // get_by_id
    writeln!(file, "pub const fn get_name(id: u16) -> Option<&'static str> {{")?;
    writeln!(file, "    match id {{")?;
    for Entry { id, name } in entries.values() {
        writeln!(file, "        {id} => Some(\"{name}\"),")?;
    }
    writeln!(file, "        _ => None,")?;
    writeln!(file, "    }}")?;
    writeln!(file, "}}")?;
    writeln!(file)?;

    // get_by_name
    writeln!(file, "pub fn get_id(name: &str) -> Option<u16> {{")?;
    writeln!(file, "    match name {{")?;
    for Entry { id, name } in entries.values() {
        writeln!(file, "        \"{name}\" => Some({id}),")?;
    }
    writeln!(file, "        _ => None,")?;
    writeln!(file, "    }}")?;
    writeln!(file, "}}")?;

    writeln!(file, "}}")?;
    Ok(())
}

#[derive(serde::Deserialize)]
struct UomFile {
    quantity: Vec<UomEntry>,
}

#[derive(serde::Deserialize)]
struct UomEntry {
    name: String,
    units: Vec<String>,
}


fn create_with_uom(out_dir: &String) -> io::Result<()>  {
    println!("cargo:rerun-if-changed={QUANTITIES_PATH}");

    let out_path = Path::new(&out_dir).join(WITH_UOM_EXPORT_FILE_NAME);   
    let mut file = BufWriter::new(File::create(&out_path)?);

    let UomFile { quantity } = toml::from_str(QUANTITIES_DATA)
        .unwrap_or_else(|e| panic!("failed to parse {QUANTITIES_PATH}: {e}"));

    file.write_all(b"#[macro_export]")?;
    file.write_all(b"macro_rules! with_uom { ($callback:ident) => { ")?;

    for UomEntry { name, units } in &quantity {
        let module = pascal_to_snake(name);

        for unit in units {
            let unit_snake = pascal_to_snake(unit);

            let module_path = format!("qitech_lib::units::{module}");
            let quantity = format!("qitech_lib::units::{name}");
            let unit = format!("{module_path}::{unit_snake}");
            let unit_trait = format!("{module_path}::Unit");
            let conversion_trait = format!("{module_path}::Conversion<f64>");

            writeln!(file, "$callback!({quantity}, {unit}, {unit_trait}, {conversion_trait});")?;
        }
    }

    file.write_all(b"};\n}\n")?;

    Ok(())
}

fn create_parse_value_type(out_dir: &String) -> io::Result<()> {
    const BASE: &str = r#"
    pub fn parse(tag: &str) -> Result<ValueType, String> {
        Ok(match tag {
            "command" => ValueType::Command,
            "event" => ValueType::Event,
            "object" => ValueType::Object,
            "array" => ValueType::Array,
            "enum" => ValueType::Enum,
            "string" => ValueType::String,
            "integer" => ValueType::Integer,
            "float" => ValueType::Float(FloatSemantic::Plain),
            "fraction" => ValueType::Float(FloatSemantic::Fraction),
            "percentage" => ValueType::Float(FloatSemantic::Percentage),
    "#;

    let out_path = Path::new(&out_dir).join(PARSE_TYPE_EXPORT_FILE_NAME);   
    let mut file = BufWriter::new(File::create(&out_path)?);

    let UomFile { quantity } = toml::from_str(QUANTITIES_DATA)
        .unwrap_or_else(|e| panic!("failed to parse {QUANTITIES_PATH}: {e}"));

    write!(file, "{BASE}")?;

    for UomEntry { name, units } in &quantity {
        for unit in units {
            let unit_snake = pascal_to_snake(unit);

            writeln!(file, "\"{unit_snake}\" => ValueType::Float(FloatSemantic::Quantity(")?;
            writeln!(file, "    Quantity::{name}({name}Unit::{unit})")?;
            writeln!(file, ")),")?;
        }
    }
    writeln!(file, "other => return Err(format!(\"Unknown value type {{other}}\"))")?;
    writeln!(file, "}})")?;
    writeln!(file, "}}")?;

    Ok(())
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
