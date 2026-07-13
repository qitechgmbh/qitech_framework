use std::{collections::BTreeMap, env, fs::File, io::{self, BufWriter, Write}, path::Path};
use serde::Deserialize;

const ENV_VAR_OUT_DIR: &str = "OUT_DIR";
const EXPORT_FILE_NAME: &str = "vendors.rs";
const VENDORS_LIST: &str = include_str!("../vendors.toml");

type Entries = BTreeMap<String, Entry>;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    id: u16,
    name: String,
}

fn main() -> io::Result<()> {
    let out_dir = env::var(ENV_VAR_OUT_DIR).unwrap();
    let out_path = Path::new(&out_dir).join(EXPORT_FILE_NAME);   
    let mut file = BufWriter::new(File::create(&out_path)?);

    // define VENDORS
    let mut vendors = phf_codegen::OrderedMap::<u16>::new();

    let entries = toml::from_str::<Entries>(VENDORS_LIST).unwrap();

    for (abbr, entry) in entries {
        writeln!(file, "pub const {}: u16 = {};", abbr.to_uppercase(), entry.id)?;
        vendors.entry(entry.id, format!("\"{}\"", entry.name));
    }

    let vendors = vendors.build();
    writeln!(file, "static REGISTRY: phf::OrderedMap<u16, &'static str> = {vendors};")?;
    Ok(())
}
