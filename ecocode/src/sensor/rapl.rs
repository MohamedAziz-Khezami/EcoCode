use crate::sensor::RAPL_PATH;
use regex::Regex;
use std::error::Error;
use std::fs::{File, read_dir};
use std::io::{BufRead, BufReader, Seek, SeekFrom};



pub fn get_energy(rapl_file: &mut BufReader<File>) -> Result<f64, Box<dyn Error>> {
    // Parse the energy consumption from the file (in microjoules)

    rapl_file.seek(SeekFrom::Start(0))?;

    let mut buffer = String::new();
    rapl_file.read_line(&mut buffer)?;

    let energy_consumed = buffer.trim().parse::<f64>()?;

    // dbg!(energy_consumed);

    Ok(energy_consumed) //energy in microjoules
} //FIXME: Counter reset handle


// --- TODO: Make it modular ---
#[derive(Debug)]
pub struct RaplFile {
    pub socket: i16,
    pub domain: String,       // cpu, dram, igpu
    pub file: BufReader<File>, //max_range: for energy counter wrapping
}
#[derive(Debug)]
pub struct RaplEnergy {
    pub socket: i16,
    pub domain: String, // cpu, dram, igpu
    pub energy: f64,
}

#[derive(Debug)]
pub struct RaplEnergyDelta {
    pub socket: i16,
    pub domain: String, // cpu, dram, igpu
    pub delta_energy: f64,
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
        };
        rapl_energies.push(rapl_energy);
    }
    Ok(rapl_energies)
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
            let entf = ent.file_name().unwrap().to_str().unwrap();
            if ent.is_dir() && re_domain.is_match(&entf) {
                let name = match entf {
                    "intel-rapl:0:0" => "cpu".to_string(),
                    "intel-rapl:0:2" => "dram".to_string(),
                    "intel-rapl:0:1" => "igpu".to_string(),
                    _ => "unknown".to_string(),
                };

                ent.push("energy_uj");
                dbg!(&ent);
                let file = File::open(&ent)?;
                let file = BufReader::new(file);

                let file = RaplFile {
                    socket: socket, // FIXED: to get the socket in the future for clusters and multisocket servers
                    domain: name,
                    file: file,
                };

                rapl_files.push(file);
            }
        }
    }
    rapl_files.sort_by(|a, b| a.domain.cmp(&b.domain)); // FIXED: but this vector is unpredictable each time cpu mem , mem cpu ???

    Ok(rapl_files) //rapl_files sorted
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
        let delta_energy_per_pid =
            (((energy_2 - energy_1) / 1000.0) / elapced_secs) * (cpu_usage / 100.0); // (mJ -> J) / s = W

        delta_energy.push(RaplEnergyDelta {
            socket: energies_1[i].socket,
            domain: energies_1[i].domain.clone(),
            delta_energy: delta_energy_per_pid,
        });
    }

    delta_energy
}


// open files(keep open) -> get energy using tokio -> close files
