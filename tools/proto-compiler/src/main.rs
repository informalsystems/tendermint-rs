use std::{
    env::var,
    path::{Path, PathBuf},
    process,
};

use tempfile::tempdir;

mod functions;
use functions::{
    copy_files, find_proto_files, generate_tendermint_lib, generate_tendermint_mod, get_commitish,
};

mod constants;
use constants::{
    CUSTOM_FIELD_ATTRIBUTES, CUSTOM_TYPE_ATTRIBUTES, TENDERMINT_REPO, TENDERMINT_VERSIONS,
};

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

    for version in TENDERMINT_VERSIONS {
        println!(
            "[info] => Fetching {TENDERMINT_REPO} at {} into {tendermint_dir:?}",
            &version.commitish,
        );
        get_commitish(&tendermint_dir, TENDERMINT_REPO, &version.commitish); // This panics if it fails.

        let proto_paths = vec![tendermint_dir.join("proto")];
        let proto_includes_paths = vec![
            tendermint_dir.join("proto"),
            tendermint_dir.join("third_party").join("proto"),
        ];
        // List available proto files
        let protos = find_proto_files(proto_paths);

        let ver_target_dir = target_dir.join("prost").join(&version.ident);
        let ver_module_dir = target_dir.join("tendermint");

        let out_dir = var("OUT_DIR")
            .map(|d| Path::new(&d).join(&version.ident))
            .or_else(|_| tempdir().map(|d| d.into_path()))
            .unwrap();

        let mut pb = prost_build::Config::new();

        // Use shared Bytes buffers for ABCI messages:
        pb.bytes(&[".tendermint.abci"]);

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
        generate_tendermint_mod(&out_dir, &version, &ver_module_dir);
    }
    generate_tendermint_lib(TENDERMINT_VERSIONS, &target_dir.join("tendermint.rs"));

    println!("[info] => Done!");
}
