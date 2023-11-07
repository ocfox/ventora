use std::error::Error;
use std::str::FromStr;
const AMDGPU_IDS: &str = std::include_str!("amdgpu.ids");

#[derive(Debug)]
pub struct DeviceInfo {
    pub device_id: u16,
    pub revision_id: u16,
    pub product_name: String,
}

impl FromStr for DeviceInfo {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(',');
        let device_id = parts.next().ok_or("Missing device_id")?.trim();
        let revision_id = parts.next().ok_or("Missing revision_id")?.trim();
        let product_name = parts.next().ok_or("Missing product_name")?.trim();

        let device_id = u16::from_str_radix(device_id, 16)?;
        let revision_id = u16::from_str_radix(revision_id, 16)?;

        Ok(DeviceInfo {
            device_id,
            revision_id,
            product_name: product_name.to_string(),
        })
    }
}

pub async fn read_amdgpu_ids() -> Result<Vec<DeviceInfo>, Box<dyn Error>> {
    let contents = AMDGPU_IDS;
    let mut device_infos = Vec::new();

    for line in contents.lines() {
        if !line.starts_with('#') {
            if let Ok(device_info) = line.parse() {
                device_infos.push(device_info);
            }
        }
    }

    Ok(device_infos)
}

pub async fn get_product_name(
    device_id: u16,
    revision_id: u16,
) -> Result<Option<String>, Box<dyn Error>> {
    let device_infos = read_amdgpu_ids().await?;

    for info in device_infos.iter() {
        if info.device_id == device_id && info.revision_id == revision_id {
            return Ok(Some(info.product_name.clone()));
        }
    }

    Ok(None)
}
