use crate::utils::read_file;
use anyhow::{anyhow, Context, Result};
use std::collections::BTreeMap;
use std::path::PathBuf;

use dirs;

pub fn get_default_config_path() -> Option<PathBuf> {
    dirs::config_dir()
}

#[derive(Debug)]
pub enum Mode {
    Auto,
    Ladder,
    Trapezoidal,
}

#[derive(Debug)]
pub enum TempMode {
    Avg,
    Junction,
    Edge,
}

#[derive(Debug)]
pub struct Point {
    pub temp: usize,
    pub pwm: usize,
}

#[derive(Debug)]
pub struct Config {
    pub bus_id: String,
    pub mode: Mode,
    pub temp: TempMode,
    pub point: Vec<Point>,
}

fn parse_mode(value: &str) -> Mode {
    match value {
        "auto" => Mode::Auto,
        "ladder" => Mode::Ladder,
        "trapezoidal" => Mode::Trapezoidal,
        _ => Mode::Auto,
    }
}

fn parse_temp_mode(value: &str) -> TempMode {
    match value {
        "avg" => TempMode::Avg,
        "junction" => TempMode::Junction,
        "edge" => TempMode::Edge,
        _ => TempMode::Avg,
    }
}

fn parse_point(point: &toml::Value) -> Result<Point> {
    if let Some(arr) = point.as_array() {
        if arr.len() == 2 {
            let temp = arr[0].as_integer().ok_or(anyhow!("Invalid temp"))? as usize;
            let pwm = arr[1].as_integer().ok_or(anyhow!("Invalid pwm"))? as usize;

            Ok(Point { temp, pwm })
        } else {
            Err(anyhow!("Invalid point array length"))
        }
    } else {
        Err(anyhow!("Invalid point format"))
    }
}

impl Config {
    pub async fn from_path(path: PathBuf) -> Result<Vec<Config>> {
        let toml_str = read_file(path)
            .await
            .with_context(|| anyhow!("Failed to read file"))?;

        let configs: BTreeMap<String, BTreeMap<String, toml::Value>> =
            toml::from_str(&toml_str).with_context(|| anyhow!("Failed to parse TOML"))?;

        let mut result = Vec::new();

        let configs: BTreeMap<String, BTreeMap<String, toml::Value>> =
            toml::from_str(&toml_str).with_context(|| anyhow!("Failed to parse TOML"))?;

        for (bus_id, config_map) in configs {
            let mode = parse_mode(config_map["mode"].as_str().ok_or(anyhow!("Invalid mode"))?);
            let temp_mode = parse_temp_mode(
                config_map["temp_mode"]
                    .as_str()
                    .ok_or(anyhow!("Invalid temp_mode"))?,
            );

            let point = config_map["point"]
                .as_array()
                .ok_or(anyhow!("Invalid point"))?
                .iter()
                .map(|point| {
                    let temp = point[0].as_integer().ok_or(anyhow!("Invalid temp"))? as usize;
                    let pwm = point[1].as_integer().ok_or(anyhow!("Invalid pwm"))? as usize;
                    Ok(Point { temp, pwm })
                })
                .collect::<Result<Vec<Point>>>()?;

            result.push(Config {
                bus_id: bus_id.clone(),
                mode,
                temp: temp_mode,
                point,
            });
        }

        Ok(result)
    }
}
