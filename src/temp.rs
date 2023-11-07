use crate::device::Device;
use crate::utils::{get_hwmon_path, read_file};
use anyhow::Context;

#[derive(Debug)]
pub struct Temp {
    edge: Option<usize>,
    junction: Option<usize>,
    mem: Option<usize>,
    avg: Option<usize>,
}

async fn get_temp_file(num: usize) -> (String, String) {
    (format!("temp{}_label", num), format!("temp{}_input", num))
}

pub trait UpdateTemp {
    fn update_value(&mut self, key: &str, value: usize);
    fn calculate_avg(&mut self);
}

impl UpdateTemp for Temp {
    fn update_value(&mut self, key: &str, value: usize) {
        match key {
            "edge" => self.edge = Some(value),
            "junction" => self.junction = Some(value),
            "mem" => self.mem = Some(value),
            _ => {}
        }
        self.calculate_avg();
    }

    fn calculate_avg(&mut self) {
        let values = [&self.edge, &self.junction, &self.mem];
        let mut res = Vec::<usize>::new();

        for value in values {
            if let Some(v) = value {
                res.push(*v);
            }
        }

        self.avg = if res.is_empty() {
            None
        } else {
            Some(res.iter().sum::<usize>() / res.len())
        };
    }
}

impl Temp {
    pub fn new() -> Self {
        Temp {
            edge: None,
            junction: None,
            mem: None,
            avg: None,
        }
    }

    pub async fn from_device(device: &Device) -> anyhow::Result<Self> {
        let hwmon_path = get_hwmon_path(&device.path)
            .await?
            .context("Failed to get hwmon path")?;

        let mut temp = Temp::new();

        for i in 1.. {
            let (label, input) = get_temp_file(i).await;

            let label_path = hwmon_path.join(&label);
            let input_path = hwmon_path.join(&input);

            if label_path.exists() {
                let label = read_file(label_path).await?;
                let input = read_file(input_path).await?;

                temp.update_value(&label.trim(), input.trim().parse::<usize>()? / 1000);
            } else {
                break;
            }
        }

        Ok(temp)
    }
}
