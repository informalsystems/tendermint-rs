use std::{
    env::var,
    fs,
    path::{Path, PathBuf},
    process,
};

use tempfile::tempdir;

mod buf_build;
use crate::buf_build::{export_dep_module, read_locked_deps};

mod functions;
use functions::{copy_files, find_proto_files, generate_cometbft_mod, get_commitish};

mod constants;
use constants::{CUSTOM_FIELD_ATTRIBUTES, CUSTOM_TYPE_ATTRIBUTES, TENDERMINT_VERSION};

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let target_dir = ["..", "..", "proto", "src"].iter().collect::<PathBuf>();
    let tendermint_dir = PathBuf::from(var("TENDERMINT_DIR").unwrap_or_else(|_| {
        root.join("..")
            .join("target")
            .join("tendermint")
            .to_str()
            .unwrap()
            .to_string()
    }));

    let version = &TENDERMINT_VERSION;

    println!(
        "[info] => Fetching {} at {} into {tendermint_dir:?}",
        version.repo, version.commitish,
    );
    get_commitish(&tendermint_dir, &version.repo, &version.commitish); // This panics if it fails.

    let proto_path = tendermint_dir.join("proto");

    let mut proto_includes_paths = vec![tendermint_dir.join("proto")];

    let buf_lock_path = proto_path.join("buf.lock");
    let _temp_dirs = if fs::metadata(&buf_lock_path).is_ok() {
        // A new-style proto module with buf dependencies.
        // Fetch the dependencies and add them to include paths.
        match read_locked_deps(&buf_lock_path) {
            Ok(deps) => deps
                .iter()
                .map(|dep| {
                    let mod_dir = tempdir().unwrap();
                    if let Err(e) = export_dep_module(dep, mod_dir.path()) {
                        eprintln!(
                            "Failed to export module {}/{}/{}: {}",
                            dep.remote, dep.owner, dep.repository, e,
                        );
                        process::exit(1);
                    }
                    proto_includes_paths.push(mod_dir.path().to_owned());
                    mod_dir
                })
                .collect::<Vec<_>>(),
            Err(e) => {
                eprintln!("Failed to read {}: {}", buf_lock_path.display(), e);
                process::exit(1);
            },
        }
    } else {
        // Old school, assume the dependency protos are bundled in the tree.
        proto_includes_paths.push(tendermint_dir.join("third_party").join("proto"));
        vec![]
    };

    // List available proto files
    let protos = find_proto_files(&proto_path);

    let ver_target_dir = target_dir.join("prost");

    let out_dir = var("OUT_DIR")
        .map(|d| Path::new(&d).to_path_buf())
        .or_else(|_| tempdir().map(|d| d.into_path()))
        .unwrap();

    let mut pb = prost_build::Config::new();

    // Use shared Bytes buffers for ABCI messages:
    pb.bytes(&[".cometbft.abci"]);

    // Compile proto files with added annotations, exchange prost_types to our own
    pb.out_dir(&out_dir);
    for type_attribute in CUSTOM_TYPE_ATTRIBUTES {
        pb.type_attribute(type_attribute.0, type_attribute.1);
    }
    for field_attribute in CUSTOM_FIELD_ATTRIBUTES {
        pb.field_attribute(field_attribute.0, field_attribute.1);
    }
    // The below in-place path redirection replaces references to the Duration
    // and Timestamp WKTs with our own versions that have valid doctest comments.
    // See also https://github.com/danburkert/prost/issues/374 .
    pb.extern_path(
        ".google.protobuf.Duration",
        "crate::google::protobuf::Duration",
    );
    pb.extern_path(
        ".google.protobuf.Timestamp",
        "crate::google::protobuf::Timestamp",
    );
    println!("[info] => Creating structs.");
    match pb.compile_protos(&protos, &proto_includes_paths) {
        Ok(()) => {},
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        },
    }

    println!(
        "[info] => Removing old structs and copying new structs to {}",
        ver_target_dir.to_string_lossy(),
    );
    copy_files(&out_dir, &ver_target_dir); // This panics if it fails.
    generate_cometbft_mod(&out_dir, version, &target_dir.join("cometbft.rs"));

    println!("[info] => Done!");
}
