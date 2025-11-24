use env_logger;
use log::{info, warn};
use std::path::PathBuf;

use serde::Deserialize;

use clap::{Parser, Subcommand, ValueEnum};

/// Doc comment
#[derive(Parser, Debug)]
#[command(version, about, bin_name = "cargo stow")]
struct Cli {
    /// release build
    //#[arg(short, long)]
    //release: bool,
    #[arg(long, value_enum, default_value = "docker")]
    backend: Backend,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Clone, ValueEnum)]
enum Backend {
    /// Now working on it
    Docker,
    // Not implemented yet
    //Youki,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Build the container image
    Build {
        /// Release build
        #[arg(short, long)]
        release: bool,
        /// Debug build
        #[arg(short, long)]
        debug: bool,

        #[arg(long, default_value = "local")]
        cache_mode: CacheMode,
    },
    /// Push to the registry (TBD)
    Push,
    /// Just render the Dockerfile and no build
    Dockerfile {
        #[arg(long, default_value = "Dockerfile")]
        output: PathBuf,
    },
}

#[derive(Debug, Clone, ValueEnum)]
enum CacheMode {
    Local,
    Gha,
}

#[derive(Deserialize, Debug)]
struct Config {
    //ip: String,
    //port: Option<u16>,
    package: Package,
}
#[derive(Deserialize, Debug)]
struct Package {
    name: String,
    metadata: Metadata,
}
#[derive(Deserialize, Debug)]
struct Metadata {
    container: Container,
}
#[derive(Deserialize, Debug)]
struct Container {
    target_image: String,
    base_image: String,
    build_deps: Vec<String>,
    runtime_deps: Vec<String>,
}

mod docker;

/**
 * Do:
 * 1. Run cargo build with given options
 * 2. Load container build recipe from container section in Cargo.toml
 * 3. Format temporary Dockerfile to kick build (in 'docker' feature)
 * 4. Run 'docker buildx build' with it
 * 5. Run
 **/
fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let mut args: Vec<String> = std::env::args().collect();
    // remove "sort" when invoked `cargo sort` sort is the first arg
    // https://github.com/rust-lang/cargo/issues/7653
    if args.len() > 1 && args[1] == "stow" {
        args.remove(1);
    } else {
        warn!("Looks like not invoked via cargo. Should better be called cargo stow [..]");
    }
    let cli = <Cli as clap::Parser>::parse_from(args);

    info!("Hello, cargo-stow🧺📦️!");
    let mut path = std::env::current_dir()?;
    path.push("Cargo.toml");

    let content = std::fs::read_to_string(path.as_path())?;
    let build_config: Config = toml::from_str(&content)?;
    let bin = build_config.package.name.clone();
    let build_config = build_config.package.metadata.container;
    info!("{:?}", build_config);
    let config = docker::ContainerBuildConfig {
        artifact: bin,
        target_image: build_config.target_image,
        base_image: build_config.base_image,
        build_deps: build_config.build_deps.join(" "),
        runtime_deps: build_config.runtime_deps.join(" "),
    };

    match cli.command {
        Commands::Build { cache_mode, .. } => {
            info!(
                "building {}: cache_mode={:?}",
                config.target_image, cache_mode
            );
            let mode = match cache_mode {
                CacheMode::Local => docker::CacheMode::Local,
                CacheMode::Gha => docker::CacheMode::Gha,
            };
            Ok(docker::build(&config, mode)?)
        }
        Commands::Push => {
            info!("pushing {}", config.target_image);
            Ok(docker::push(&config.target_image)?)
        }
        Commands::Dockerfile { output } => {
            info!("saving to {}", output.display());
            Ok(docker::dockerfile(&config, output.as_path())?)
        }
    }
}
