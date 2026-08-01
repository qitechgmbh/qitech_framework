use std::collections::BTreeMap;
use std::env;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::io::{self};
use std::path::Path;

use serde::Deserialize;

const ENV_VAR_OUT_DIR: &str = "OUT_DIR";

const VENDORS_PATH: &str = "vendors.toml";
const VENDORS_DATA: &str = include_str!("vendors.toml");
const VENDORS_EXPORT_FILE_NAME: &str = "vendors.rs";

const QUANTITIES_PATH: &str = "quantities.toml";
const QUANTITIES_DATA: &str = include_str!("quantities.toml");
const QUANTITY_EXPORT_FILE_NAME: &str = "quantity.rs";
const WITH_UOM_EXPORT_FILE_NAME: &str = "with_uom.rs";

fn main() -> io::Result<()> {
    println!("cargo:rerun-if-changed={VENDORS_PATH}");
    println!("cargo:rerun-if-changed={QUANTITIES_PATH}");

    let out_dir = env::var(ENV_VAR_OUT_DIR).unwrap();
    create_vendors(&out_dir)?;
    create_quantity(&out_dir)?;
    create_with_uom(&out_dir)?;
    Ok(())
}

/// generates the vendors constants and lookups from the vendors.toml
fn create_vendors(out_dir: &String) -> io::Result<()> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Entry {
        id: u16,
        name: String,
    }

    let out_path = Path::new(&out_dir).join(VENDORS_EXPORT_FILE_NAME);
    let mut file = BufWriter::new(File::create(&out_path)?);

    let entries = toml::from_str::<BTreeMap<String, Entry>>(VENDORS_DATA).unwrap();

    // encapsulate inside private module
    writeln!(file, "mod generated {{")?;

    // --- emit constants ---
    writeln!(
        file,
        "pub struct Entry {{ pub id: u16, pub name: &'static str }}\n"
    )?;
    for (abbr, Entry { id, name }) in &entries {
        let abbr = abbr.to_uppercase();
        writeln!(
            file,
            "pub const {abbr}: Entry = Entry {{ id: {id}, name: \"{name}\" }};",
        )?;
    }
    writeln!(file)?;

    // --- emit get_by_id(...) ---
    writeln!(
        file,
        "pub const fn get_name(id: u16) -> Option<&'static str> {{"
    )?;
    writeln!(file, "    match id {{")?;
    for Entry { id, name } in entries.values() {
        writeln!(file, "        {id} => Some(\"{name}\"),")?;
    }
    writeln!(file, "        _ => None,")?;
    writeln!(file, "    }}")?;
    writeln!(file, "}}")?;
    writeln!(file)?;

    // --- emit get_by_name(...) ---
    writeln!(file, "pub fn get_id(name: &str) -> Option<u16> {{")?;
    writeln!(file, "    match name {{")?;
    for Entry { id, name } in entries.values() {
        writeln!(file, "        \"{name}\" => Some({id}),")?;
    }
    writeln!(file, "        _ => None,")?;
    writeln!(file, "    }}")?;
    writeln!(file, "}}")?;

    // --- finish module ---
    writeln!(file, "}}")?;
    Ok(())
}

