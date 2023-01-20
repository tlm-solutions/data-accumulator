extern crate clap;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "data-accumulator")]
#[clap(author = "contact@tlm.solutions")]
#[clap(version = "0.4.1")]
#[clap(about = "data collection server with authentication and statistics", long_about = None)]
pub struct Args {
    #[clap(short, long, default_value_t = String::from("127.0.0.1"))]
    pub host: String,

    #[clap(long, default_value_t = String::from("127.0.0.1"))]
    pub prometheus_host: String,

    #[clap(short, long, default_value_t = 8080)]
    pub port: u16,

    #[clap(long, default_value_t = 8081)]
    pub prometheus_port: u16,

    #[clap(short, long, action)]
    pub offline: bool,
}
