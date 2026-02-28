use crate::sensor::RAPL_PATH;
use regex::Regex;
use std::error::Error;
use std::fs::{File, read_dir};
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::Path;

// --- TODO: Make it modular ---
#[derive(Debug)]
pub struct RaplFile {
    pub socket: i16,
    pub domain: String, // cpu, dram, igpu
    pub file: BufReader<File>,
    pub max_energy: f64, // in microjoules, used to detect counter resets
}
#[derive(Debug)]
pub struct RaplEnergy {
    pub socket: i16,
    pub domain: String, // cpu, dram, igpu
    pub energy: f64,
    pub max_energy: f64, // in microjoules, used to detect counter resets
}

#[derive(Debug)]
pub struct RaplEnergyDelta {
    pub socket: i16,
    pub domain: String, // cpu, dram, igpu
    pub delta_energy: f64,
}

// Get energy in microjoules from a rapl file
pub fn get_energy(rapl_file: &mut BufReader<File>) -> Result<f64, Box<dyn Error>> {
    // Parse the energy consumption from the file (in microjoules)

    rapl_file.seek(SeekFrom::Start(0))?;

    let mut buffer = String::new();
    rapl_file.read_line(&mut buffer)?;

    let energy_consumed = buffer.trim().parse::<f64>()?;

    Ok(energy_consumed) //energy in microjoules
} //FIXME: Counter reset handle

fn get_max_energy(path: &Path) -> Result<f64, Box<dyn Error>> {
    Ok(std::fs::read_to_string(path)?.trim().parse::<f64>()?)
}

pub fn get_all_energies(
    rapl_readers: &mut Vec<RaplFile>,
) -> Result<Vec<RaplEnergy>, Box<dyn Error>> {
    let mut rapl_energies = Vec::new();
    for reader in rapl_readers {
        let energy = get_energy(&mut reader.file)?; //energy in microjoules
        let rapl_energy = RaplEnergy {
            socket: reader.socket,
            domain: reader.domain.clone(),
            energy: energy,
            max_energy: reader.max_energy,
            // TODO: use this to detect counter resets
        };
        rapl_energies.push(rapl_energy);
    }
    Ok(rapl_energies)
}

// TODO: Make it concurrant for faster processing
pub fn delta_cpu_energy_per_pid_w(
    energies_1: &[RaplEnergy],
    energies_2: &[RaplEnergy],
    elapced_secs: f64,
    cpu_usage: f64,
) -> Vec<RaplEnergyDelta> {
    //Check the lengths
    if energies_1.len() != energies_2.len() {
        panic!("energy vectors must have the same length");
    };
    let mut delta_energy = Vec::new();
    for i in 0..energies_1.len() {
        let energy_1 = energies_1[i].energy;
        let energy_2 = energies_2[i].energy;
        let max_energy = energies_1[i].max_energy; // in microjoules

        // Handle counter reset
        let energy_2 = if energy_2 < energy_1 {
            energy_2 + max_energy
        } else {
            energy_2
        };

        let delta_energy_per_pid =
            ((((energy_2 - energy_1) / 1_000_000.0) / elapced_secs) * (cpu_usage / 100.0)).max(0.0); // convert to joules, divide by time to get power in watts, then multiply by cpu usage percentage to get the power attributed to the process

        delta_energy.push(RaplEnergyDelta {
            socket: energies_1[i].socket,
            domain: energies_1[i].domain.clone(),
            delta_energy: delta_energy_per_pid,
        });
    }

    delta_energy
}

// TODO: Make it concurrant for faster search
pub fn scan_rapl_files() -> Result<Vec<RaplFile>, Box<dyn Error>> {
    let mut rapl_files = Vec::new();

    let re_socket = Regex::new(r"^intel-rapl:(\d+)$")?;
    let re_domain = Regex::new(r"^intel-rapl:\d+:\d+$")?;

    let entries = read_dir(RAPL_PATH)?;

    for entry in entries {
        //socket level lookout
        let entry = entry.unwrap().path();

        let dir_name = entry
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| "Failed to convert directory name to string")?; //good one for using ?

        let socket: i16 = match re_socket.captures(dir_name).and_then(|c| c.get(1)) {
            Some(m) => m.as_str().parse()?,
            None => continue, // skip non-socket dirs
        };

        //domain level lookout
        for sub_entry in entry.read_dir()? {
            let mut ent = sub_entry.unwrap().path();
            let name_path = ent.join("name");
            let raw_name =
                std::fs::read_to_string(name_path).unwrap_or_else(|_| "unknown".to_string());
            let entf = ent.file_name().unwrap().to_str().unwrap();
            if ent.is_dir() && re_domain.is_match(&entf) {
                let name = match raw_name.trim() {
                    // FIX: this only handle one socket machines
                    "core" => "cpu".to_string(),
                    "dram" => "dram".to_string(),
                    "uncore" => "igpu".to_string(),
                    other => other.to_string(),
                };
                let max_energy_path = ent.join("max_energy_range_uj");
                // dbg!(&ent);
                ent.push("energy_uj");
                // dbg!(&ent);
                let file = File::open(&ent)?;
                let file = BufReader::new(file);

                let file = RaplFile {
                    socket: socket, // FIXED: to get the socket in the future for clusters and multisocket servers
                    domain: name,
                    file: file,
                    max_energy: get_max_energy(&max_energy_path)?, // TODO: read the max energy from the corresponding max_energy_range_uj file to detect counter resets
                };
                // dbg!(&file);
                rapl_files.push(file);
            }
        }
    }
    rapl_files.sort_by(|a, b| a.domain.cmp(&b.domain)); // FIXED: but this vector is unpredictable each time cpu mem , mem cpu ???

    Ok(rapl_files) //rapl_files sorted
}

// open files(keep open) with buffer -> get energy using tokio -> close files
