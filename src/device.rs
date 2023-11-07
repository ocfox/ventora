use crate::ids::get_product_name;
use crate::utils::{get_bus_id, read_file, read_u16};
use std::error::Error;
use std::path::PathBuf;
use tokio::fs::read_dir;

static DRM_PATH: &str = "/sys/class/drm";

pub struct Device {
    pub path: PathBuf,
    pub bus_id: String,
}
impl Device {
    pub async fn get_all_cards() -> std::io::Result<Option<Vec<Device>>> {
        let mut cards: Vec<Device> = Vec::new();
        let mut dir = read_dir(DRM_PATH).await?;

        while let Some(entry) = dir.next_entry().await? {
            let name = entry.file_name();
            let Some(name) = name.to_str() else { continue };
            if !name.starts_with("card") {
                continue;
            }

            let mut path = entry.path();
            path.push("device");

            let Ok(uevent) = read_file(path.join("uevent")).await else {
                continue;
            };

            // AMD PCI_ID
            if uevent.contains("PCI_ID=1002") {
                cards.push(Device {
                    bus_id: get_bus_id(&path).await?.to_string(),
                    path,
                });
                continue;
            }
        }

        match cards.is_empty() {
            false => Ok(Some(cards)),
            true => Ok(None),
        }
    }

    pub async fn get_gpu_name(&self) -> Result<Option<String>, Box<dyn Error>> {
        let device_id = read_u16(self.path.join("device")).await?.unwrap();
        let revision_id = read_u16(self.path.join("revision")).await?.unwrap();

        match get_product_name(device_id, revision_id).await? {
            Some(product_name) => Ok(Some(product_name.to_string())),
            None => Ok(Some("Not found".to_string())),
        }
    }
}
