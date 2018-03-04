extern crate json;


use std::process::Command;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::from_utf8;

static TARGET_MUTAGEN: &'static str = "target/mutagen";
static MUTATIONS_LIST: &'static str = "mutations.txt";

fn run_mutation(mutation_count: usize) -> Result<String, String> {
    let output = Command::new("cargo")
        .args(&["test"])
        // 0 is actually no mutations so we need i + 1 here
        .env("MUTATION_COUNT", mutation_count.to_string())
        .output()
        .expect("failed to execute process");

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();

    if output.status.success() {
        Ok(stdout)
    } else {
        Err(stdout)
    }
}

fn run_mutations(list: Vec<String>) {
    let max_mutation = list.len();

    let mut failures = Vec::new();

    println!("Running {} mutations", max_mutation);
    for i in 0..max_mutation {
        print!("{}", list[i]);

        // Mutation count starts from 1 (0 is not mutations)
        let result = run_mutation(i + 1);

        if let Ok(stdout) = result {
            // A succeeding test suite is actually a failure for us.
            // At least on test should have failed
            println!(" ... FAILED");
            failures.push((&list[i], stdout))
        } else {
            println!(" ... ok");
        }
    }

    if !failures.is_empty() {
        println!("\nFailures:\n");

        for &(ref mutation, ref failure) in &failures {
            println!("---- {} stdout ----", mutation);
            for line in failure.split("\n") {
                println!("  {}", line);
            }
            println!("");
        }

        println!("\nFailures:\n");

        for &(mutation, _) in &failures {
            println!("\t{}", mutation);
        }
    }

    println!(
        "\nMutation results: {}. {} passed; {} failed",
        if failures.is_empty() { "ok" } else { "FAILED" },
        list.len() - failures.len(),
        failures.len()
    );
}

fn get_mutations_filename() -> PathBuf {
    let metadata = Command::new("cargo")
        .arg("metadata")
        .output()
        .expect("failed to fetch metadata. Is this a Rust project?");
    if !metadata.status.success() {
        println!("failed to fetch metadata, cargo returned non-zero status.");
        panic!("{}", from_utf8(&metadata.stderr).unwrap());
    }
    let meta_json = json::parse(from_utf8(&metadata.stdout)
                                                   .expect("non-UTF8 cargo output"))
                                  .unwrap();
    let root_dir = Path::new(meta_json["workspace_root"]
                                  .as_str()
                                  .expect("cargo metadata misses workspace_root"));
    let mutagen_dir = root_dir.join(TARGET_MUTAGEN);
    if !mutagen_dir.exists() {
        panic!("Mutations are missing");
    }
    mutagen_dir.join(MUTATIONS_LIST)
}

fn compile_tests() {
    // TODO make this actually work
    let compiled_tests = Command::new("cargo")
        .args(&["test", "--no-run"])
        .output()
        .expect("failed to execute process")
        .status
        .success();

    if !compiled_tests {
        panic!("Could not compile tests");
    }
}

fn read_mutations(filename: PathBuf) -> Vec<String> {
    let mut file = File::open(filename).expect("Mutations are missing");
    let mut s = String::new();
    file.read_to_string(&mut s)
        .expect("Failed reading mutations file");
    s.split("\n")
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect()
}

fn main() {
    compile_tests();
    let filename = get_mutations_filename();
    let list = read_mutations(filename);

    if let Err(_) = run_mutation(0){
        println!("You need to make sure you don't have failing tests before running 'cargo mutagen'");
        return;
    }

    run_mutations(list)
}
