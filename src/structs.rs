extern crate clap;
extern crate derive_builder;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "data-accumulator")]
#[clap(author = "dump@dvb.solutions")]
#[clap(version = "0.3.0")]
#[clap(about = "data collection server with authentication and statistics", long_about = None)]
pub struct Args {
    #[clap(short, long, default_value_t = String::from("127.0.0.1"))]
    pub host: String,

    #[clap(short, long, default_value_t = 8080)]
    pub port: u16,

    #[clap(short, long, action)]
    pub offline: bool,

    #[clap(short, long, action)]
    pub verbose: bool
}
