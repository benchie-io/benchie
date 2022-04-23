use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(long, short)]
    pub flag: bool,

    #[clap(required = true, multiple_values = true, allow_hyphen_values = true)]
    pub command: Vec<String>,
}

pub fn parse_arguments() -> Args {
    Args::parse()
}
