use std::path::PathBuf;
use tokio::fs;
use tokio::fs::read_dir;
use tokio::io;

pub async fn read_file(path: PathBuf) -> io::Result<String> {
    let buffer = tokio::fs::read_to_string(path).await?;
    Ok(buffer)
}

pub async fn get_bus_id(path: &PathBuf) -> io::Result<String> {
    let real_path = fs::read_link(path).await?;
    let bus_id = real_path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid file name"))?
        .to_str()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid Unicode string"))?;

    Ok(bus_id.to_string())
}

pub async fn get_hwmon_path(path: &PathBuf) -> io::Result<Option<PathBuf>> {
    let mut dir = read_dir(path.join("hwmon")).await?;

    while let Some(entry) = dir.next_entry().await? {
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if !name.starts_with("hwmon") {
            continue;
        }
        return Ok(Some(entry.path()));
    }

    Ok(None)
}

pub async fn read_u16(file_path: PathBuf) -> std::io::Result<Option<u16>> {
    let content = read_file(file_path).await?;
    let str = content.trim();

    match u16::from_str_radix(&str[2..], 16) {
        Ok(value) => Ok(Some(value)),
        Err(parse_error) => Err(io::Error::new(io::ErrorKind::InvalidData, parse_error)),
    }
}
