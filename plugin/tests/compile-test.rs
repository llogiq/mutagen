#![feature(test)]
extern crate compiletest_rs as compiletest;
extern crate test;
extern crate mutagen;

use std::path::PathBuf;
use compiletest::test_opts;
use compiletest::uidiff::diff_lines;
use std::sync::mpsc::channel;
use test::{MonitorMsg, TestResult};
use std::io::{Error, Read};
use std::path::Path;

fn run_mode(mode: &'static str) {
    let mut config = create_config(mode);
    config.mode = mode.parse().expect("Invalid mode");

    compiletest::run_tests(&config);
}

fn create_config(mode: &'static str) -> compiletest::Config {
    let mut config = compiletest::Config::default();

    config.src_base = PathBuf::from(format!("tests/{}", mode));
    config.link_deps(); // Populate config.target_rustcflags with dependencies on the path
    config.clean_rmeta(); // If your tests import the parent crate, this helps with E0464

    config
}

/// Compiles the given filename located on mutagen directory
fn compile(filename: &str) {
    let cfg = create_config("mutagen");
    let mut opts = test_opts(&cfg);
    opts.test_threads = Some(1);
    let mut file = cfg.src_base.clone();
    file.push(format!("{}.rs", filename));

    let path = compiletest::common::TestPaths {
        file,
        base: cfg.src_base.clone(),
        relative_dir: PathBuf::from(""),
    };

    let t = compiletest::make_test(&cfg, &path);
    let (tx, rx) = channel::<MonitorMsg>();
    test::run_test(&opts, false, t, tx.clone());
    let result = rx.recv().unwrap();
    match result.1 {
        TestResult::TrOk => (),
        _ => {
            panic!("Failed to run {}", std::str::from_utf8(&result.2).unwrap());
        }
    }

}


#[test]
fn compile_test() {
    run_mode("compile-fail");
    run_mode("run-pass");
}

#[test]
fn compile_mutations() {
    let files = ["binops", "interchange"];
    let mut results = Vec::new();
    std::fs::create_dir_all("target/mutagen").unwrap();

    for f in files.iter() {
        results.push(run_test(f));
    }

    let mut errors = false;
    for r in results {
        match r {
            Err(e) => {
                println!("Output for mutagen/{}.rs errored", e.0);
                println!("{}", e.1);
                errors = true;
            },
            _ => (),
        }
    }

    if errors {
        panic!()
    }
}

type TestErr<'a> = (&'a str, String);

fn run_test(target: &str) -> Result<&str, TestErr> {
    compile(target);
    std::fs::copy(
        "tests/mutagen/target/mutagen/mutations.txt",
        format!("target/mutagen/{}.mut", target)
    ).map_err(|s| (target, s.to_string()))?;

    let expected = load_file(format!("tests/mutagen/{}.mut", target)).map_err(|s| (target, s.to_string()))?;
    let mut expected = expected
        .split("\n")
        .collect::<Vec<&str>>();
    expected.sort();

    let current = load_file(format!("target/mutagen/{}.mut", target)).map_err(|s| (target, s.to_string()))?;
    let mut current = current
        .split("\n")
        .collect::<Vec<&str>>();
    current.sort();

    let diff = diff_lines(&current.join("\n"), &expected.join("\n"));

    if diff.len() > 0 {
        Err((target, diff.join("\n")))
    } else {
        Ok(target)
    }
}

/// Loads the given file returning the lines as a vector of strings
fn load_file<P: AsRef<Path>>(filename: P) -> Result<String, Error> {
    let mut file = std::fs::File::open(filename)?;
    let mut s = String::new();
    file.read_to_string(&mut s)?;

    Ok(s)
}
