use std::{collections::BTreeMap, env, fs::File, io::{self, BufWriter, Write}, path::Path};
use serde::Deserialize;

const ENV_VAR_OUT_DIR: &str = "OUT_DIR";
const EXPORT_FILE_NAME: &str = "vendors.rs";
const VENDORS_LIST: &str = include_str!("../vendors.toml");

/// Generates code related to vendors
fn main() -> io::Result<()> {
    let out_dir = env::var(ENV_VAR_OUT_DIR).unwrap();
    let out_path = Path::new(&out_dir).join(EXPORT_FILE_NAME);   
    let mut file = BufWriter::new(File::create(&out_path)?);

    let entries = toml::from_str::<BTreeMap<String, Entry>>(VENDORS_LIST).unwrap();

    // constants
    writeln!(file, "struct Entry {{ pub id: u16, pub name: &'static str }}\n")?;
    for (abbr, Entry { id, name }) in &entries {
        let abbr = abbr.to_uppercase();
        writeln!(file, "const {abbr}: Entry = Entry {{ id: {id}, name: \"{name}\" }};",)?;
    }
    writeln!(file)?;

    // get_by_id
    writeln!(file, "pub const fn get_by_id(id: u16) -> Option<&'static str> {{")?;
    writeln!(file, "    match id {{")?;
    for Entry { id, name } in entries.values() {
        writeln!(file, "        {id} => Some(\"{name}\"),")?;
    }
    writeln!(file, "        _ => None,")?;
    writeln!(file, "    }}")?;
    writeln!(file, "}}")?;
    writeln!(file)?;

    // get_by_name
    writeln!(file, "pub fn get_by_name(name: &str) -> Option<u16> {{")?;
    writeln!(file, "    match name {{")?;
    for Entry { id, name } in entries.values() {
        writeln!(file, "        \"{name}\" => Some({id}),")?;
    }
    writeln!(file, "        _ => None,")?;
    writeln!(file, "    }}")?;
    writeln!(file, "}}")?;

    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry { id: u16, name: String }