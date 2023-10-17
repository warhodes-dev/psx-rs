use clap::{ValueEnum, Parser};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[clap(value_enum, short, long, default_value_t=LogLevel::Error)]
    pub log: LogLevel,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

pub struct Config {
    pub log_level: LogLevel
}

impl Config {
    pub fn parse_args() -> Config {
        let args = Args::parse();
        Config::from_args(args)
    }
    fn from_args(args: Args) -> Config {
        Config {
            log_level: args.log,
        }
    }
}

