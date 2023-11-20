use std::{
    collections::BTreeSet,
    fs::{copy, create_dir_all, remove_dir_all, File},
    io::Write,
    path::{Path, PathBuf},
};

use git2::{
    build::{CheckoutBuilder, RepoBuilder},
    AutotagOption, Commit, FetchOptions, Oid, Reference, Repository,
};
use subtle_encoding::hex;
use walkdir::WalkDir;
use std::io::Error;

use crate::constants::TendermintVersion;

/// Clone or open+fetch a repository and check out a specific commitish
/// In case of an existing repository, the origin remote will be set to `url`.
pub fn get_commitish(dir: &Path, url: &str, commitish: &str) {
    let repo = if dir.exists() {
        fetch_existing(dir, url)
    } else {
        clone_new(dir, url)
    };
    checkout_commitish(&repo, commitish)
}

fn clone_new(dir: &Path, url: &str) -> Repository {
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

fn fetch_existing(dir: &Path, url: &str) -> Repository {
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
        },
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

    let mut try_reference = repo.resolve_reference_from_short_name(commitish);
    if try_reference.is_err() {
        // Local branch might be missing, try the remote branch
        try_reference = repo.resolve_reference_from_short_name(&format!("origin/{commitish}"));
        tried_origin = true;
        if try_reference.is_err() {
            // Remote branch not found, last chance: try as a commit ID
            // Note: Oid::from_str() currently does an incorrect conversion and cuts the second half
            // of the ID. We are falling back on Oid::from_bytes() for now.
            let commitish_vec =
                hex::decode(commitish).unwrap_or_else(|_| hex::decode_upper(commitish).unwrap());
            return (
                None,
                repo.find_commit(Oid::from_bytes(commitish_vec.as_slice()).unwrap())
                    .unwrap(),
            );
        }
    }

    let mut reference = try_reference.unwrap();
    if reference.is_branch() {
        if tried_origin {
            panic!("[error] => local branch names with 'origin/' prefix not supported");
        }
        try_reference = repo.resolve_reference_from_short_name(&format!("origin/{commitish}"));
        reference = try_reference.unwrap();
        if reference.is_branch() {
            panic!("[error] => local branch names with 'origin/' prefix not supported");
        }
    }

    let commit = reference.peel_to_commit().unwrap();
    (Some(reference), commit)
}

/// Copy generated files to target folder
pub fn copy_files(src_dir: &Path, target_dir: &Path, project:&str) {
    // Remove old compiled files
    remove_dir_all(target_dir).unwrap_or_default();
    create_dir_all(target_dir).unwrap();

    // Copy new compiled files (prost does not use folder structures)
    let project_dot = format!("{project}.");
    let errors = WalkDir::new(src_dir)
        .contents_first(true)
        .into_iter()
        .filter_entry(|e| {
            e.file_type().is_file()
                && e.file_name()
                    .to_str()
                    .map(|name| name.starts_with(&project_dot))
                    .unwrap_or(false)
        })
        .map(|res| {
            let e = res?;
            copy(e.path(), target_dir.join(e.file_name()))
        })
        .filter_map(|res| res.err())
        .collect::<Vec<_>>();

    if !errors.is_empty() {
        for e in errors {
            println!("[error] => Error while copying compiled file: {e}");
        }
        panic!("[error] => Aborted.");
    }
}

/// Walk through the directory recursively and gather all *.proto files
pub fn find_proto_files(proto_path: &Path) -> Vec<PathBuf> {
    let mut protos: Vec<PathBuf> = vec![];
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
    protos
}

struct ModNode {
    name: String,
    children: Vec<ModNode>,
}

