use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CoverageHit {
    pub mutator_id: usize,
}
