use anyhow::{format_err, Result};
use std::fs::{read_dir, File};
use std::io::Write;
use std::ops::Index;
use std::path::PathBuf;

fn generate_structs<W: Write>(mut wtr: W) -> Result<W> {
    let mut elements = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    elements.push("elements");

    let d = read_dir(elements)?;
    for entry_result in d {
        let entry = entry_result?;
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(b';')
            .from_path(entry.path())?;
        let stem = entry
            .path()
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .trim_end_matches("_par")
            .to_string();
        write!(wtr, "pub struct {} {{\n", heck::AsUpperCamelCase(stem))?;
        for rec_result in rdr.records() {
            let rec = rec_result?;
            let param = rec.index(0).trim_end_matches('*');
            let datatype = rec.index(1);
            let explanation = match rec.len() {
                3 => rec.index(2),
                4 => rec.index(3),
                _ => {
                    return Err(format_err!("record length must be 3 or 4: {}", rec.len()));
                }
            }
            .trim_start()
            .replace("\n", "    /// ")
            .replace("[", "\\[")
            .replace("]", "\\]");
            if !explanation.is_empty() {
                write!(wtr, "    /// {}\n", explanation)?;
            }
            write!(
                wtr,
                "    pub {}: {},\n",
                match param {
                    "type" => "r#type",
                    _ => param,
                },
                match datatype {
                    "string" | "String" => "String",
                    "float" => "f64",
                    "boolean" | "bool" => "bool",
                    "integer" | "int" => "usize",
                    "list" => {
                        match param {
                            "coords" => "Vec<(f64, f64)>",
                            _ => {
                                return Err(format_err!("unsupported list datatype: {}", param));
                            }
                        }
                    }
                    _ => {
                        return Err(format_err!("unsupported datatype: {}", datatype));
                    }
                }
            )?;
        }
        write!(wtr, "}}\n")?;
    }
    Ok(wtr)
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let mut path = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    path.push("generated.rs");
    let generated = File::create(path).unwrap();
    std::process::exit(match generate_structs(generated) {
        Err(err) => {
            eprintln!("error: {:?}", err);
            1
        }
        Ok(_) => 0,
    })
}
