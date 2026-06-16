use clap::ValueEnum;
use colored::Colorize;
use std::fmt;

#[derive(Debug, Clone, ValueEnum, Default)]
pub enum Mode {
    #[default]
    Manual,
    Semi,
    Auto,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mode::Manual => write!(f, "manual"),
            Mode::Semi => write!(f, "semi"),
            Mode::Auto => write!(f, "auto"),
        }
    }
}

pub fn success(msg: &str) {
    println!("{} {}", "✓".green().bold(), msg);
}

pub fn warn(msg: &str) {
    println!("{} {}", "⚠".yellow().bold(), msg);
}

#[allow(dead_code)]
pub fn error(msg: &str) {
    eprintln!("{} {}", "✗".red().bold(), msg);
}

pub fn info(msg: &str) {
    println!("{} {}", "→".blue().bold(), msg);
}
