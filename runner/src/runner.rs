use std::process::{Command, Stdio};
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use std::fs::remove_file;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use wait_timeout::ChildExt;

/// Runner allows to execute the testsuite with the target mutation count
pub trait Runner {
    /// run executes the testsuite with the given mutation count and returns the output
    /// if all tests has passed
    fn run(&mut self, mutation_count: usize) -> Result<(), ()>;
}

pub struct RuntimeModifier {
    multiplier: u32,
    divisor: u32,
    baseline: Duration,
}

impl RuntimeModifier {
    fn new(mul: u32, div: u32, baseline: Duration) -> RuntimeModifier {
        RuntimeModifier {
            multiplier: mul,
            divisor: div,
            baseline
        }
    }

    fn compute(&self, duration: Duration) -> Duration {
        duration * self.multiplier / self.divisor + self.baseline
    }
}

fn to_result(cond: bool) -> Result<(), ()> {
    if cond { Ok(()) } else { Err(()) }
}

/// Full suite runner executes all the test at once, given the path of the executable
pub struct FullSuiteRunner {
    test_executable: PathBuf,
    runtime_mod: RuntimeModifier,
    timeout: Duration,
}

impl FullSuiteRunner {
    /// creates a runner from the test executable path
    pub fn new(test_executable: PathBuf) -> FullSuiteRunner {
        FullSuiteRunner {
            test_executable,
            runtime_mod: RuntimeModifier::new(5, 2, Duration::from_millis(250)),
            timeout: Duration::from_secs(0),
        }
    }
}

impl Runner for FullSuiteRunner {
    fn run(&mut self, mutation_count: usize) -> Result<(), ()> {
        let mut command = Command::new(&self.test_executable);
        command.env("MUTATION_COUNT", mutation_count.to_string())
            .stdout(Stdio::null());
        if mutation_count == 0 {
            let start = Instant::now();
            let status = command.status().expect("failed to execute process");
            self.timeout = self.runtime_mod.compute(start.elapsed());
            to_result(status.success())
        } else {
            let mut child = command.spawn().expect("failed to execute process");
            if let Some(status) = child.wait_timeout(self.timeout)
                    .expect("error while waiting for test") {
                to_result(status.success())
            } else {
                Err(())
            }
        }
    }
}

/// Coverage runner will only run those tests that are affected by, at least, one mutation. To check
/// which tests needs to be ran, it executes the suite with a specific environment variable that
/// reports if any mutation has been hit.
/// Note that, due to limitations on Rust's tests, they need to be executed one by one (so, one exec
/// by test), which may be non-performant if almost all the tests are mutated
pub struct CoverageRunner {
    test_executable: PathBuf,
    test_runtimes: HashMap<String, Duration>,
    runtime_mod: RuntimeModifier,
    tests_with_mutations: TestsByMutation,
}

impl CoverageRunner {
    /// creates a runner from the test executable path
    pub fn new(test_executable: PathBuf) -> CoverageRunner {
        CoverageRunner {
            test_executable,
            test_runtimes: HashMap::new(),
            runtime_mod: RuntimeModifier::new(2, 1, Duration::from_millis(50)),
            tests_with_mutations: TestsByMutation::new(),
        }
    }

    /// returns the tests names that has, at least, one mutation
    fn tests_with_mutations(&mut self) {
        let tests = self.read_tests().unwrap_or(Vec::new());
        self.check_test_coverage(tests);
    }

    /// Returns the list of tests present on the target binary
    fn read_tests(&self) -> Result<Vec<String>, ()> {
        let raw_tests = Command::new(&self.test_executable)
            .args(&["--list"])
            .output()
            .expect("Could not get the list of tests");

        let stdout = String::from_utf8_lossy(&raw_tests.stdout).into_owned();
        let tests = stdout
            .split('\n')
            .filter_map(|current: &str| {
                let parts: Vec<&str> = current.split(": ").collect();
                if parts.len() != 2 {
                    return None;
                }

                if parts.get(1)? != &"test" {
                    return None;
                }

                parts.get(0).map(|tn| tn.to_string())
            })
            .collect();

        Ok(tests)
    }

