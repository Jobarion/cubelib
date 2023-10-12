use std::collections::HashMap;
use std::str::FromStr;
use cubelib::algs::{Algorithm, Solution};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct SolverRequest {
    pub scramble: String,
    pub quality: Option<usize>,
    pub steps: Vec<StepConfig>
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Debug)]
pub struct SolverResponse {
    pub solution: Vec<SolutionStep>
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Debug)]
pub struct SolutionStep {
    pub name: String,
    pub alg: String
}

impl Into<Solution> for SolverResponse {
    fn into(self) -> Solution {
        Solution {
            steps: self.solution.into_iter()
                .map(|step| (step.name, Algorithm::from_str(step.alg.as_str()).expect("Expect API to return correct algorithm")))
                .collect(),
            ends_on_normal: true //we don't care right now
        }
    }
}

impl From<Solution> for SolverResponse {
    fn from(value: Solution) -> Self {
        Self {
            solution: value.steps.into_iter()
                .map(SolutionStep::from)
                .collect()
        }
    }
}

impl From<(String, Algorithm)> for SolutionStep {
    fn from(value: (String, Algorithm)) -> Self {
        SolutionStep {
            name: value.0,
            alg: value.1.to_string()
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct StepConfig {
    pub kind: StepKind,
    pub substeps: Vec<String>,
    pub min: u8,
    pub max: u8,
    pub niss: NissSwitchType,
    pub params: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
pub enum StepKind {
    EO,
    RZP,
    DR,
    HTR,
    FR,
    FRLS,
    FIN
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
pub enum NissSwitchType {
    Never,
    Before,
    Always,
}