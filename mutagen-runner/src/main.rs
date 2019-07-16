#![feature(type_ascription)]

use failure::{bail, format_err, Fallible};
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process;
use std::process::{Command, Stdio};
use std::str;

use cargo_mutagen::*;
use mutagen::{get_mutations_file, BakedMutation};

fn main() {
    if let Err(err) = run() {
        eprintln!();
        eprintln!("Error!");
        eprintln!("{}", err);
        process::exit(1);
    }
}

fn run() -> Fallible<()> {
    // gather context information
    let tests_executables = compile_tests()?;
    if tests_executables.is_empty() {
        bail!("test executable(s) not found");
    }

    println!("test suites:");
    for e in &tests_executables {
        println!("{}", e.display());
    }

    let test_bins = tests_executables
        .iter()
        .map(|e| TestBin::new(&e))
        .map(|b| b.run_test())
        .collect::<Fallible<Vec<_>>>()?;

    // collect mutations
    let mutations = read_mutations(&check_mutations_file()?)?;

    println!("test mutants:");
    // run the mutations on the test-suites
    run_mutations(&test_bins, &mutations)?;

    Ok(())
}

/// run all mutations on all test-executables
fn run_mutations(test_bins: &[TestBinTimed], mutations: &[BakedMutation]) -> Fallible<()> {
    for m in mutations {
        print!("{} ... ", m.log_string());
        std::io::stdout().flush()?;

        let mut mutant_status = MutantStatus::MutantSurvived;

        for bin in test_bins {
            mutant_status = bin.check_mutant(m)?;
            if mutant_status != MutantStatus::MutantSurvived {
                break;
            }
        }

        if mutant_status == MutantStatus::MutantSurvived {
            println!("SURVIVED");
        } else {
            println!("killed");
        }
    }
    Ok(())
}

/// build all tests and collect test-suite executables
fn compile_tests() -> Fallible<Vec<PathBuf>> {
    let mut tests: Vec<PathBuf> = Vec::new();
    let compile_out = Command::new("cargo")
        .args(&["test", "--no-run", "--message-format=json"])
        .stderr(Stdio::inherit())
        .output()?;

    if !compile_out.status.success() {
        println!("{}", str::from_utf8(&compile_out.stdout)?);
        bail!("`cargo test --no-run` returned non-zero exit status");
    }
    let compile_stdout = str::from_utf8(&compile_out.stdout)?;
    for line in compile_stdout.lines() {
        let msg_json = json::parse(line)?;
        if msg_json["reason"].as_str() == Some("compiler-artifact")
            && msg_json["profile"]["test"].as_bool() == Some(true)
        {
            tests.push(msg_json["executable"].as_str().unwrap().to_string().into());
        }
    }
    Ok(tests)
}

/// This functions gets the file that describes all mutations performed on the target program and ensures that it exists.
fn check_mutations_file() -> Fallible<PathBuf> {
    let mutagen_file = get_mutations_file()?;
    if !mutagen_file.exists() {
        bail!(
            "file `target/mutagen/mutations.txt` is not found\n\
             maybe there are not mutations defined or the attribute `#[mutate]` is not enabled"
        )
    }
    Ok(mutagen_file)
}

/// read all mutations from the given file
fn read_mutations(mutations_file: &PathBuf) -> Fallible<Vec<BakedMutation>> {
    println!("mutations-file: {}", mutations_file.display());
    let mut mutations = BufReader::new(File::open(mutations_file)?)
        .lines()
        .map(|line| {
            serde_json::from_str(&line?).map_err(|e| format_err!("mutation format error: {}", e))
        })
        .collect::<Fallible<Vec<BakedMutation>>>()?;

    mutations.sort_unstable_by_key(BakedMutation::id);
    Ok(mutations)
}
