use anyhow::{bail, Result};
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::process;
use std::process::{Command, Stdio};
use std::str;
use std::time::Instant;

use cargo_mutagen::*;
use mutagen_core::comm;
use mutagen_core::comm::{BakedMutation, CoverageCollection, MutagenReport, MutantStatus};

fn main() {
    if let Err(err) = run() {
        eprintln!();
        eprintln!("Error!");
        eprintln!("{}", err);
        process::exit(1);
    }
}

use structopt::StructOpt;
#[derive(StructOpt, Debug)]
struct Options {
    /// Space-separated list of features to activate
    #[structopt(long, name = "FEATURES")]
    features: Option<String>,

    /// Activate all available features
    #[structopt(long)]
    all_features: bool,

    /// Package to run tests for
    #[structopt(long, name = "SPEC")]
    package: Option<String>,

    /// Test all packages in the workspace
    #[structopt(long)]
    workspace: bool,
}

fn run() -> Result<()> {
    let mutagen_start = Instant::now();

    // drop "mutagen" arg in cargo-subcommand mode
    let mut args = env::args();
    if env::var("CARGO").is_ok() {
        // we're invoked by cargo, drop the first arg
        args.next();
    }
    let opt = Options::from_iter(args);

    // build the testsuites and collect mutations
    let test_bins = compile_tests(&opt)?;
    if test_bins.is_empty() {
        bail!("no test executable(s) found");
    }
    let mutations = read_mutations()?;
    let num_mutations = mutations.len();

    let mut progress = Progress::new(mutations.len());
    progress.summary_compile(mutations.len(), test_bins.len())?;

    // run all test-binaries without mutations and collect coverge
    progress.section_testsuite_unmutated(test_bins.len())?;

    let test_bins = test_bins
        .iter()
        .enumerate()
        .map(|(i, e)| TestBin::new(e, i))
        .filter_map(|bin| {
            bin.run_test(&mut progress, &mutations)
                .map(|bin| Some(bin).filter(|bin| bin.coveres_any_mutation()))
                .transpose()
        })
        .collect::<Result<Vec<_>>>()?;

    let coverage = CoverageCollection::merge(num_mutations, test_bins.iter().map(|b| &b.coverage));
    progress.summary_testsuite_unmutated(coverage.num_covered())?;

    // run the mutations on the test-suites
    progress.section_mutants()?;
    let mutagen_report = run_mutations(&mut progress, &test_bins, mutations, &coverage)?;

    progress.section_summary()?;

    // final report
    mutagen_report.print_survived();
    mutagen_report.summary().print();

    progress.finish(mutagen_start.elapsed())?;

    Ok(())
}

/// run all mutations on all test-executables
fn run_mutations(
    progress: &mut Progress,
    test_bins: &[TestBinTested],
    mutations: Vec<BakedMutation>,
    coverage: &CoverageCollection,
) -> Result<MutagenReport> {
    let mut mutagen_report = MutagenReport::new();

    for m in mutations {
        let mutant_status = if coverage.is_covered(m.id()) {
            progress.start_mutation_covered(&m)?;

            // run all test binaries
            let mut mutant_status = MutantStatus::Survived;
            for bin in test_bins {
                mutant_status = bin.check_mutant(&m)?;
                if mutant_status != MutantStatus::Survived {
                    break;
                }
            }
            progress.finish_mutation(mutant_status)?;

            mutant_status
        } else {
            progress.skip_mutation_uncovered(&m)?;
            MutantStatus::NotCovered
        };
        mutagen_report.add_mutation_result(m, mutant_status);
    }

    Ok(mutagen_report)
}

/// build all tests and collect test-suite executables
fn compile_tests(opt: &Options) -> Result<Vec<PathBuf>> {
    let mut tests: Vec<PathBuf> = Vec::new();

    let mut feature_args: Vec<&str> = vec![];
    if let Some(f) = &opt.features {
        feature_args.extend(&["--features", f]);
    }
    if opt.all_features {
        feature_args.push("--all-features");
    }
    if let Some(p) = &opt.package {
        feature_args.extend(&["--package", p]);
    }
    if opt.workspace {
        feature_args.push("--workspace");
    }

    // execute `cargo test --no-run --message-format=json` and collect output
    let compile_out = Command::new("cargo")
        .args(&["test", "--no-run", "--message-format=json"])
        .args(&feature_args)
        .stderr(Stdio::inherit())
        .output()?;
    if !compile_out.status.success() {
        bail!("`cargo test --no-run` returned non-zero exit status");
    }
    let compile_stdout = str::from_utf8(&compile_out.stdout)?;

    // each line is a json-value, we want to extract the test-executables
    // these are compiler artifacts that have set `test:true` in the profile
    let current_dir = std::env::current_dir()?;
    for line in compile_stdout.lines() {
        let msg_json = json::parse(line)?;
        if msg_json["reason"].as_str() == Some("compiler-artifact")
            && msg_json["profile"]["test"].as_bool() == Some(true)
        {
            let mut test_exe: PathBuf = msg_json["executable"].as_str().unwrap().to_string().into();

            // if the executable is found in the `deps` folder, execute it from there instead
            let test_exe_in_deps_dir = test_exe
                .parent()
                .unwrap()
                .join("deps")
                .join(test_exe.file_name().unwrap());
            if test_exe_in_deps_dir.exists() {
                test_exe = test_exe_in_deps_dir
            }

            // try to make path relative to current path
            test_exe = test_exe
                .strip_prefix(&current_dir)
                .map(|x| x.to_owned())
                .unwrap_or(test_exe);

            tests.push(test_exe);
        }
    }
    Ok(tests)
}

/// read all mutations from the given file
///
/// This functions gets the file that describes all mutations performed on the target program and ensures that it exists.
/// The list of mutations is also preserved
fn read_mutations() -> Result<Vec<BakedMutation>> {
    let mutations_file = comm::get_mutations_file()?;
    if !mutations_file.exists() {
        bail!(
            "file `target/mutagen/mutations` is not found\n\
             maybe there are no mutations defined or the attribute `#[mutate]` is not enabled"
        )
    }

    let mutations = comm::read_items::<BakedMutation>(&mutations_file)?;

    // write the collected mutations
    let mutations_map = mutations
        .iter()
        .map(|m| (m.id(), m.as_ref()))
        .collect::<HashMap<_, _>>();
    let mutations_writer = BufWriter::new(File::create(comm::get_mutations_file_json()?)?);
    serde_json::to_writer(mutations_writer, &mutations_map)?;

    Ok(mutations)
}
