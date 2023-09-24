use clap::Parser;
use log::LevelFilter;

#[derive(Parser)]
#[command(name = "Cubelib")]
#[command(author = "Jonas Balsfulland <cubelib@joba.me>")]
#[command(version = "1.0")]
pub struct Cli {
    #[arg(short, long, default_value_t = false, group = "log_level", help = "Enables more detailed logging")]
    pub verbose: bool,
    #[arg(short, long, default_value_t = false, group = "log_level", help = "Prints nothing but the solutions")]
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

    pub scramble: String,
}
