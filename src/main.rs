//#![allow(dead_code)]
//#![allow(unused_variables)]


use rayon::prelude::*;
use exif;
use std::collections::HashMap;
use serde_json::json;
use serde::{Deserialize, Serialize};
use docopt::Docopt;
use glob::glob;
use std::path::PathBuf;



const USAGE: &'static str = "
Usage: just-the-exif <path-glob>

Description:
    Extract EXIF data from a collection of images in parallel.

Options:
    <path_glob>         Specify the path to the image file.
    -h, --help     Show this message.
";


#[derive(Debug, Deserialize)]
struct Args {
    arg_path_glob: String,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                          .and_then(|d| d.deserialize())
                          .unwrap_or_else(|e| e.exit());

    let paths = read_paths(&args.arg_path_glob);
    read_exif_parallel(paths);
}

fn read_paths(path_glob: &str) -> Vec<PathBuf>{
    let mut paths: Vec<PathBuf> = Vec::new();

    for entry in glob(path_glob).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => paths.push(path),
            Err(e) => println!("{:?}", e),
        }
    }

    paths
}

fn read_exif_parallel(paths: Vec<PathBuf>) {
    paths.par_iter().for_each(|path| {
        let data = get_exif(path.to_str().unwrap());

        print_exif(data);
    });
}

fn print_exif(data: Result<HashMap<String, SerialisedField>, Box<dyn std::error::Error>>) {
    if data.is_ok() {
        let json = json!(data.unwrap());
        println!("{}", json);
        return;
    }

}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum SerialisedField {
    Int(i64),
    Float(f64),
    String(String),
}

fn serialise_value(field: &exif::Field) -> SerialisedField {
    let serialised = field.value.display_as(field.tag).to_string();

    match serialised.parse::<i64>() {
        Ok(i) => SerialisedField::Int(i),
        Err(_) => match serialised.parse::<f64>() {
            Ok(f) => SerialisedField::Float(f),
            Err(_) => SerialisedField::String(serialised),
        }
    }
}

fn get_exif(path: &str) -> Result<HashMap<String, SerialisedField>, Box<dyn std::error::Error>>{
    let mut map = HashMap::new();

    let file = std::fs::File::open(path)?;
    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = exif::Reader::new();
    let exif = exifreader.read_from_container(&mut bufreader)?;

    for field in exif.fields() {
        map.insert(field.tag.to_string(), serialise_value(field));
    }

    Ok(map)
}