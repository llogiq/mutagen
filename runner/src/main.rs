#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::process::Command;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

static TARGET_MUTAGEN: &'static str = "target/mutagen";
static MUTATIONS_LIST: &'static str = "mutations.txt";

#[derive(Deserialize)]
struct Metadata {
    workspace_root: String,
}

fn run_mutation(i: usize) -> Result<String, String> {
    let output = Command::new("cargo")
        .args(&["test"])
        // 0 is actually no mutations so we need i + 1 here
        .env("MUTATION_COUNT", (i + 1).to_string())
        .output()
        .expect("failed to execute process");

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();

    if output.status.success() {
        Err(stdout)
    } else {
        Ok(stdout)
    }
}

fn run_mutations(list: Vec<String>) {
    let max_mutation = list.len();

    let mut failures = Vec::new();

    println!("Running {} mutations", max_mutation);
    for i in 0..max_mutation {
        print!("{}", list[i]);

        let result = run_mutation(i);

        if let Err(stdout) = result {
            println!(" ... FAILED");
            failures.push((&list[i], stdout))
        } else {
            //failures.push((&list[i], result.unwrap()));
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
    // TODO can we improve this without needing serde etc 
    // to basically get the root dir of the crate?
    let metadata = Command::new("cargo")
        .args(&["metadata"])
        .output()
        .expect("failed to execute process")
        .stdout;
    let metadata: Metadata = serde_json::from_slice(&metadata).unwrap();
    let root_dir = metadata.workspace_root;

    let mutagen_dir = Path::new(&root_dir).join(TARGET_MUTAGEN);
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
    run_mutations(list)
}
