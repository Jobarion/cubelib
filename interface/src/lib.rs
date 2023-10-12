use std::collections::HashMap;
use std::str::FromStr;
use cubelib::algs::Algorithm;
use serde::{Deserialize, Serialize};
use cubelib::defs::*;
use cubelib::solution::Solution;

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
    #[serde(with = "StepKindDef")]
    pub kind: StepKind,
    pub alg: String,
    pub variant: String,
}

impl Into<SolutionStep> for cubelib::solution::SolutionStep {
    fn into(self) -> SolutionStep {
        SolutionStep {
            kind: self.kind,
            variant: self.variant,
            alg: self.alg.to_string(),
        }
    }
}

impl From<SolutionStep> for cubelib::solution::SolutionStep  {
    fn from(value: SolutionStep) -> cubelib::solution::SolutionStep {
        cubelib::solution::SolutionStep {
            kind: value.kind,
            variant: value.variant,
            alg: Algorithm::from_str(value.alg.as_str()).expect("Expected correct alg string"),
        }
    }
}

impl Into<Solution> for SolverResponse {
    fn into(self) -> Solution {
        Solution {
            steps: self.solution.into_iter()
                .map(|step| step.into())
                .collect(),
            ends_on_normal: true //we don't care right now
        }
    }
}

impl From<Solution> for SolverResponse {
    fn from(value: Solution) -> Self {
        Self {
            solution: value.steps.into_iter()
                .map(|step| step.into())
                .collect()
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct StepConfig {
    #[serde(with = "StepKindDef")]
    pub kind: StepKind,
    pub substeps: Vec<String>,
    pub min: u8,
    pub max: u8,
    #[serde(with = "NissSwitchTypeDef")]
    pub niss: NissSwitchType,
    pub params: HashMap<String, String>,
}

#[serde(remote = "StepKind")]
#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
pub enum StepKindDef {
    EO,
    RZP,
    DR,
    HTR,
    FR,
    FRLS,
    FIN
}

#[serde(remote = "NissSwitchType")]
#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
pub enum NissSwitchTypeDef {
    Never,
    Before,
    Always,
}