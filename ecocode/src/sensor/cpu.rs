//! CPU energy monitoring module using Intel RAPL (Running Average Power Limit).
//!
//! This module provides functionality to read CPU energy consumption from the system's
//! Intel RAPL interface. Energy values are measured in microjoules.

use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};

/// Refreshes and retrieves the current energy consumption from the Intel RAPL interface.
///
/// This function reads the energy consumption file from the Intel RAPL power monitoring
/// subsystem, which tracks CPU energy usage in microjoules. It requires elevated privileges
/// (sudo) to access the system file.
///
/// # Returns
///
/// `Result<usize, Box<dyn Error>>` - The energy consumption in microjoules, or an error
/// if the file could not be read or parsed.
///
/// # Errors
///
/// Returns an error if:
/// - The sudo command fails or returns non-zero status
/// - The file cannot be read due to permission issues
/// - The energy value cannot be parsed as a valid integer
///
/// # Example
///
/// ```no_run
/// use sensor::energy::refresh_energy;
///
/// let energy = refresh_energy()?;
/// println!("Energy consumed: {} microjoules", energy);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Note
///
/// - This function requires the program to be run with sudo privileges
/// - Energy values are cumulative and represent total consumption since system boot
/// - Call this function periodically to calculate energy consumed during a time period
pub fn get_energy(rapl_file: &mut BufReader<File>) -> Result<f64, Box<dyn Error>> { //fixme: give an array of files and run thread on them
    // Parse the energy consumption from the file (in microjoules)
    //sudo chmod +r /sys/class/powercap/intel-rapl/intel-rapl:0/energy_uj

   rapl_file.seek(SeekFrom::Start(0))?;

    let mut buffer = String::new();
    rapl_file.read_line(&mut buffer)?;

    let energy_consumed = buffer.trim().parse::<f64>()?;

    // dbg!(energy_consumed);

    Ok(energy_consumed) //energy in microjoules
}//fix: Counter reset handle


//TODO: Make it modular


// open files(keep open) -> get energy using tokio -> close files