use crate::MCU;
use std::fs::{read_to_string, write};
use std::path::PathBuf;

#[derive(Debug)]
pub enum FirmwareStatus {
    Offline,
    Running,
}

pub fn check_mcu_exists(path: &str, mcu: &str) -> Result<String, String> {
    let path = PathBuf::from(path);
    for entry in path.read_dir().expect("read_dir failed") {
        if let Ok(entry) = entry {
            let path = entry.path();
            let path = path.to_str().unwrap();
            let mcu_file = read_to_string(format!("{path}/{MCU}")).unwrap();
            for line in mcu_file.lines() {
                if line == mcu {
                    return Ok(path.to_string());
                }
            }
        }
    }
    return Err("no mcu detected".to_string());
}

pub fn check_status(path: &str) -> Result<FirmwareStatus, String> {
    let status = read_to_string(path).unwrap();
    for line in status.lines() {
        match line {
            "offline" => {
                println!("Microprocessor is offline.");
                return Ok(FirmwareStatus::Offline);
            }
            "running" => {
                println!("Microprocessor is ruuning.");
                return Ok(FirmwareStatus::Running);
            }
            _ => return Err("Checking MCU status error".to_string()),
        }
    }
    Err("Checking MCU status error".to_string())
}

pub fn change_status(path: &str, new: FirmwareStatus) {
    match new {
        FirmwareStatus::Offline => {
            write(path, "stop").expect("Unable to write file");
            println!("Microprocessor is offline.");
        }
        FirmwareStatus::Running => {
            write(path, "start").expect("Unable to write file");
            println!("Microprocessor is running.");
        }
    }
}

pub fn read_info(path: &str) -> String {
    let file = read_to_string(path).unwrap();
    for line in file.lines() {
        return line.to_string();
    }
    "".to_string()
}