/// generates quantity.rs from quantities.rs
fn create_quantity(out_dir: &String) -> io::Result<()> {
    let out_path = Path::new(&out_dir).join(QUANTITY_EXPORT_FILE_NAME);
    let mut file = BufWriter::new(File::create(&out_path)?);

    // encapsulate inside private module
    writeln!(file, "mod generated {{")?;
    writeln!(file, "use serde::Serialize;")?;
    writeln!(file, "use super::*;")?;

    let UomFile { quantity } = toml::from_str(QUANTITIES_DATA)
        .unwrap_or_else(|e| panic!("failed to parse {QUANTITIES_PATH}: {e}"));

    // --- pass one: Quantity definition ---
    writeln!(file, "#[derive(Debug, Clone, Copy, Serialize)]")?;
    writeln!(file, "pub enum Quantity {{")?;

    for UomEntry { name, .. } in &quantity {
        writeln!(file, "{name}({name}Unit),")?;
    }

    writeln!(file, "}}")?;

    // --- pass two: emit Quantity unit types ---
    for UomEntry { name, units } in &quantity {
        writeln!(file, "#[derive(Debug, Clone, Copy, Serialize)]")?;
        writeln!(file, "pub enum {name}Unit {{")?;

        for unit in units {
            writeln!(file, "{unit},")?;
        }

        writeln!(file, "}}")?;
    }

    // --- pass three: emit Display for Quantity ---
    writeln!(file, "impl Display for Quantity {{")?;
    writeln!(
        file,
        "    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {{"
    )?;
    writeln!(file, "        match self {{")?;

    for UomEntry { name, .. } in &quantity {
        writeln!(file, "        Self::{name}(u) => write!(f, \"{{u}}\"),")?;
    }

    writeln!(file, "        }}")?;
    writeln!(file, "    }}")?;
    writeln!(file, "}}")?;

    // --- pass four: emit Display for quantity unit types ---
    for UomEntry { name, units } in &quantity {
        writeln!(file, "impl Display for {name}Unit {{")?;
        writeln!(
            file,
            "    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {{"
        )?;
        writeln!(file, "        match self {{")?;

        for unit in units {
            let unit_snake = pascal_to_snake(unit);
            writeln!(file, "        Self::{unit} => write!(f, \"{unit_snake}\"),")?;
        }

        writeln!(file, "        }}")?;
        writeln!(file, "    }}")?;
        writeln!(file, "}}")?;
    }

    // --- pass five: emit Quantity::from_str(tag: &str) ---
    writeln!(file, "impl std::str::FromStr for Quantity {{")?;
    writeln!(file, "    type Err = String;")?;
    writeln!(
        file,
        "    fn from_str(s: &str) -> Result<Self, Self::Err> {{"
    )?;
    writeln!(file, "        Ok(match s {{")?;

    for UomEntry { name, units } in &quantity {
        for unit in units {
            let unit_snake = pascal_to_snake(unit);

            writeln!(
                file,
                "            \"{unit_snake}\" => Self::{name}({name}Unit::{unit}),"
            )?;
        }
    }

    writeln!(
        file,
        "            other => return Err(format!(\"undefined quantity unit {{other}}\")),"
    )?;
    writeln!(file, "        }})")?;
    writeln!(file, "    }}")?;
    writeln!(file, "}}")?;

    // --- pass six: emit Quantity::as_str(&self) -> &'static str ---
    writeln!(file, "impl Quantity {{")?;
    writeln!(file, "    pub fn as_str(&self) -> &'static str {{")?;
    writeln!(file, "        match self {{")?;

    for UomEntry { name, units } in &quantity {
        for unit in units {
            let unit_snake = pascal_to_snake(unit);

            writeln!(
                file,
                "            Self::{name}({name}Unit::{unit}) => \"{unit_snake}\","
            )?;
        }
    }

    writeln!(file, "        }}")?;
    writeln!(file, "    }}")?;
    writeln!(file, "}}")?;

    // --- finish mod generated ---
    writeln!(file, "}}")?;

    Ok(())
}

/// generates with_uom!() from quantities.rs.
/// a macro that accepts a macro to iterate all uom units
fn create_with_uom(out_dir: &String) -> io::Result<()> {
    let out_path = Path::new(&out_dir).join(WITH_UOM_EXPORT_FILE_NAME);
    let mut file = BufWriter::new(File::create(&out_path)?);

    let UomFile { quantity } = toml::from_str(QUANTITIES_DATA)
        .unwrap_or_else(|e| panic!("failed to parse {QUANTITIES_PATH}: {e}"));

    // --- pass one: quantities  ---
    file.write_all(b"#[macro_export]")?;
    file.write_all(b"macro_rules! with_uom_quantities { ($callback:ident) => { ")?;

    for UomEntry { name, .. } in &quantity {
        let module = pascal_to_snake(name);

        let module_path = format!("qitech_lib::units::{module}");
        let quantity = format!("qitech_lib::units::{name}");
        let unit_trait = format!("{module_path}::Unit");
        let conversion_trait = format!("{module_path}::Conversion<f64>");

        writeln!(
            file,
            "$callback!({quantity}, {unit_trait}, {conversion_trait});"
        )?;
    }

    file.write_all(b"};\n}\n")?;

    // --- pass two: units  ---
    file.write_all(b"#[macro_export]")?;
    file.write_all(b"macro_rules! with_uom_units { ($callback:ident) => { ")?;

    for UomEntry { name, units } in &quantity {
        let module = pascal_to_snake(name);

        for unit in units {
            let module_path = format!("qitech_lib::units::{module}");

            let quantity = format!("qitech_lib::units::{name}");
            let unit = format!("{module_path}::{}", pascal_to_snake(unit));
            let unit_trait = format!("{module_path}::Unit");
            let conversion_trait = format!("{module_path}::Conversion<f64>");

            writeln!(
                file,
                "$callback!({quantity}, {unit}, {unit_trait}, {conversion_trait});"
            )?;
        }
    }

    file.write_all(b"};\n}\n")?;

    Ok(())
}

// --- utils ---
#[derive(serde::Deserialize)]
struct UomFile {
    quantity: Vec<UomEntry>,
}

#[derive(serde::Deserialize)]
struct UomEntry {
    name: String,
    units: Vec<String>,
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
