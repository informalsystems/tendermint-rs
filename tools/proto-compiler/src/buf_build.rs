//! Support for [buf](https://buf.build) build tools.

use serde::Deserialize;

use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;
use std::process::{Command, ExitStatus};

/// Errors that can occur when working with buf files.
#[derive(Debug, thiserror::Error)]
pub enum FileError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),
    #[error("unsupported lock file version {}", .encountered)]
    UnsupportedVersion { encountered: String },
}

/// Serialization object for the `buf.lock` file.
#[derive(Debug, Deserialize)]
struct LockFile {
    pub version: String,
    pub deps: Vec<LockedDep>,
}

/// Serialization object for an entry under the `deps` key in a `buf.lock` file.
#[derive(Debug, Deserialize)]
pub struct LockedDep {
    pub remote: String,
    pub owner: String,
    pub repository: String,
    pub commit: String,
}

pub fn read_locked_deps(lockfile_path: impl AsRef<Path>) -> Result<Vec<LockedDep>, FileError> {
    let file = File::open(lockfile_path)?;
    let lock_config: LockFile = serde_yaml::from_reader(BufReader::new(file))?;
    if lock_config.version != "v1" {
        return Err(FileError::UnsupportedVersion {
            encountered: lock_config.version,
        });
    }
    Ok(lock_config.deps)
}

/// Errors that can occur from invocation of the buf CLI tool.
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("failed to execute buf: {}", .0)]
    Spawn(#[from] io::Error),
    #[error("buf exited with {}", .0)]
    Failure(ExitStatus),
}

pub fn export_dep_module(dep: &LockedDep, out_dir: &Path) -> Result<(), ToolError> {
    let module_ref = format!(
        "{}/{}/{}:{}",
        dep.remote, dep.owner, dep.repository, dep.commit
    );
    println!("Exporting `{}` into `{}`", module_ref, out_dir.display());
    let status = Command::new("buf")
        .arg("export")
        .arg(module_ref)
        .arg("-o")
        .arg(out_dir)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(ToolError::Failure(status))
    }
}
