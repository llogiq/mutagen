use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::ops::Range;
use std::sync::Mutex;
use std::collections::HashMap;

lazy_static! {
    static ref COVERAGE_ALREDY_CHECKED: Mutex<HashMap<usize, ()>> = Mutex::new(HashMap::new());
}

pub fn report_coverage(mutations: Range<usize>) {
    let mutagen_coverage = env::var_os("MUTAGEN_COVERAGE");
    if mutagen_coverage.is_none() {
        return
    }

    match COVERAGE_ALREDY_CHECKED.lock() {
        Ok(mut hash) => {
            hash.entry(mutations.start).or_insert_with(|| {
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

                ()
            });
        },
        _ => (),
    }
}