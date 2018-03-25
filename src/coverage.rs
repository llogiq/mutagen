use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::ops::Range;
use std::sync::atomic::{AtomicUsize, Ordering};

pub fn report_coverage(mutations: Range<usize>, flag: &AtomicUsize, mask: usize) {
    if flag.fetch_or(mask, Ordering::SeqCst) & mask != 0 {
        return // already logged
    }
    if let Some(_) = env::var_os("MUTAGEN_COVERAGE") {
        // TODO: Should parse the env var and check the reporting strategy: file, socket, ...
        let muts: Vec<String> = mutations
            .map(|n| format!("{}", n))
            .collect();

        let _res = OpenOptions::new()
            .create(true)
            .append(true)
            .truncate(false)
            .open("target/mutagen/coverage.txt")
            .and_then(|mut f| {
                let mut joined = muts.join(",");
                joined.push_str(",");

                f.write_all(joined.as_bytes())
            });
    }
}
