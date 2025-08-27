use crate::cli::{LogLevel, SolutionFormat, SolveCommand, SolverBackend};
use core::fmt;
use cubelib::defs::StepKind;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer};
use serde_with::__private__::DeError;
use serde_with::{serde_as, DeserializeAs, KeyValueMap};
use std::collections::HashMap;
use std::str::FromStr;
use std::string::ToString;

#[derive(Debug, Deserialize)]
pub struct CubelibConfig {
    #[serde(default = "default_log_level")]
    pub log: LogLevel,
    #[serde(default = "default_check_update")]
    pub check_update: bool,
    #[serde(rename = "solver")]
    pub solver_config: SolverConfig,
}

fn default_log_level() -> LogLevel {
    LogLevel::Warn
}

fn default_check_update() -> bool {
    true
}

fn default_solver_config() -> SolverConfig {
    SolverConfig::default()
}

impl Default for CubelibConfig {
    fn default() -> Self {
        Self {
            log: default_log_level(),
            check_update: true,
            solver_config: default_solver_config(),
        }
    }
}

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct SolverConfig {
    #[serde(default = "default_quality")]
    pub quality: usize,
    #[serde(default = "default_format")]
    pub format: SolutionFormat,
    #[serde(default = "default_all_solutions")]
    pub all_solutions: bool,
    #[serde(default = "default_min")]
    pub min: usize,
    pub max: Option<usize>,
    pub solution_count: Option<usize>,
    #[serde(default = "default_steps")]
    pub steps: String,
    #[serde(default = "default_backend")]
    pub backend: SolverBackend,
    #[serde_as(as = "KeyValueMap<_>")]
    #[serde(default)]
    prototypes: Vec<StepOverrideInternal>,
}

fn default_quality() -> usize {
    100
}

fn default_all_solutions() -> bool {
    false
}

fn default_steps() -> String {
    "EO > RZP > DR[triggers=R,RUR,RU'R,RU2R] > HTR > FIN".to_string()
}

fn default_backend() -> SolverBackend {
    SolverBackend::MultiPathChannel
}

fn default_format() -> SolutionFormat {
    SolutionFormat::Detailed
}

fn default_min() -> usize {
    0
}

impl Default for SolverConfig {
    fn default() -> Self {
        SolverConfig {
            quality: default_quality(),
            format: default_format(),
            all_solutions: default_all_solutions(),
            min: default_min(),
            max: None,
            solution_count: None,
            steps: default_steps(),
            backend: default_backend(),
            prototypes: vec![],
        }
    }
}

#[derive(Clone, Debug)]
pub struct StepOverride {
    pub kind: StepKind,
    pub parameters: HashMap<String, String>,
}

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
struct StepOverrideInternal {
    #[serde(rename = "$key$")]
    name: String,
    parent: String,
    #[serde(flatten)]
    #[serde_as(as = "HashMap<_, DeserializeAsString>")]
    inner: HashMap<String, String>,
}

impl SolverConfig {
    pub fn get_merged_overrides(&self) -> HashMap<String, StepOverride> {
        let mut merged: HashMap<String, StepOverride> = HashMap::default();
        for so in &self.prototypes {
            let (parent, mut parent_params) = merged
                .get(&so.parent)
                .cloned()
                .map(|x| (x.kind, x.parameters))
                .unwrap_or((StepKind::from_str(&so.parent).unwrap(), Default::default()));
            parent_params.extend(so.inner.clone());
            merged.insert(
                so.name.clone(),
                StepOverride {
                    kind: parent,
                    parameters: parent_params,
                },
            );
        }
        merged
    }

    pub fn merge_cli_parameters(&mut self, cmd: SolveCommand) {
        if let Some(format) = cmd.format {
            self.format = format;
        }
        if let Some(all_solutions) = cmd.all_solutions {
            self.all_solutions = all_solutions;
        }
        if let Some(min) = cmd.min {
            self.min = min;
        }
        if let Some(max) = cmd.max {
            self.max = Some(max);
        }
        if let Some(solution_count) = cmd.solution_count {
            self.solution_count = Some(solution_count);
        }
        if let Some(quality) = cmd.quality {
            self.quality = quality;
        }
        if let Some(steps) = cmd.steps {
            self.steps = steps;
        }
        if let Some(backend) = cmd.backend {
            self.backend = backend;
        }
    }
}

struct DeserializeAsString;

impl<'de> DeserializeAs<'de, String> for DeserializeAsString {
    fn deserialize_as<D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Helper;
        impl Visitor<'_> for Helper {
            type Value = String;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a string")
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(value.to_string())
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(value.to_string())
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(value)
            }
        }

        deserializer.deserialize_any(Helper)
    }
}
