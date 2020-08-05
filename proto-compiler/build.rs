extern crate prost_build;

use std::env::var;
use std::path::PathBuf;
use walkdir::WalkDir;

fn main() {
    // Paths
    // Assume that the tendermint Go repository was cloned into the current repository's
    // target/tendermint folder.
    let tendermint_go_path =
        var("TENDERMINT_DIR").unwrap_or_else(|_| "../target/tendermint".to_string());
    let proto_paths = [format!("{}/proto", tendermint_go_path)];
    let proto_includes_paths = [
        format!("{}/proto", tendermint_go_path),
        format!("{}/third_party/proto", tendermint_go_path),
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
    prost_build::compile_protos(&protos, &includes).unwrap();
}
