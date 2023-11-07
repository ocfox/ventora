use crate::device::Device;
use anyhow::Result;
use temp::Temp;

mod config;
mod device;
mod ids;
mod pwm;
mod temp;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    let cards = Device::get_all_cards().await?.unwrap();
    for card in cards {
        let temp = Temp::from_device(&card).await;
        let name = Device::get_gpu_name(&card).await?;
        if name != None {
            println!("{}", name.unwrap());
        }

        println!("{:#?}", &temp);
    }
    Ok(())
}
