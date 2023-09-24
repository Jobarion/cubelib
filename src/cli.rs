use clap::Parser;

#[derive(Parser)]
#[command(name = "Cubelib")]
#[command(author = "Jonas Balsfulland <cubelib@joba.me>")]
#[command(version = "1.0")]
pub struct Cli {
    #[arg(short, long, default_value_t = false, group = "log_level")]
    pub verbose: bool,
    #[arg(short, long, default_value_t = false, group = "log_level")]
    pub quiet: bool,
    #[arg(short = 'c', long = "compact", default_value_t = false)]
    pub short_solution: bool,
    pub scramble: String,
}
