use clap::Parser;
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use yaml_rust::yaml::Hash;
use yaml_rust::yaml::Yaml;
use yaml_rust::{YamlEmitter, YamlLoader};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// File containing the base spec
    #[clap(short, long)]
    base: String,

    /// Comma seperated list of paths or filenames to exclude
    #[clap(short, long, default_value = "")]
    exclude: String,

    ///Documentation Directory.
    #[clap(short, long, default_value = "./")]
    directory: String,

    ///Output file. Will be created if it doesn't exist
    #[clap(short, long, default_value = "./output.yaml")]
    output: String,
}

fn get_files(folder: &str) -> Vec<PathBuf> {
    let paths = fs::read_dir(folder).expect("Directory does not exist");
    let mut file_paths: Vec<PathBuf> = Vec::new();
    for path in paths {
        let value = path.unwrap().path();
        if value.is_file() {
            file_paths.push(value);
        }
    }
    return file_paths;
}

fn get_matched_files(folder: &str, extensions: Vec<&str>, exclude: Vec<&str>) -> Vec<PathBuf> {
    let files = get_files(folder);
    let exclusion_set: HashSet<&str> = exclude.into_iter().collect();
    let mut matched_files: Vec<PathBuf> = Vec::new();
    let extension_set: HashSet<&str> = extensions.into_iter().collect();
    for file in files {
        let filename = file.as_path().as_os_str().to_str().unwrap();
        if exclusion_set.contains(filename) {
            continue;
        }
        let ext = file.extension();
        match ext {
            Some(ext_name) => {
                let plain_ext_name = ext_name.to_str().unwrap();
                if extension_set.contains(plain_ext_name) {
                    matched_files.push(file)
                }
            }
            None => continue,
        }
    }
    return matched_files;
}

fn get_spec(file: PathBuf) -> Hash {
    let content = fs::read_to_string(file).expect("File does not exist");
    let docs = YamlLoader::load_from_str(&content).unwrap();
    let mut spec = &Hash::new();
    if docs.len() < 1 {
        return spec.clone();
    }
    let doc = &docs[0];
    match doc {
        Yaml::Hash(inner_spec) => spec = inner_spec,
        _ => (),
    }
    return spec.clone();
}

fn get_base_spec(path: &str) -> Hash {
    let path_buf = PathBuf::from(path);
    return get_spec(path_buf);
}

fn get_underlying_hash(spec: &Hash, key: &str) -> Hash {
    let content = spec.get(&Yaml::String(String::from(key)));
    let mut default_hash = &Yaml::Hash(Hash::new());
    match content {
        Some(value) => default_hash = value,
        _ => (),
    }
    let mut underlying_hash = &Hash::new();
    match default_hash {
        Yaml::Hash(hash) => underlying_hash = hash,
        _ => (),
    }
    return underlying_hash.clone();
}

fn merge_paths(base_paths: &Hash, other_paths: &Hash) -> Hash {
    let mut output = Hash::new();
    output.extend(base_paths.into_iter().map(|(k, v)| (k.clone(), v.clone())));
    output.extend(other_paths.into_iter().map(|(k, v)| (k.clone(), v.clone())));
    return output.clone();
}

fn get_yaml_specs(file_path: &str, exclude: Vec<&str>) -> Vec<Hash> {
    let yaml_files = get_matched_files(file_path, vec!["yaml", "yml"], exclude);
    let mut specs: Vec<Hash> = vec![];
    for yaml_file in yaml_files {
        let spec = get_spec(yaml_file);
        specs.push(spec)
    }
    return specs;
}

fn merge_specs(base_spec: Hash, other_specs: Vec<Hash>) -> Hash {
    let base_spec_paths = get_underlying_hash(&base_spec, "paths");
    let base_spec_definitions = get_underlying_hash(&base_spec, "definitions");
    let mut path_output = Hash::new();
    path_output.extend(
        base_spec_paths
            .into_iter()
            .map(|(k, v)| (k.clone(), v.clone())),
    );
    let mut definitions_output = Hash::new();
    definitions_output.extend(
        base_spec_definitions
            .into_iter()
            .map(|(k, v)| (k.clone(), v.clone())),
    );
    for spec in other_specs {
        let path_underlying_hash = get_underlying_hash(&spec, "paths");
        path_output = merge_paths(&path_output, &path_underlying_hash);
        let definitions_underlying_hash = get_underlying_hash(&spec, "definitions");
        definitions_output = merge_paths(&definitions_output, &definitions_underlying_hash)
    }
    let mut output_spec = base_spec.clone();
    output_spec.insert(Yaml::String(String::from("paths")), Yaml::Hash(path_output));
    output_spec.insert(
        Yaml::String(String::from("definitions")),
        Yaml::Hash(definitions_output),
    );
    return output_spec;
}

fn write_output(output_str: String, output_path: &str) {
    let output_handle = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(output_path);
    let mut file = output_handle.expect("Unable to open file");
    file.write_all(output_str.as_bytes())
        .expect("Failed to write file");
}

fn main() {
    let args: Args = Args::parse();
    let base_file = args.base;
    let output_file = args.output;
    let doc_dir = args.directory;
    let exclude: Vec<&str> = args.exclude.split(",").collect();

    let mut base_spec = get_base_spec(base_file.as_str());
    let other_specs: Vec<Hash> = get_yaml_specs(doc_dir.as_str(), exclude);
    base_spec = merge_specs(base_spec, other_specs);
    let mut out_str = String::new();
    let mut emitter = YamlEmitter::new(&mut out_str);
    emitter.dump(&Yaml::Hash(base_spec)).unwrap();
    write_output(out_str, output_file.as_str());
    println!("YAML docs written to {:?} ", output_file);
}
