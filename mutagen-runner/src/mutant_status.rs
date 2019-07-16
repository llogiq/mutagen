/// Result from a test run
#[derive(PartialEq, Eq)]
pub enum MutantStatus {
    /// test pass
    MutantSurvived,
    /// the test broke with an error code
    MutantKilled(Option<i32>),
    /// the test timed out
    Timeout,
}
