use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::ops::Range;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::collections::HashMap;

lazy_static! {
    static ref COVERAGE_ALREADY_CHECKED: Vec<AtomicBool> = {
        let max = env::var("MUTAGEN_MUTATION_AMOUNT").map(|s|s.parse().unwrap_or(0)).unwrap_or(0) + 1;
        (0..max).map(|_| AtomicBool::new(false)).collect()
    };
}

pub fn report_coverage(mutations: Range<usize>) {
    let mutagen_coverage = env::var_os("MUTAGEN_COVERAGE");
    if mutagen_coverage.is_none() {
        return
    }

    if let Some(ab) = COVERAGE_ALREADY_CHECKED.get(mutations.start) {
        if ab.compare_and_swap(false, true, Ordering::SeqCst) == false {
            if let Some(_) = mutagen_coverage {
                // TODO: Should parse the env var and check the reporting strategy: file, socket, ...
                let muts: Vec<String> = mutations
                    .map(|n| format!("{}", n))
                    .collect();

                OpenOptions::new()
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
    }
}