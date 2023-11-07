use crate::device::Device;
use temp::Temp;
use tokio::io::Result;

mod config;
mod device;
mod ids;
mod temp;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    let cards = Device::get_all_cards().await?.unwrap();
    for card in cards {
        println!("{}\n{}\n", card.path.to_string_lossy(), card.bus_id);
        println!("{}", Device::get_gpu_name(&card).await.unwrap().unwrap());
        println!("{:?}", Temp::from_device(&card).await);
    }
    Ok(())
}
