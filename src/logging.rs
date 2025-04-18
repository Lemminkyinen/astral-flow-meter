use crate::AmperageData;
use std::{fs, io::Write, path::PathBuf};

pub(crate) fn log_amp_data(
    log_path: &PathBuf,
    amp_data: &AmperageData,
) -> Result<(), anyhow::Error> {
    if log_path.exists() && fs::metadata(log_path).map(|m| m.len() > 0).unwrap_or(false) {
        // File exists and has content, append data in CSV format
        let csv_line = format!(
            "{},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2}\n",
            amp_data.timestamp,
            amp_data.pin_value_buffer[0],
            amp_data.pin_value_buffer[1],
            amp_data.pin_value_buffer[2],
            amp_data.pin_value_buffer[3],
            amp_data.pin_value_buffer[4],
            amp_data.pin_value_buffer[5]
        );

        Ok(fs::OpenOptions::new()
            .append(true)
            .open(log_path)
            .and_then(|mut file| file.write_all(csv_line.as_bytes()))?)
    } else {
        // File doesn't exist or is empty, create it with headers
        let header = "timestamp,pin1,pin2,pin3,pin4,pin5,pin6\n";
        let csv_line = format!(
            "{},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2}\n",
            amp_data.timestamp,
            amp_data.pin_value_buffer[0],
            amp_data.pin_value_buffer[1],
            amp_data.pin_value_buffer[2],
            amp_data.pin_value_buffer[3],
            amp_data.pin_value_buffer[4],
            amp_data.pin_value_buffer[5]
        );

        let content = format!("{}{}", header, csv_line);
        Ok(fs::write(log_path, content)?)
    }
}
