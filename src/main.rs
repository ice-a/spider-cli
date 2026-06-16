mod cli;
mod core;

mod crypto;
mod calc;
mod har;
mod proxy;
mod js;
mod crawl;
mod export;
mod render;
mod app_reverse;
mod tools;
mod config;
mod mcp;
mod ai;
pub mod ai_config;
pub mod frida;

use clap::Parser;
use cli::Cli;
use colored::Colorize;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "reptool=info".into()),
        )
        .init();

    let cli = Cli::parse();

    if let Err(e) = cli.execute() {
        eprintln!("{} {}", "error:".red().bold(), e);
        std::process::exit(1);
    }
}
