//! CPU energy monitoring module using Intel RAPL (Running Average Power Limit).
//!
//! This module provides functionality to read CPU energy consumption from the system's
//! Intel RAPL interface. Energy values are measured in microjoules (µJ).
//!
//! # Prerequisites
//!
//! The RAPL energy file must be readable. Grant read access with:
//! ```sh
//! sudo chmod +r /sys/class/powercap/intel-rapl/intel-rapl:0/energy_uj
//! ```

use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};

/// Reads the current cumulative energy consumption from the Intel RAPL interface.
///
/// This function re-reads the RAPL sysfs file by seeking back to the start,
/// making it efficient for repeated polling without reopening the file.
///
/// # Returns
///
/// The cumulative energy consumption in **microjoules (µJ)** since system boot.
///
/// # Errors
///
/// Returns an error if the file cannot be seeked, read, or parsed.
///
/// # Note
///
/// Energy values are cumulative — take two readings and compute the delta
/// to get the energy consumed during a specific interval.
pub fn get_energy(rapl_file: &mut BufReader<File>) -> Result<f64, Box<dyn Error>> {
    rapl_file.seek(SeekFrom::Start(0))?;

    let mut buffer = String::new();
    rapl_file.read_line(&mut buffer)?;

    let energy_consumed = buffer.trim().parse::<f64>()?;

    Ok(energy_consumed)
}
