/// Result from a test run
#[derive(PartialEq, Eq, Copy, Clone)]
pub enum MutantStatus {
    /// The test suite did not cover the mutator
    NotCovered,
    /// test pass
    MutantSurvived,
    /// the test broke with an error code
    MutantKilled(Option<i32>),
    /// the test timed out
    Timeout,
}
