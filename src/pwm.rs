use crate::device::Device;
pub struct pwm {
    enable: bool,
    pwm: usize,
    min: usize,
    max: usize,
}

impl pwm {
    pub async fn from_device(device: &Device) -> Self {
        todo!()
    }
}
