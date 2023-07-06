use std::{path::Path, str::FromStr};

use clap::{Parser, Subcommand, ValueEnum};
use serde::de::DeserializeOwned;

/// A CLI utility for working with .tot files.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The file to operate on.
    file: String,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Verify a .tot file.
    Check,
    /// Convert the input file to the given file type.
    To {
        #[command(flatten)]
        opts: ConvertOptions,
    },
    /// Convert the input file from the given file type.
    From {
        #[command(flatten)]
        opts: ConvertOptions,
    },
}

#[derive(clap::Args, Debug)]
struct ConvertOptions {
    /// The file type to work with.
    #[arg(value_enum)]
    file_type: FileType,
    /// The path where the converted file should be written.
    out_path: String,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum FileType {
    #[cfg(feature = "json")]
    Json,
    #[cfg(feature = "yaml")]
    Yaml,
    #[cfg(feature = "toml")]
    Toml,
}

pub fn run() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    log::debug!("Starting!");

    let args = Args::try_parse()?;

    log::debug!("{args:?}");

    match args.command {
        Command::Check => check(&args.file)?,
        Command::To { opts } => match opts.file_type {
            #[cfg(feature = "json")]
            FileType::Json => convert_to_json(&args.file, &opts.out_path)?,
            #[cfg(feature = "yaml")]
            FileType::Yaml => convert_to_yaml(&args.file, &opts.out_path)?,
            #[cfg(feature = "toml")]
            FileType::Toml => convert_to_toml(&args.file, &opts.out_path)?,
        },
        Command::From { opts } => match opts.file_type {
            #[cfg(feature = "json")]
            FileType::Json => convert_from_json(&args.file, &opts.out_path)?,
            #[cfg(feature = "yaml")]
            FileType::Yaml => convert_from_yaml(&args.file, &opts.out_path)?,
            #[cfg(feature = "toml")]
            FileType::Toml => convert_from_toml(&args.file, &opts.out_path)?,
        },
    }

    Ok(())
}

fn check(path: &String) -> anyhow::Result<()> {
    let contents = std::fs::read_to_string(path)?;

    // TODO stub

    Ok(())
}

fn convert_to_json(path: &String, output: &String) -> anyhow::Result<()> {
    let contents = std::fs::read_to_string(path)?;

    // TODO stub

    Ok(())
}

fn convert_to_yaml(path: &String, output: &String) -> anyhow::Result<()> {
    let contents = std::fs::read_to_string(path)?;

    // TODO stub

    Ok(())
}

fn convert_to_toml(path: &String, output: &String) -> anyhow::Result<()> {
    let contents = std::fs::read_to_string(path)?;

    // TODO stub

    Ok(())
}

fn convert_from_json(path: &String, output: &String) -> anyhow::Result<()> {
    let contents = std::fs::read_to_string(path)?;
    let value = serde_json::to_value(&contents)?;

    // TODO stub

    Ok(())
}

fn convert_from_yaml(path: &String, output: &String) -> anyhow::Result<()> {
    let contents = std::fs::read_to_string(path)?;
    let value = serde_yaml::to_value(&contents)?;

    // TODO stub

    Ok(())
}

fn convert_from_toml(path: &String, output: &String) -> anyhow::Result<()> {
    let contents = std::fs::read_to_string(path)?;
    let value = toml::Value::from_str(contents.as_str())?;

    // TODO stub

    Ok(())
}
