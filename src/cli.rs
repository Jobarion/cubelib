use std::collections::HashMap;
use std::str::FromStr;
use clap::Parser;
use itertools::Itertools;
use regex::Regex;
use crate::df_search::{NissType, SearchOptions};

#[derive(Parser)]
#[command(name = "Cubelib")]
#[command(author = "Jonas Balsfulland <cubelib@joba.me>")]
#[command(version = "1.1")]
pub struct Cli {
    #[arg(short, long, default_value_t = false, group = "log_level", help = "Enables more detailed logging")]
    pub verbose: bool,
    #[arg(long, default_value_t = false, group = "log_level", help = "Prints nothing but the solutions")]
    pub quiet: bool,
    #[arg(id = "compact", short = 'c', long = "compact", default_value_t = false, help = "Prints only the solution, and not the different steps")]
    pub compact_solutions: bool,
    #[arg(short = 'p', long = "plain", default_value_t = false, requires = "compact", help = "Does not print the number of moves of the solution")]
    pub plain_solution: bool,
    #[arg(short = 'a', long = "all", default_value_t = false, help = "Print solutions that would otherwise get filtered out. E.g. an EO ending in F'")]
    pub all_solutions: bool,
    #[arg(short = 'm', long = "min", default_value_t = 0, help = "Minimum length of solutions")]
    pub min: usize,
    #[arg(short = 'M', long = "max", help = "Maximum length of solutions")]
    pub max: Option<usize>,
    #[arg(short = 'N', long = "niss", default_value_t = false, help = "Allows using NISS in some parts of solution")]
    pub niss: bool,
    #[arg(short = 'n', help = "The number of solutions returned. By default 1 unless this option or --max is set")]
    pub solution_count: Option<usize>,
    #[arg(short = 'q', long = "quality", group = "limits", help = "Maximum number of solutions calculated per step variations (Variations like eoud and eofb count separately). Defaults to 100")]
    pub quality: Option<usize>,
    #[arg(short = 'l', long = "limit", group = "step-limits", help = "Number of solutions carried over between steps (i.e. number of EOs considered for DRs)")]
    pub step_limit: Option<usize>,
    #[arg(long = "optimal", groups = ["limits", "step-limits"], help = "Look for optimal solutions")]
    pub optimal: bool,
    #[arg(long = "steps", short = 's', default_value = "EO > DR > HTR > FR > FIN", help = "")]
    pub steps: String,
    pub scramble: String,
}

#[derive(Debug)]
pub struct StepConfig {
    pub kind: StepKind,
    pub substeps: Option<Vec<String>>,
    pub min: Option<u8>,
    pub max: Option<u8>,
    pub quality: Option<usize>,
    pub solution_count: Option<usize>,
    pub niss: Option<NissType>,
    pub params: HashMap<String, String>,
}

#[derive(Copy, Clone, Debug)]
pub enum StepKind {
    EO,
    DR,
    HTR,
    FR,
    FRLS,
    FIN
}

impl FromStr for StepKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "eo" => Ok(Self::EO),
            "dr" => Ok(Self::DR),
            "htr" => Ok(Self::HTR),
            "fr" => Ok(Self::FR),
            "frls" => Ok(Self::FRLS),
            "finish" | "fin" => Ok(Self::FIN),
            x=> Err(format!("Unknown step '{x}'"))
        }
    }
}

impl Cli {

    const DEFAULT_STEP_QUALITY: usize = 100;

    fn get_default_step_quality(&self) -> Option<usize> {
        if self.optimal {
            None
        } else {
            Some(self.quality.unwrap_or(Self::DEFAULT_STEP_QUALITY))
        }
    }

    fn get_default_niss_type(&self) -> Option<NissType> {
        if self.niss {
            None //This means undefined, so steps can define their own niss type.
        } else {
            Some(NissType::None)
        }
    }

    fn get_default_step_limit(&self) -> Option<usize> {
        if self.optimal {
            None
        } else {
            self.step_limit
        }
    }

    pub fn parse_step_configs(&self) -> Result<Vec<StepConfig>, String> {
        let step_name_regex = Regex::new("[A-Za-z0-9_-]").unwrap();
        let default_quality = self.get_default_step_quality();
        let default_solution_count = self.get_default_step_limit();
        let default_niss_type = self.get_default_niss_type();
        self.steps.split(">")
            .map(|step| step.trim())
            .map(|step| {
                let param_start = step.find("[");
                if param_start.is_none() {
                    if !step_name_regex.is_match(step) {
                        return Err(format!("Invalid step name {}", step));
                    }
                    return Ok(StepConfig {
                        kind: StepKind::from_str(step)?,
                        substeps: None,
                        min: None,
                        max: None,
                        niss: default_niss_type,
                        solution_count: default_solution_count,
                        quality: default_quality,
                        params: HashMap::new()
                    });
                } else {
                    if !step.ends_with("]") {
                        return Err(format!("Expected step parameters to end with ] {}", step));
                    }
                    let param_start = param_start.unwrap();
                    let name = &step[0..param_start];
                    let mut step_prototype = StepConfig {
                        kind: StepKind::from_str(name)?,
                        substeps: None,
                        min: None,
                        max: None,
                        niss: default_niss_type,
                        solution_count: default_solution_count,
                        quality: default_quality,
                        params: HashMap::new()
                    };
                    let params = (&step[(param_start + 1)..(step.len() - 1)]).split(";").collect_vec();
                    for param in params {
                        if param.contains("=") {
                            let parts = param.split("=").collect_vec();
                            if parts.len() != 2 {
                                return Err(format!("Invalid param format {}", param));
                            }
                            match parts[0] {
                                "limit" => step_prototype.solution_count = Some(usize::from_str(parts[1]).map_err(|x| format!("Unable to parse value '{}' for count. '{x}'", parts[1]))?),
                                "quality" => step_prototype.quality = Some(usize::from_str(parts[1]).map_err(|x| format!("Unable to parse value '{}' for quality. '{x}'", parts[1]))?),
                                "min" => step_prototype.min = Some(u8::from_str(parts[1]).map_err(|x| format!("Unable to parse value '{}' for min. '{x}'", parts[1]))?),
                                "max" => step_prototype.max = Some(u8::from_str(parts[1]).map_err(|x| format!("Unable to parse value '{}' for max. '{x}'", parts[1]))?),
                                "niss" => step_prototype.niss = Some(match parts[1] {
                                    "always" => NissType::During,
                                    "before" => NissType::Before,
                                    "none" => NissType::None,
                                    x => Err(format!("Invalid NISS type {x}. Expected one of 'always', 'before', 'none'"))?
                                }),
                                key => {
                                    step_prototype.params.insert(key.to_string(), parts[1].to_string());
                                }
                            }
                        } else {
                            step_prototype.substeps = Some(step_prototype.substeps.map_or(vec![param.to_string()], |mut v|{
                                v.push(param.to_string());
                                v
                            }));
                        }
                    }
                    Ok(step_prototype)
                }
            })
            .collect()
    }
}