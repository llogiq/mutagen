#[macro_use]
extern crate failure;
extern crate json;

mod runner;

use std::process::{self, Command, Stdio};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::from_utf8;
use runner::{CoverageRunner, FullSuiteRunner, Runner};

static TARGET_MUTAGEN: &'static str = "target/mutagen";
static MUTATIONS_LIST: &'static str = "mutations.txt";

type Result<T> = std::result::Result<T, failure::Error>;

fn run_mutations(runner: Box<Runner>, list: &[String]) {
    let max_mutation = list.len();

    let mut failures = Vec::new();

    println!("Running {} mutations", max_mutation);
    for i in 0..max_mutation {
        // Mutation count starts from 1 (0 is not mutations)
        let mutation_count = i + 1;

        print!("{} ({})", list[i], mutation_count);

        let result = runner.run(mutation_count);

        if let Ok(stdout) = result {
            // A succeeding test suite is actually a failure for us.
            // At least on test should have failed
            failures.push((&list[i], mutation_count, stdout));
            println!(" ... FAILED");
        } else {
            println!(" ... ok");
        }
    }

    if !failures.is_empty() {
        println!("\nFailures:\n");

        for &(ref mutation, ref m_count, ref failure) in &failures {
            println!("---- {} ({}) stdout ----", mutation, m_count);
            for line in failure.split("\n") {
                println!("  {}", line);
            }
            println!("");
        }

        println!("\nFailures:\n");

        for &(mutation, m_count, _) in &failures {
            println!("\t{} ({})", mutation, m_count);
        }
    }

    println!(
        "\nMutation results: {}. {} passed; {} failed",
        if failures.is_empty() { "ok" } else { "FAILED" },
        list.len() - failures.len(),
        failures.len()
    );
}

fn get_mutations_filename() -> Result<PathBuf> {
    let metadata = Command::new("cargo").arg("metadata").output()?;
    let stderr = from_utf8(&metadata.stderr)?;
    if !metadata.status.success() {
        bail!("{}", stderr);
    }
    let stdout = from_utf8(&metadata.stdout)?;
    let meta_json = json::parse(stdout)?;
    let root_dir = Path::new(
        meta_json["workspace_root"]
            .as_str()
            .expect("cargo metadata misses workspace_root"),
    );
    let mutagen_dir = root_dir.join(TARGET_MUTAGEN);
    if !mutagen_dir.exists() {
        bail!("mutations are missing")
    }
    Ok(mutagen_dir.join(MUTATIONS_LIST))
}

fn compile_tests() -> Result<Vec<PathBuf>> {
    let mut tests: Vec<PathBuf> = Vec::new();
    let compile_out = Command::new("cargo")
        .args(&["test", "--no-run", "--message-format=json"])
        // We need to skip first two arguments (path to mutagen binary and "mutagen" string)
        .args(std::env::args_os().skip(2))
        .stderr(Stdio::inherit())
        .output()?;

    if !compile_out.status.success() {
        bail!("cargo test returned non-zero status");
    }
    let compile_stdout = from_utf8(&compile_out.stdout)?;
    for line in compile_stdout.lines() {
        let msg_json = json::parse(line)?;
        if msg_json["reason"].as_str().unwrap() == "compiler-artifact"
            && msg_json["profile"]["test"].as_bool().unwrap_or(false)
        {
            for filename in msg_json["filenames"].members() {
                let f = filename.as_str().unwrap();
                if !f.ends_with(".rlib") {
                    tests.push(f.to_string().into());
                }
            }
        }
    }
    Ok(tests)
}

fn read_mutations(filename: &PathBuf) -> Result<Vec<String>> {
    let mut file = File::open(filename)?;
    let mut s = String::new();
    file.read_to_string(&mut s)?;
    Ok(s.split("\n")
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect())
}

fn has_flag(flag: &str) -> bool {
    let mut args = std::env::args_os();

    args.find(|f| f == flag).is_some()
}

fn run() -> Result<()> {
    let tests_executable = compile_tests()?;
    if tests_executable.is_empty() {
        bail!("executable path not found");
    }
    let filename = get_mutations_filename()?;
    let list = read_mutations(&filename)?;

    let with_coverage = has_flag("--coverage");
    for test_executable in tests_executable {
        println!("test executable at {:?}", test_executable);
        let runner: Box<Runner> = if with_coverage {
            Box::new(CoverageRunner::new(test_executable.clone()))
        } else {
            Box::new(FullSuiteRunner::new(test_executable.clone()))
        };

        if let Err(_) = runner.run(0) {
            bail!(
                "You need to make sure you don't have failing tests before running 'cargo mutagen'"
            );
        }

        run_mutations(runner, &list)
    }
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{}", err);
        process::exit(1);
    }
}
