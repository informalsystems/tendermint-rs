use git2::Repository;
use prost_build::compile_protos;
use std::env::var;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn main() {
    let tendermint_dir = var("TENDERMINT_DIR").unwrap_or_else(|_| "target/tendermint".to_string());
    if !Path::new(&tendermint_dir).exists() {
        let url = "https://github.com/tendermint/tendermint";
        Repository::clone(url, &tendermint_dir).unwrap();
    }
    let proto_paths = [format!("{}/proto", tendermint_dir)];
    let proto_includes_paths = [
        format!("{}/proto", tendermint_dir),
        format!("{}/third_party/proto", tendermint_dir),
    ];

    // List available proto files
    let mut protos: Vec<PathBuf> = vec![];
    for proto_path in &proto_paths {
        protos.append(
            &mut WalkDir::new(proto_path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.file_type().is_file()
                        && e.path().extension().is_some()
                        && e.path().extension().unwrap() == "proto"
                })
                .map(|e| e.into_path())
                .collect(),
        );
    }

    // List available paths for dependencies
    let includes: Vec<PathBuf> = proto_includes_paths.iter().map(PathBuf::from).collect();

    // Compile all proto files
    compile_protos(&protos, &includes).unwrap();
}
