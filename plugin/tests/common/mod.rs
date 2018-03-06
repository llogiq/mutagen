use std::io;
use std::path::Path;
use std::fs::File;
use std::io::Error;
use std::io::BufRead;

/// MutationsChecker allows to check if mutations has been generated on a given file.
pub struct MutationsChecker<'a> {
    filename: &'a str,
    mutations: Vec<String>,
}

impl<'a> MutationsChecker<'a> {
    pub fn new(filename: &'a str) -> Result<MutationsChecker<'a>, Error> {
        let lines = load_file("./tests/target/mutagen/mutations.txt")?;
        let mc = MutationsChecker {
            filename,
            mutations: lines,
        };

        Ok(mc)
    }

    /// returns true if the given mutation string can be found on the given location.
    /// TODO: This can be improved if we properly parse the mutations.txt file, as currently it
    /// only performs a sub-string search.
    pub fn has(&self, msg: &str, at: &str) -> bool {
        let location = format!("{}:{}", self.filename, at);

        self.mutations
            .iter()
            .find(|current| current.contains(msg) && current.contains(&location))
            .is_some()
    }

    /// returns true if all the given messages are at the given position
    pub fn has_multiple(&self, msg: &[&str], at: &str) -> bool {
        msg.iter().all(|current| self.has(current, at))
    }
}

/// Loads the given file returning the lines as a vector of strings
fn load_file<P: AsRef<Path>>(filename: P) -> Result<Vec<String>, Error> {
    let file = File::open(filename)?;
    let lines = io::BufReader::new(file).lines();

    Ok(lines.map(|l| l.unwrap_or("".to_string())).collect())
}
