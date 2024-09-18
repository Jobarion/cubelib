use std::collections::HashMap;
use std::str::FromStr;
use clap::Parser;
use regex::Regex;
use cubelib::defs::*;
use cubelib::steps::step::{StepConfig};

#[derive(Parser)]
#[command(name = "Cubelib")]
#[command(author = "Jonas Balsfulland <cubelib@joba.me>")]
#[command(version = "1.2")]
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
    #[arg(short = 'q', long = "quality", default_value_t = 100, help = "Influences the maximum number of solutions calculated per step. Set to 0 for infinite quality")]
    pub quality: usize,
    #[arg(long = "steps", short = 's', default_value = "EO > RZP > DR[triggers=R,RUR,RU'R,RU2R] > HTR > FIN", help = "List of steps to perform")]
    pub steps: String,
    pub scramble: String,
}

impl Cli {

    fn get_default_niss_type(&self) -> Option<NissSwitchType> {
        if self.niss {
            None //This means undefined, so steps can define their own niss type.
        } else {
            Some(NissSwitchType::Never)
        }
    }

    pub fn parse_step_configs(&self) -> Result<Vec<StepConfig>, String> {
        let step_name_regex = Regex::new("[A-Za-z0-9_-]").unwrap();
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
                        absolute_min: None,
                        absolute_max: None,
                        niss: default_niss_type,
                        step_limit: None,
                        quality: self.quality,
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
                        absolute_min: None,
                        absolute_max: None,
                        niss: default_niss_type,
                        step_limit: None,
                        quality: self.quality,
                        params: HashMap::new()
                    };
                    let params: Vec<&str> = (&step[(param_start + 1)..(step.len() - 1)]).split(";").collect();
                    for param in params {
                        if param.contains("=") {
                            let parts: Vec<&str> = param.split("=").collect();
                            if parts.len() != 2 {
                                return Err(format!("Invalid param format {}", param));
                            }
                            match parts[0] {
                                "limit" => step_prototype.step_limit = Some(usize::from_str(parts[1]).map_err(|x| format!("Unable to parse value '{}' for count. '{x}'", parts[1]))?),
                                key @ "min" | key @ "min-rel" => step_prototype.min = Some(u8::from_str(parts[1]).map_err(|x| format!("Unable to parse value '{}' for {key}. '{x}'", parts[1]))?),
                                key @ "max" | key @ "max-rel" => step_prototype.max = Some(u8::from_str(parts[1]).map_err(|x| format!("Unable to parse value '{}' for {key}. '{x}'", parts[1]))?),
                                "min-abs" => step_prototype.absolute_min = Some(u8::from_str(parts[1]).map_err(|x| format!("Unable to parse value '{}' for min-abs. '{x}'", parts[1]))?),
                                "max-abs" => step_prototype.absolute_max = Some(u8::from_str(parts[1]).map_err(|x| format!("Unable to parse value '{}' for max-abs. '{x}'", parts[1]))?),
                                "niss" => step_prototype.niss = Some(match parts[1] {
                                    "always" | "true" => NissSwitchType::Always,
                                    "before" => NissSwitchType::Before,
                                    "none" | "never" | "false" => NissSwitchType::Never,
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