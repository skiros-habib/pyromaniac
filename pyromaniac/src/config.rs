use std::path::PathBuf;

use anyhow::Result;
use dotenvy::dotenv;
use std::sync::OnceLock;
use std::time::Duration;

#[derive(Debug)]
pub struct Config {
    pub resource_path: PathBuf,
    pub port: u16,
    pub runner_config: RunnerConfig,
}

#[derive(Debug)]
pub struct RunnerConfig {
    pub cpus: u32,
    pub memory: u32,
    pub compile_timeout: Duration,
    pub run_timeout: Duration,
}

static CONFIG: OnceLock<Config> = OnceLock::new();

impl Config {
    fn init_from_env() -> Result<Config> {
        //load dotenv file if it exists
        match dotenv() {
            Err(_) => tracing::info!("No .env file found, nothing to load"),
            Ok(_) => tracing::info!("Loaded config from .env file"),
        }

        let resource_path = dotenvy::var("RESOURCE_PATH")
            .map_err(Into::<anyhow::Error>::into) //error trait bullshit
            .unwrap_or_else(|_| {
                tracing::warn!("No resource path provided defaulting to ./resources");
                "./resources".to_owned()
            })
            .into();

        //port defaults to 3000 if not provided
        let port = dotenvy::var("PORT")
            .map_err(Into::<anyhow::Error>::into) //error trait bullshit
            .and_then(|p| p.parse().map_err(Into::into))
            .unwrap_or_else(|_| {
                tracing::warn!("No port provided, defaulting to 3000");
                3000
            });

        let c = Ok(Config {
            resource_path,
            port,
            runner_config: RunnerConfig {
                cpus: 1,
                memory: 1024,
                compile_timeout: Duration::from_secs(10),
                run_timeout: Duration::from_secs(15),
            },
        });

        tracing::info!("Loaded config from environment!");

        c
    }
}
pub fn get() -> &'static Config {
    CONFIG.get_or_init(|| Config::init_from_env().unwrap())
}
