use std::str::FromStr;

use cubelib::solution::Solution;
use cubelib::steps::step::StepConfig;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct SolverRequest {
    pub scramble: String,
    pub steps: Vec<StepConfig>
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Debug)]
pub struct SolverResponse {
    pub solution: Solution
}