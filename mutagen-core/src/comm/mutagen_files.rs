use anyhow::{bail, Result, Context};
use serde::{de::DeserializeOwned, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;

const DEFAULT_MUTAGEN_DIR: &str = "target/mutagen";
const DEFAULT_MUTAGEN_FILENAME: &str = "mutations";
const JSON_MUTAGEN_FILENAME: &str = "mutations.json";

const DEFAULT_COVERAGE_FILENAME: &str = "coverage";

/// Finds the file that contains the descriptions of all mutations as written by the procedural macro
pub fn get_mutations_file() -> Result<PathBuf> {
    Ok(mutagen_dir()?.join(DEFAULT_MUTAGEN_FILENAME))
}

pub fn get_mutations_file_json() -> Result<PathBuf> {
    Ok(mutagen_dir()?.join(JSON_MUTAGEN_FILENAME))
}

pub fn get_coverage_file() -> Result<PathBuf> {
    Ok(mutagen_dir()?.join(DEFAULT_COVERAGE_FILENAME))
}

/// queries `cargo` for the workspace root and locates the directory to write mutagen-specific information
fn mutagen_dir() -> Result<PathBuf> {
    let metadata = Command::new("cargo").arg("metadata").output()?;
    if !metadata.status.success() {
        bail!("{}", str::from_utf8(&metadata.stderr)?);
    }
    let meta_json = json::parse(str::from_utf8(&metadata.stdout)?)?;
    let root_dir = Path::new(
        meta_json["workspace_root"]
            .as_str()
            .with_context(|| "cargo metadata misses workspace_root")?,
    );
    Ok(root_dir.join(DEFAULT_MUTAGEN_DIR))
}

pub fn read_items<T: DeserializeOwned>(filepath: &Path) -> Result<Vec<T>> {
    BufReader::new(File::open(filepath)?)
        .lines()
        .map(|line| {
            serde_json::from_str(&line?).with_context(|| "mutation format error")
        })
        .collect()
}

pub fn append_item<T: Serialize + ?Sized>(file: &mut File, item: &T) -> Result<()> {
    let mut w = BufWriter::new(file);
    serde_json::to_writer(&mut w, item)?;
    writeln!(&mut w)?; // write newline
    Ok(())
}
