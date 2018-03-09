use std::process::Command;
use std::path::PathBuf;
use std::cell::RefCell;
use std::rc::Rc;
use std::fs::File;
use std::io::Read;
use std::fs::remove_file;
use std::collections::HashMap;

/// Runner allows to execute the testsuite with the target mutation count
pub trait Runner {
    /// run executes the testsuite with the given mutation count and returns the output
    /// if all tests has passed
    fn run(&self, mutation_count: usize) -> Result<String, String>;
}

/// Full suite runner executes all the test at once, given the path of the executable
pub struct FullSuiteRunner {
    test_executable: PathBuf,
}

impl FullSuiteRunner {
    /// creates a runner from the test executable path
    pub fn new(test_executable: PathBuf) -> FullSuiteRunner {
        FullSuiteRunner {
            test_executable
        }
    }
}

impl Runner for FullSuiteRunner {
    fn run(&self, mutation_count: usize) -> Result<String, String> {
        let output = Command::new(&self.test_executable)
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
}

/// Coverage runner will only run those tests that are affected by, at least, one mutation. To check
/// which tests needs to be ran, it executes the suite with a specific environment variable that
/// reports if any mutation has been hit.
/// Note that, due to limitations on Rust's tests, they need to be executed one by one (so, one exec
/// by test), which may be non-performant if almost all the tests are mutated
pub struct CoverageRunner {
    test_executable: PathBuf,
    tests_with_mutations: RefCell<Option<Rc<TestsByMutation>>>,
}

impl CoverageRunner {
    /// creates a runner from the test executable path
    pub fn new(test_executable: PathBuf) -> CoverageRunner {
        CoverageRunner {
            test_executable,
            tests_with_mutations: RefCell::new(None),
        }
    }

    /// returns the tests names that has, at least, one mutation
    fn tests_with_mutations(&self) -> Rc<TestsByMutation> {
        if let Some(ref twm) = *self.tests_with_mutations.borrow() {
            return twm.clone();
        }

        let tests = self.read_tests().unwrap_or(Vec::new());
        let tests_by_mutation = self.check_test_coverage(tests);

        let r = Rc::new(tests_by_mutation);
        *self.tests_with_mutations.borrow_mut() = Some(r.clone());
        r.clone()
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
    fn check_test_coverage(&self, tests: Vec<String>) -> TestsByMutation {
        let mut mutations_test = TestsByMutation::new();

        tests
            .into_iter()
            .for_each(|test_name| {
                remove_file("target/mutagen/coverage.txt");

                let cmd_result = Command::new(&self.test_executable)
                    .args(&[&test_name])
                    .env("MUTAGEN_COVERAGE", "file:target/mutagen/coverage.txt")
                    .output();

                let cmd_successful = cmd_result
                    .map(|output| output.status.success())
                    .unwrap_or(false);

                if !cmd_successful {
                    return;
                }

                let test_contains_mutation = File::open("target/mutagen/coverage.txt")
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


                        mutations_test.add_test(&test_name, &mutations);
                    });
            });

        mutations_test
    }

    fn execute(&self, test_name: &str, mutation_count: usize) -> Result<String, String> {
        let output = Command::new(&self.test_executable)
            .args(&[test_name])
            .args(&["--exact"])
            // 0 is actually no mutations so we need i + 1 here
            .env("MUTATION_COUNT", mutation_count.to_string())
            .output()
            .map_err(|_| "could not execute test".to_string())?;

        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        if output.status.success() {
            Ok(stdout)
        } else {
            Err(stdout)
        }
    }
}

impl Runner for CoverageRunner {
    fn run(&self, mutation_count: usize) -> Result<String, String> {
        let test_by_mutation = self.tests_with_mutations();

        let out: (String, bool) = test_by_mutation
            .tests(mutation_count)
            .map(|tests| {
                tests
                    .iter()
                    .map(|tn| self.execute(tn, mutation_count))
                    .fold((String::new(), true), |mut acc, test_result| {
                        match test_result {
                            Ok(stdout) => {
                                acc.0.push_str(&stdout);
                                (acc.0, acc.1 & true)
                            },
                            Err(stdout) => {
                                acc.0.push_str(&stdout);

                                (acc.0, acc.1 & false)
                            }
                        }
                    })
            })
            .unwrap_or((String::new(), true));

        if out.1 == true {
            Ok(out.0)
        } else {
            Err(out.0)
        }
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

    pub fn tests(&self, mutation: usize) -> Option<&Vec<String>> {
        self.mutations.get(&mutation)
    }
}

#[cfg(test)]
mod tests {
    use super::TestsByMutation;

    #[test]
    fn it_returns_no_tests_on_empty() {
        let tbi = TestsByMutation::new();

        assert!(tbi.tests(4).is_none())
    }

    #[test]
    fn it_returns_test_if_they_were_added() {
        let mut tbi = TestsByMutation::new();

        tbi.add_test(&"test_name".to_string(), &[5, 32]);
        tbi.add_test(&"test_name2".to_string(), &[5]);

        assert_eq!("test_name".to_string(), tbi.tests(5).unwrap()[0]);
        assert_eq!("test_name2".to_string(), tbi.tests(5).unwrap()[1]);
        assert_eq!("test_name".to_string(), tbi.tests(32).unwrap()[0]);
    }
}