    /// Runs the given tests and returns a new collection which contains only the tests
    /// which contains some mutation
    fn check_test_coverage(&mut self, tests: Vec<String>) {
        let CoverageRunner {
            ref runtime_mod,
            ref test_executable,
            ref mut test_runtimes,
            ref mut tests_with_mutations,
        } = *self;
        for test_name in tests {
            let _res = remove_file("target/mutagen/coverage.txt");

            let start = Instant::now();
            let cmd_result = Command::new(test_executable)
                .args(&[&test_name])
                .env("MUTAGEN_COVERAGE", "file:target/mutagen/coverage.txt")
                .output();
            let runtime = runtime_mod.compute(start.elapsed());
            test_runtimes.insert(test_name.to_string(), runtime);

            let cmd_successful = cmd_result
                .map(|output| output.status.success())
                .unwrap_or(false);

            if !cmd_successful {
                return;
            }

            let _res = File::open("target/mutagen/coverage.txt")
                .map(|mut file| {
                    let mut s = String::new();
                    file.read_to_string(&mut s).unwrap();

                    s
                })
                .map(|contents| {
                    let mutations: Vec<usize> = contents
                        .split(",")
                        .map(|s| s.parse().unwrap_or(0usize))
                        .filter(|mutation_id| *mutation_id != 0)
                        .collect();


                    tests_with_mutations.add_test(&test_name, &mutations);
                });
        }
    }
}

fn execute(runtimes: &HashMap<String, Duration>,
           executable: &PathBuf,
           test_name: &str,
           mutation_count: usize) -> Result<(), ()> {
    let mut command = Command::new(executable);
    command.args(&[test_name])
           .env("MUTATION_COUNT", mutation_count.to_string())
           .stdout(Stdio::null());
    let timeout = runtimes[test_name];
    let mut child = command.spawn().expect("failed to execute process");
    if let Some(status) = child.wait_timeout(timeout).expect("error while waiting for test") {
        to_result(status.success())
    } else { // timeout
        Err(())
    }
}

impl Runner for CoverageRunner {
    fn run(&mut self, mutation_count: usize) -> Result<(), ()> {
        self.tests_with_mutations();
        let CoverageRunner {
            runtime_mod: _,
            ref test_executable,
            ref test_runtimes,
            ref tests_with_mutations
        } = *self;
        to_result(tests_with_mutations.tests(mutation_count)
                    .iter()
                    .map(|tn| execute(test_runtimes, test_executable, tn, mutation_count))
                    .all(|test_result| test_result.is_ok()))
    }
}

/// Tests by mutation records, per mutation, which tests has been executed by this mutation
#[derive(Clone)]
pub struct TestsByMutation {
    mutations: HashMap<usize, Vec<String>>,
}

impl TestsByMutation {
    /// Creates a new instance of TestsByMutation
    pub fn new() -> TestsByMutation {
        TestsByMutation {
            mutations: HashMap::new(),
        }
    }

    /// Records that target test has been executed by the given mutations identifiers
    pub fn add_test(&mut self, test: &String, mutations: &[usize]) {
        mutations
            .iter()
            .for_each(|mi| {
                let entry = self.mutations.entry(*mi).or_insert(Vec::new());
                entry.push(test.clone())
            })
    }

    pub fn tests(&self, mutation: usize) -> &[String] {
        self.mutations.get(&mutation).map(Vec::as_ref).unwrap_or(&[])
    }
}

#[cfg(test)]
mod tests {
    use super::TestsByMutation;

    #[test]
    fn it_returns_no_tests_on_empty() {
        let tbi = TestsByMutation::new();

        assert!(tbi.tests(4).is_empty())
    }

    #[test]
    fn it_returns_test_if_they_were_added() {
        let mut tbi = TestsByMutation::new();

        tbi.add_test(&"test_name".to_string(), &[5, 32]);
        tbi.add_test(&"test_name2".to_string(), &[5]);

        assert_eq!("test_name".to_string(), tbi.tests(5)[0]);
        assert_eq!("test_name2".to_string(), tbi.tests(5)[1]);
        assert_eq!("test_name".to_string(), tbi.tests(32)[0]);
    }
}
