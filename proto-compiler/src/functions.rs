use git2::build::{CheckoutBuilder, RepoBuilder};
use git2::{AutotagOption, FetchOptions, Repository};
use std::fs::remove_dir_all;
use std::fs::{copy, create_dir_all};
use std::path::PathBuf;
use walkdir::WalkDir;

/// Clone/open, fetch and check out a specific commitish
pub fn get_commitish(dir: &PathBuf, url: &str, commitish: &str) {
    if dir.exists() {
        update_and_get_commitish(dir, commitish)
    } else {
        clone_and_get_commitish(dir, url, commitish)
    }
}

fn clone_and_get_commitish(dir: &PathBuf, url: &str, commitish: &str) {
    println!("  [info] => Cloning {} to {}", url, dir.to_string_lossy());

    let mut fo = FetchOptions::new();
    fo.download_tags(AutotagOption::All);
    fo.update_fetchhead(true);

    let mut builder = RepoBuilder::new();
    builder.fetch_options(fo);

    let repo = builder.clone(url, dir).unwrap();
    checkout_commitish(&repo, commitish);
}

fn update_and_get_commitish(dir: &PathBuf, commitish: &str) {
    println!("  [info] => Opening {}", dir.to_string_lossy());
    let repo = Repository::open(dir).unwrap();

    let mut fo = git2::FetchOptions::new();
    fo.download_tags(git2::AutotagOption::All);

    let mut remote = repo.find_remote("origin").unwrap();
    println!("  [info] => Fetching {} for repo", remote.name().unwrap());
    remote.fetch(&[commitish], Some(&mut fo), None).unwrap();

    let stats = remote.stats();
    if stats.local_objects() > 0 {
        println!(
            "  [info] => Received {}/{} objects in {} bytes (used {} local \
     objects)",
            stats.indexed_objects(),
            stats.total_objects(),
            stats.received_bytes(),
            stats.local_objects()
        );
    } else {
        println!(
            "  [info] => Received {}/{} objects in {} bytes",
            stats.indexed_objects(),
            stats.total_objects(),
            stats.received_bytes()
        );
    }

    checkout_commitish(&repo, commitish);
}

fn checkout_commitish(repo: &Repository, commitish: &str) {
    let ref_name = format!("refs/{}", commitish);
    let commitish_ref = repo.find_reference(&ref_name).unwrap();

    if commitish_ref.is_tag() {
        println!(
            "  [info] => Checking out repo in detached HEAD mode at {}",
            commitish
        );
        repo.set_head_detached(commitish_ref.target().unwrap())
            .unwrap();
    } else if commitish_ref.is_branch() {
        println!("  [info] => Checking out repo at branch {}", commitish);
        repo.set_head(&ref_name).unwrap();
    } else {
        panic!(
            "  [error] => Commitish \"{}\" is neither a tag nor a branch",
            commitish
        );
    }

    repo.checkout_head(Some(CheckoutBuilder::new().force()))
        .unwrap();
}

/// Copy generated files to target folder
pub fn copy_files(src_dir: PathBuf, target_dir: PathBuf) {
    // Remove old compiled files
    remove_dir_all(&target_dir).unwrap_or_default();
    create_dir_all(&target_dir).unwrap();

    // Copy new compiled files (prost does not use folder structures)
    let errors = WalkDir::new(&src_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| {
            copy(
                e.path(),
                std::path::Path::new(&format!(
                    "{}/{}",
                    &target_dir.display(),
                    &e.file_name().to_os_string().to_str().unwrap()
                )),
            )
        })
        .filter_map(|e| e.err())
        .collect::<Vec<_>>();

    if !errors.is_empty() {
        for e in errors {
            println!("[error] Error while copying compiled file: {}", e);
        }
        panic!("[error] Aborted.");
    }
}

/// Walk through the list of directories and gather all *.proto files
pub fn find_proto_files(proto_paths: Vec<String>) -> Vec<PathBuf> {
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
    protos
}
