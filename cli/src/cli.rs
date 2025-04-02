use clap::{Parser, Subcommand, ValueEnum};
use log::LevelFilter;
use serde::Deserialize;

#[derive(Parser)]
#[command(name = "Cubelib")]
#[command(author = "Jonas Balsfulland <cubelib@joba.me>")]
#[command(version = "1.2")]
pub struct Cli {
    #[arg(short, long = "log", help = "Log level")]
    pub log: Option<LogLevel>,
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Solve(SolveCommand),
    Scramble,
    Invert(InvertCommand),
}

#[derive(Parser)]
pub struct InvertCommand {
    #[arg(help = "Scramble to invert (use '-' to read from stdin)")]
    pub scramble: String,
}

#[derive(Parser)]
pub struct SolveCommand {
    #[arg(short, long = "format", help="Solution output format")]
    pub format: Option<SolutionFormat>,
    #[arg(short = 'a', long = "all", help = "Print solutions that would otherwise get filtered out. E.g. an EO ending in F'")]
    pub all_solutions: Option<bool>,
    #[arg(short = 'm', long = "min", help = "Minimum length of solutions")]
    pub min: Option<usize>,
    #[arg(short = 'M', long = "max", help = "Maximum length of solutions")]
    pub max: Option<usize>,
    #[arg(short = 'n', help = "The number of solutions returned. By default 1 unless this option or --max is set")]
    pub solution_count: Option<usize>,
    #[arg(short = 'q', long = "quality", help = "Influences the maximum number of solutions calculated per step. Set to 0 to find optimal solutions")]
    pub quality: Option<usize>,
    #[arg(long = "steps", short = 's', help = "List of steps to perform")]
    pub steps: Option<String>,
    #[arg(help = "Scramble to solve (use '-' to read from stdin)")]
    pub scramble: String,
    #[arg(long = "backend", help = "Solver backend to use")]
    pub backend: Option<SolverBackend>,
}

#[derive(ValueEnum, Clone, Default, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SolverBackend {
    IterStream,
    #[default]
    MultiPathChannel,
}

#[derive(ValueEnum, Clone, Default, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Error, // Unrecoverable error. Results cannot be trusted
    #[default]
    Warn, // Unexpected input, user-correctable
    Info, // User-meaningful message
    Debug, // Developer-meaningful message
    Trace // DFS step
}

impl LogLevel {
    pub fn to_level_filter(&self) -> LevelFilter {
        match self {
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Trace => LevelFilter::Trace,
        }
    }
}

#[derive(ValueEnum, Clone, Default, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SolutionFormat {
    #[default]
    Detailed,
    Compact,
    Plain
}
