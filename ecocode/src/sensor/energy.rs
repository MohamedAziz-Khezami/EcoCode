//! CPU energy monitoring module using Intel RAPL (Running Average Power Limit).
//!
//! This module provides functionality to read CPU energy consumption from the system's
//! Intel RAPL interface. Energy values are measured in microjoules.

use std::process::Command;
use std::{error::Error, io};

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
pub fn get_energy() -> Result<f64, Box<dyn Error>> {
    const RAPL_PATH: &str = "/sys/class/powercap/intel-rapl/intel-rapl:0/energy_uj";

    // Use sudo to read the energy file with elevated privileges
    let output = Command::new("sudo")
        .arg("cat")
        .arg(RAPL_PATH)
        .output()
        .expect("Failed to execute sudo command");

    // Check if the command succeeded
    if !output.status.success() {
        eprintln!("Failed to read the energy file with sudo.");
        return Err(Box::new(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Permission denied when reading RAPL energy file",
        )));
    }

    let content = String::from_utf8_lossy(&output.stdout);

    // Parse the energy consumption from the file (in microjoules)
    let energy_consumed: f64 = content.trim().parse()?;

    Ok(energy_consumed) //energy in microjoules
}
