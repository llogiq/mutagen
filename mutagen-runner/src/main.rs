use failure::{bail, Fallible};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::path::PathBuf;
use std::process;
use std::process::{Command, Stdio};
use std::str;

use cargo_mutagen::*;
use mutagen_core::comm;
use mutagen_core::comm::{BakedMutation, CoverageHit};

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

    // collect mutations
    let mutations = read_mutations()?;

    let test_bins = tests_executables
        .iter()
        .map(|e| TestBin::new(&e))
        .map(|b| b.run_test(mutations.len() as u32))
        .collect::<Fallible<Vec<_>>>()?;

    let coverage = read_coverage()?;

    // run the mutations on the test-suites
    run_mutations(&test_bins, &mutations, &coverage)?;

    Ok(())
}

/// run all mutations on all test-executables
fn run_mutations(
    test_bins: &[TestBinTimed],
    mutations: &[BakedMutation],
    coverage: &HashSet<u32>,
) -> Fallible<()> {
    let mut killed = 0;
    let mut survived = 0;
    let mut not_covered = 0;

    for m in mutations {
        print!("{} ... ", m.log_string());
        std::io::stdout().flush()?;

        let mutant_covered = coverage.contains(&m.mutator_id());
        let mutant_status = if mutant_covered {
            // run all test binaries
            let mut mutant_status = MutantStatus::MutantSurvived;
            for bin in test_bins {
                mutant_status = bin.check_mutant(m)?;
                if mutant_status != MutantStatus::MutantSurvived {
                    break;
                }
            }
            mutant_status
        } else {
            MutantStatus::NotCovered
        };

        match mutant_status {
            MutantStatus::MutantSurvived => {
                survived += 1;
                println!("SURVIVED");
            }
            MutantStatus::NotCovered => {
                not_covered += 1;
                survived += 1;
                println!("NOT COVERED");
            }
            _ => {
                killed += 1;
                print!("killed");
                if mutant_status == MutantStatus::Timeout {
                    print!(" (timeout)");
                }
                println!();
            }
        }
    }

    let coverage = ((killed * 10000) / (killed + survived)) as f64 / 100.0;

    println!();
    println!("{} mutants killed", killed);
    println!(
        "{} mutants SURVIVED ({} NOT COVERED)",
        survived, not_covered
    );
    println!("{}% mutation coverage", coverage);

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

/// read all mutations from the given file
///
/// This functions gets the file that describes all mutations performed on the target program and ensures that it exists.
/// The list of mutations is also preserved
fn read_mutations() -> Fallible<Vec<BakedMutation>> {
    let mutations_file = comm::get_mutations_file()?;
    if !mutations_file.exists() {
        bail!(
            "file `target/mutagen/mutations` is not found\n\
             maybe there are no mutations defined or the attribute `#[mutate]` is not enabled"
        )
    }

    println!("mutations-file: {}", mutations_file.display());
    let mutations = comm::read_items::<BakedMutation, _>(mutations_file)?;

    // write the collected mutations
    let mutations_map = mutations
        .iter()
        .map(|m| (m.id(), m.as_ref()))
        .collect::<HashMap<_, _>>();
    let mutations_writer = BufWriter::new(File::create(comm::get_mutations_file_json()?)?);
    serde_json::to_writer(mutations_writer, &mutations_map)?;

    Ok(mutations)
}

/// read all coverage-hits from the coverage-file
fn read_coverage() -> Fallible<HashSet<u32>> {
    let coverage_file = comm::get_coverage_file()?;
    if !coverage_file.exists() {
        bail!("file `target/mutagen/coverage` is not found")
    }

    println!("coverage-file: {}", coverage_file.display());
    let coverage_hits = comm::read_items::<CoverageHit, _>(coverage_file)?;
    Ok(coverage_hits.iter().map(|c| c.mutator_id).collect())
}
