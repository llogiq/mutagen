use failure::{bail, format_err, Fallible};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;

const DEFAULT_MUTAGEN_DIR: &str = "target/mutagen";
const DEFAULT_MUTAGEN_FILENAME: &str = "mutations";

/// Finds the file that contains the descriptions of all mutations
///
/// This function is used to locate and to create the file
pub fn get_mutations_file() -> Fallible<PathBuf> {
    let metadata = Command::new("cargo").arg("metadata").output()?;
    if !metadata.status.success() {
        bail!("{}", str::from_utf8(&metadata.stderr)?);
    }
    let meta_json = json::parse(str::from_utf8(&metadata.stdout)?)?;
    let root_dir = Path::new(
        meta_json["workspace_root"]
            .as_str()
            .ok_or_else(|| format_err!("cargo metadata misses workspace_root"))?,
    );
    let mutagen_dir = root_dir.join(DEFAULT_MUTAGEN_DIR);
    let mutagen_file = mutagen_dir.join(DEFAULT_MUTAGEN_FILENAME);
    Ok(mutagen_file)
}
