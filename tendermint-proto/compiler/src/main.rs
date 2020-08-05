use std::fs::remove_dir_all;
use std::fs::{copy, create_dir_all};
use walkdir::WalkDir;

pub(crate) fn main() {
    let tendermint_proto_path = "tendermint-proto/src/prost";

    println!("{:?}", std::env::current_dir().unwrap());
    // Remove old compiled files
    remove_dir_all(tendermint_proto_path).unwrap_or_default();
    create_dir_all(tendermint_proto_path).unwrap();

    // Copy new compiled files (prost does not use folder structures)
    let err: Vec<std::io::Error> = WalkDir::new(env!("OUT_DIR"))
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| {
            copy(
                e.path(),
                std::path::Path::new(&format!(
                    "{}/{}",
                    tendermint_proto_path,
                    &e.file_name().to_os_string().to_str().unwrap()
                )),
            )
        })
        .filter_map(|e| e.err())
        .collect();

    if !err.is_empty() {
        for e in err {
            dbg!(e);
        }
        panic!("error while copying compiled files")
    }
}
