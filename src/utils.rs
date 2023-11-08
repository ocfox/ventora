use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::fs;
use tokio::fs::read_dir;
use tokio::io;

pub async fn read_file(path: PathBuf) -> io::Result<String> {
    let buffer = tokio::fs::read_to_string(path).await?;
    Ok(buffer)
}

pub async fn read_file_tirmmed(path: PathBuf) -> io::Result<String> {
    let buffer = read_file(path).await?;
    Ok(buffer.trim().to_string())
}

pub async fn read_file_to_usize(path: PathBuf) -> Result<usize> {
    let num = read_file_tirmmed(path).await?;
    let res = num
        .parse::<usize>()
        .context(format!("Failed parse {} to usize", num))?;
    Ok(res)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[tokio::test]
    async fn test_read_u16() {
        let test_file_path = PathBuf::from("test_u16.txt");
        let test_file_content = "0x1234\n";
        tokio::fs::write(&test_file_path, test_file_content)
            .await
            .expect("Failed to create test file");

        let result = read_u16(test_file_path.clone()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(0x1234));

        if let Err(e) = tokio::fs::remove_file(test_file_path).await {
            eprintln!("Failed to clean up test file: {}", e);
        }
    }

    #[tokio::test]
    async fn test_read_u16_invalid() {
        let test_file_path = PathBuf::from("test_u16_invalid.txt");
        let test_file_content = "invalid\n";
        tokio::fs::write(&test_file_path, test_file_content)
            .await
            .expect("Failed to create test file");

        let result = read_u16(test_file_path.clone()).await;
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().kind(), io::ErrorKind::InvalidData);

        if let Err(e) = tokio::fs::remove_file(test_file_path).await {
            eprintln!("Failed to clean up test file: {}", e);
        }
    }
}
