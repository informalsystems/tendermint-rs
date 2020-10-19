use git2::build::RepoBuilder;
use git2::Repository;
use std::fs::remove_dir_all;
use std::fs::{copy, create_dir_all};
use std::path::PathBuf;
use walkdir::WalkDir;

/// Clone/open, fetch and check out a specific commitish
pub fn get_commitish(dir: &PathBuf, url: &str, commitish: &str) {
    // Open repo
    let repo = match dir.exists() {
        false => {
            println!("  [info] => Cloning {} to {}", url, dir.to_string_lossy());
            let mut fo = git2::FetchOptions::new();
            fo.download_tags(git2::AutotagOption::All);
            RepoBuilder::new()
                .fetch_options(fo)
                .clone(url, dir)
                .unwrap()
        }
        true => {
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
            Repository::open(dir).unwrap()
        }
    };

    // Check out commitish and fast forward, if necessary
    let fetch_head = repo.find_reference("FETCH_HEAD").unwrap();
    let fetch_commit = repo.reference_to_annotated_commit(&fetch_head).unwrap();
    let analysis = repo.merge_analysis(&[&fetch_commit]).unwrap();
    if analysis.0.is_up_to_date() {
        println!("  [info] => repo is up to date");
    } else if analysis.0.is_fast_forward() {
        let refname = format!("refs/{}", commitish);
        let mut lb = repo.find_reference(&refname).unwrap();
        let name = match lb.name() {
            Some(s) => s.to_string(),
            None => String::from_utf8_lossy(lb.name_bytes()).to_string(),
        };
        let msg = format!(
            "  [info] => Fast-Forward: Setting {} to id: {}",
            name,
            fetch_commit.id()
        );
        println!("{}", msg);
        lb.set_target(fetch_commit.id(), &msg).unwrap();
        repo.set_head(&name).unwrap();
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().safe()))
            .unwrap();
    } else {
        panic!("  [error] => fast forward not possible ({:?})", analysis.0);
    }
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
