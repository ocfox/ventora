use crate::device::Device;
use crate::utils::{get_hwmon_path, read_file};

#[derive(Debug)]
pub struct Temp {
    edge: Option<usize>,
    junction: Option<usize>,
    mem: Option<usize>,
}

async fn get_temp_file(num: usize) -> (String, String) {
    (format!("temp{}_label", num), format!("temp{}_input", num))
}

trait UpdateTemp {
    fn update_value(&mut self, key: &str, value: usize);
}

impl UpdateTemp for Temp {
    fn update_value(&mut self, key: &str, value: usize) {
        match key {
            "edge" => self.edge = Some(value),
            "junction" => self.junction = Some(value),
            "mem" => self.mem = Some(value),
            _ => {}
        }
    }
}

impl Temp {
    pub fn new() -> Self {
        Temp {
            edge: None,
            junction: None,
            mem: None,
        }
    }

    pub async fn from_device(device: &Device) -> Self {
        let hwmon_path = get_hwmon_path(&device.path).await.unwrap().unwrap();
        let mut temp = Temp::new();

        for i in 1.. {
            let (label, input) = get_temp_file(i).await;

            let label_path = hwmon_path.join(&label);
            let input_path = hwmon_path.join(&input);

            if label_path.exists() {
                let label = read_file(label_path).await.unwrap();
                let input = read_file(input_path).await.unwrap();
                temp.update_value(&label.trim(), input.trim().parse::<usize>().unwrap() / 1000);
            } else {
                break;
            }
        }

        temp
    }
}