impl ModNode {
    fn new(name: String) -> Self {
        ModNode {
            name: name,
            children: vec![],
        }
    }
    fn get_or_add_child(&mut self, child_name: String) -> &mut Self {
        let idx = self.children
                .iter()
                .position(|c| c.name == child_name)
                .unwrap_or_else(| | {
                    let new_child = ModNode::new(child_name);
                    self.children.push(new_child);
                    self.children.len() - 1
                });
        &mut self.children[idx]
    }
    fn traverse_children(&self, version: &str, file: &mut File, mod_names: &Vec<String>) -> Result<(), Error> {
        for child in &self.children {
            child.generate_mods(version, file, &mod_names)?;
            if mod_names.len() == 1 {
                writeln!(file)?
            }
        }
        Ok(())
    }
    fn generate_mods(&self, version: &str, file: &mut File, mod_names: &Vec<String>) -> Result<(), Error> {
        if mod_names.is_empty() { // Top module, shouldn't be present
            return self.traverse_children(version, file, &vec![self.name.clone()]);
        }

        let tabs = "    ".to_owned().repeat(mod_names.len() - 1);
        writeln!(file, "{}pub mod {} {{", tabs, self.name)?;

        let mut updated_mod_names = mod_names.clone();
        updated_mod_names.push(self.name.clone());
        self.traverse_children(version, file, &updated_mod_names)?;
        if self.children.is_empty() {
            writeln!(file, "    {}include!(\"../prost/{}/{}.rs\");",
                tabs,
                version,
                updated_mod_names.join(".")
            )?;
        }

        writeln!(file, "{tabs}}}")
    }
}

/// Create a module including generated content for the specified
/// Tendermint source version.
pub fn generate_tendermint_mod(prost_dir: &Path, version: &TendermintVersion, target_dir: &Path) -> Result<(), Error> {
    create_dir_all(target_dir)?;

    let project_dot = format!("{}.", version.project);
    let file_names = WalkDir::new(prost_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.file_name().to_str().unwrap().starts_with(&project_dot)
                && e.file_name().to_str().unwrap().ends_with(".rs")
        })
        .map(|d| d.file_name().to_str().unwrap().to_owned())
        .collect::<BTreeSet<_>>();
    let file_names = Vec::from_iter(file_names);

    let tendermint_mod_target = target_dir.join(format!("{}.rs", version.ident));
    let mut file =
        File::create(tendermint_mod_target)?;

    writeln!(&mut file, "//! Protobuf auto-generated sub-modules for Tendermint. DO NOT EDIT\n")?;

    let mut root_mod = ModNode::new(version.project.to_owned());
    for file_name in file_names {
        let parts: Vec<_> = file_name
            .strip_prefix(&project_dot)
            .unwrap()
            .strip_suffix(".rs")
            .unwrap()
            .split('.')
            .collect();
        let mut curr_mod = &mut root_mod;
        for part in parts {
            //eprintln!("Part: {part}");
            curr_mod = curr_mod.get_or_add_child(part.to_owned());
        }
    }
    root_mod.generate_mods(&version.ident, &mut file, &vec![])?;

    // Add meta
    let tab = "    ".to_string();
    writeln!(&mut file,
        "pub mod meta {{\n{}pub const REPOSITORY: &str = \"{}\";\n{}pub const COMMITISH: &str = \"{}\";\n}}",
        tab,
        version.repo,
        tab,
        version.commitish,
    )
}

pub fn generate_tendermint_lib(versions: &[TendermintVersion], tendermint_lib_target_dir: &Path, project: &str, write_use: bool) -> Result<(), Error> {
    let tendermint_lib_target = tendermint_lib_target_dir.join(format!("{project}.rs"));
    let mut file =
        File::create(tendermint_lib_target).expect("tendermint library file create failed");
    let project_versions = versions.iter().filter(|v| v.project == project).collect::<Vec<_>>();
    for version in &project_versions {
        writeln!(&mut file, "pub mod {};", version.ident)?;
    }
    if write_use {
        let last_version = project_versions.last().unwrap();
        writeln!(&mut file, "pub use {}::*;", last_version.ident)?
    }
    Ok(())
}
