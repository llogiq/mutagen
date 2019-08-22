use failure::{bail, Fallible};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::process;
use std::process::{Command, Stdio};
use std::str;

use cargo_mutagen::*;
use mutagen_core::comm;
use mutagen_core::comm::{BakedMutation, CoverageHit, MutagenReport, MutantStatus};

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
        .map(|b| b.run_test(mutations.len()))
        .collect::<Fallible<Vec<_>>>()?;

    let coverage = read_coverage()?;

    // run the mutations on the test-suites
    run_mutations(&test_bins, mutations, &coverage)?;

    Ok(())
}

/// run all mutations on all test-executables
fn run_mutations(
    test_bins: &[TestBinTimed],
    mutations: Vec<BakedMutation>,
    coverage: &HashSet<usize>,
) -> Fallible<()> {
    let mut mutagen_report = MutagenReport::new();
    let mut progress = Progress::new(mutations.len());

    for m in mutations {
        progress.start_mutation(&m)?;

        let mutant_covered = coverage.contains(&m.mutator_id());
        let mutant_status = if mutant_covered {
            // run all test binaries
            let mut mutant_status = MutantStatus::Survived;
            for bin in test_bins {
                mutant_status = bin.check_mutant(&m)?;
                if mutant_status != MutantStatus::Survived {
                    break;
                }
            }
            mutant_status
        } else {
            MutantStatus::NotCovered
        };

        mutagen_report.add_mutation_result(m, mutant_status);

        progress.finish_mutation(mutant_status)?;
    }
    progress.finish()?;

    mutagen_report.summary().print();

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
fn read_coverage() -> Fallible<HashSet<usize>> {
    let coverage_file = comm::get_coverage_file()?;
    if !coverage_file.exists() {
        bail!("file `target/mutagen/coverage` is not found")
    }

    let coverage_hits = comm::read_items::<CoverageHit, _>(coverage_file)?;
    Ok(coverage_hits.iter().map(|c| c.mutator_id).collect())
}
