use std::sync::{Once, ONCE_INIT};
use std::env;
use std::fs::File;
use std::io::Write;

static REPORT_COVERAGE: Once = ONCE_INIT;

pub fn report_coverage() {
    REPORT_COVERAGE.call_once(|| {
        let mutagen_coverage = env::var_os("MUTAGEN_COVERAGE");

        if let Some(_) = mutagen_coverage {
            // Should parse the env var and check the reporting strategy: file, socket, ...
            File::create("target/mutagen/coverage.txt")
                .and_then(|mut f| f.write_all(b"covered"));
        }
    })
}