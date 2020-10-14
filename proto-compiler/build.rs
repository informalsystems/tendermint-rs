use git2::build::RepoBuilder;
use git2::Repository;
use std::env::var;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const TENDERMINT_COMMITISH: &str = "tags/v0.34.0-rc5";

fn git_commitish(dir: &Path, url: &str, commitish: &str) {
    // Open repo
    let repo = match dir.exists() {
        false => {
            println!("Cloning {} to {}", url, dir.to_string_lossy());
            let mut fo = git2::FetchOptions::new();
            fo.download_tags(git2::AutotagOption::All);
            RepoBuilder::new()
                .fetch_options(fo)
                .clone(url, dir)
                .unwrap()
        }
        true => {
            println!("Opening {}", dir.to_string_lossy());
            let repo = Repository::open(dir).unwrap();
            let mut fo = git2::FetchOptions::new();
            fo.download_tags(git2::AutotagOption::All);
            let mut remote = repo.find_remote("origin").unwrap();
            println!("Fetching {} for repo", remote.name().unwrap());
            remote.fetch(&[commitish], Some(&mut fo), None).unwrap();
            let stats = remote.stats();
            if stats.local_objects() > 0 {
                println!(
                    "Received {}/{} objects in {} bytes (used {} local \
             objects)",
                    stats.indexed_objects(),
                    stats.total_objects(),
                    stats.received_bytes(),
                    stats.local_objects()
                );
            } else {
                println!(
                    "Received {}/{} objects in {} bytes",
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
        println!("repo is up to date");
    } else if analysis.0.is_fast_forward() {
        let refname = format!("refs/{}", commitish);
        let mut lb = repo.find_reference(&refname).unwrap();
        let name = match lb.name() {
            Some(s) => s.to_string(),
            None => String::from_utf8_lossy(lb.name_bytes()).to_string(),
        };
        let msg = format!(
            "Fast-Forward: Setting {} to id: {}",
            name,
            fetch_commit.id()
        );
        println!("{}", msg);
        lb.set_target(fetch_commit.id(), &msg).unwrap();
        repo.set_head(&name).unwrap();
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().safe()))
            .unwrap();
    } else {
        panic!("fast forward not possible ({:?})", analysis.0);
    }
}

fn main() {
    let tendermint_dir = var("TENDERMINT_DIR").unwrap_or_else(|_| "target/tendermint".to_string());
    git_commitish(
        Path::new(&tendermint_dir),
        &"https://github.com/tendermint/tendermint",
        TENDERMINT_COMMITISH,
    );

    let proto_paths = [
        format!("{}/proto", tendermint_dir),
    ];
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
    let mut pb = prost_build::Config::new();
    pb.compile_well_known_types();
    pb.type_attribute(".", "#[derive(::serde::Deserialize, ::serde::Serialize)]");
    pb.compile_protos(&protos, &includes).unwrap();
}
