use git2::build::{CheckoutBuilder, RepoBuilder};
use git2::{AutotagOption, Commit, FetchOptions, Oid, Reference, Repository};
use std::fs::remove_dir_all;
use std::fs::{copy, create_dir_all};
use std::path::PathBuf;
use walkdir::WalkDir;

/// Clone or open+fetch a repository and check out a specific commitish
/// In case of an existing repository, the origin remote will be set to `url`.
pub fn get_commitish(dir: &PathBuf, url: &str, commitish: &str) {
    let repo = if dir.exists() {
        fetch_existing(dir, url)
    } else {
        clone_new(dir, url)
    };
    checkout_commitish(&repo, commitish)
}

fn clone_new(dir: &PathBuf, url: &str) -> Repository {
    println!(
        "  [info] => Cloning {} into {} folder",
        url,
        dir.to_string_lossy()
    );

    let mut fo = FetchOptions::new();
    fo.download_tags(AutotagOption::All);
    fo.update_fetchhead(true);

    let mut builder = RepoBuilder::new();
    builder.fetch_options(fo);

    builder.clone(url, dir).unwrap()
}

fn fetch_existing<'a>(dir: &PathBuf, url: &str) -> Repository {
    println!(
        "  [info] => Fetching from {} into existing {} folder",
        url,
        dir.to_string_lossy()
    );
    let repo = Repository::open(dir).unwrap();

    let mut fo = git2::FetchOptions::new();
    fo.download_tags(git2::AutotagOption::All);
    fo.update_fetchhead(true);

    let mut remote = repo
        .find_remote("origin")
        .unwrap_or_else(|_| repo.remote("origin", url).unwrap());
    if remote.url().is_none() || remote.url().unwrap() != url {
        repo.remote_set_url("origin", url).unwrap();
    }
    println!("  [info] => Fetching repo using remote `origin`");
    let specs: &[&str] = &[];
    remote.fetch(specs, Some(&mut fo), None).unwrap();

    let stats = remote.stats();
    if stats.local_objects() > 0 {
        println!(
            "  [info] => Received {}/{} objects in {} bytes (used {} local objects)",
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

fn checkout_commitish(repo: &Repository, commitish: &str) {
    let (reference, commit) = find_reference_or_commit(repo, commitish);

    println!(
        "  [info] => Checking out repo in detached HEAD mode:\n    \
             [info] => id: {},\n    \
             [info] => author: {},\n    \
             [info] => committer: {},\n    \
             [info] => summary: {}",
        commit.id(),
        commit.author(),
        commit.committer(),
        commit.summary().unwrap_or(""),
    );

    match reference {
        None => repo.set_head_detached(commit.id()).unwrap(),
        Some(reference) => {
            println!("    [info] => name: {}", reference.shorthand().unwrap());
            repo.set_head(reference.name().unwrap()).unwrap();
        }
    }

    let mut checkout_options = CheckoutBuilder::new();
    checkout_options
        .force()
        .remove_untracked(true)
        .remove_ignored(true)
        .use_theirs(true);
    repo.checkout_head(Some(&mut checkout_options)).unwrap();
}

fn find_reference_or_commit<'a>(
    repo: &'a Repository,
    commitish: &str,
) -> (Option<Reference<'a>>, Commit<'a>) {
    let mut tried_origin = false; // we tried adding 'origin/' to the commitish

    let mut try_reference = repo.resolve_reference_from_short_name(&commitish);
    if try_reference.is_err() {
        // Local branch might be missing, try the remote branch
        try_reference = repo.resolve_reference_from_short_name(&format!("origin/{}", commitish));
        tried_origin = true;
        if try_reference.is_err() {
            // Remote branch not found, last chance: try as a commit ID
            return (
                None,
                repo.find_commit(Oid::from_str(commitish).unwrap()).unwrap(),
            );
        }
    }

    let mut reference = try_reference.unwrap();
    if reference.is_branch() {
        if tried_origin {
            panic!("[error] => local branch names with 'origin/' prefix not supported");
        }
        try_reference = repo.resolve_reference_from_short_name(&format!("origin/{}", commitish));
        reference = try_reference.unwrap();
        if reference.is_branch() {
            panic!("[error] => local branch names with 'origin/' prefix not supported");
        }
    }

    let commit = reference.peel_to_commit().unwrap();
    (Some(reference), commit)
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
            println!("[error] => Error while copying compiled file: {}", e);
        }
        panic!("[error] => Aborted.");
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
