use async_trait::async_trait;
use std::path::PathBuf;

use crate::{
    device::Device,
    utils::{
        check_running_root, get_hwmon_path, read_file_tirmmed, read_file_to_usize, write_file,
        PathProvider,
    },
};
use anyhow::{anyhow, Context, Ok, Result};

#[derive(Debug)]
pub struct Pwm {
    pub auto: bool,
    pub pwm: usize,
    pub min: usize,
    pub max: usize,
    pub path: PwmPath,
}

#[derive(Debug)]
pub struct PwmPath {
    pub auto: Option<PathBuf>,
    pub pwm: Option<PathBuf>,
    pub min: Option<PathBuf>,
    pub max: Option<PathBuf>,
}

#[async_trait]
pub trait Control {
    async fn enable_auto(&mut self) -> Result<()>;
    async fn enable_manual(&mut self) -> Result<()>;
    async fn set_pwm_percent(&mut self, percent: usize) -> Result<()>;
    fn get_percent(self) -> usize;
}

#[async_trait]
impl Control for Pwm {
    async fn enable_manual(&mut self) -> Result<()> {
        if !check_running_root() {
            return Err(anyhow!("Only run as root could change mode"));
        }
        write_file(self.path.auto.get_path(), "1".to_string()).await?;
        self.auto = false;
        Ok(())
    }

    async fn enable_auto(&mut self) -> Result<()> {
        if !check_running_root() {
            return Err(anyhow!("Only run as root could change mode"));
        }
        write_file(self.path.auto.get_path(), "2".to_string()).await?;
        self.auto = false;
        Ok(())
    }

    fn get_percent(self) -> usize {
        self.pwm / self.max
    }

    async fn set_pwm_percent(&mut self, percent: usize) -> Result<()> {
        self.enable_manual().await?;
        if percent > 100 {
            return Err(anyhow!("{} is not a valid percentage", percent));
        }
        if !check_running_root() {
            return Err(anyhow!("Only run as root could change mode"));
        }
        write_file(self.path.pwm.get_path(), percent.to_string()).await?;
        self.pwm = percent;
        Ok(())
    }
}

trait InitPath {
    fn init(&mut self, auto: PathBuf, pwm: PathBuf, min: PathBuf, max: PathBuf);
}

impl InitPath for PwmPath {
    fn init(&mut self, auto: PathBuf, pwm: PathBuf, min: PathBuf, max: PathBuf) {
        self.auto = Some(auto);
        self.pwm = Some(pwm);
        self.min = Some(min);
        self.max = Some(max);
    }
}

impl PwmPath {
    pub async fn new() -> Self {
        Self {
            auto: None,
            pwm: None,
            min: None,
            max: None,
        }
    }
}

impl Pwm {
    async fn from(auto: bool, pwm: usize, min: usize, max: usize, path: PwmPath) -> Self {
        Self {
            auto,
            pwm,
            min,
            max,
            path,
        }
    }

    pub async fn from_device(device: &Device) -> Result<Option<Self>> {
        let hwmon_path = get_hwmon_path(&device.path)
            .await?
            .context("Failed to get hwmon path")?;

        for i in 1..5 {
            if !hwmon_path.join(format!("pwm{}", i)).exists() {
                continue;
            }

            let mut pwm_path = PwmPath::new().await;

            pwm_path.init(
                hwmon_path.join(format!("pwm{}_enable", i)),
                hwmon_path.join(format!("pwm{}", i)),
                hwmon_path.join(format!("pwm{}_min", i)),
                hwmon_path.join(format!("pwm{}_max", i)),
            );

            let pwm = Pwm::from(
                read_file_tirmmed(pwm_path.auto.get_path()).await? != "1",
                read_file_to_usize(pwm_path.pwm.get_path()).await?,
                read_file_to_usize(pwm_path.min.get_path()).await?,
                read_file_to_usize(pwm_path.max.get_path()).await?,
                pwm_path,
            )
            .await;

            return Ok(Some(pwm));
        }

        Ok(None)
    }
}
