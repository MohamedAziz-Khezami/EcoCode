

use std::error::Error;
use std::fs::{File, read_dir};
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::os::unix::net::SocketAddr;
use regex::Regex;
use crate::sensor::RAPL_PATH;


pub fn get_energy(rapl_file: &mut BufReader<File>) -> Result<f64, Box<dyn Error>> { 
    // Parse the energy consumption from the file (in microjoules)
 

   rapl_file.seek(SeekFrom::Start(0))?;

    let mut buffer = String::new();
    rapl_file.read_line(&mut buffer)?;

    let energy_consumed = buffer.trim().parse::<f64>()?;

    // dbg!(energy_consumed);

    Ok(energy_consumed) //energy in microjoules
}//fix: Counter reset handle



// --- TODO: Make it modular ---
#[derive(Debug)]
pub struct RAPL_File {
    pub Socket: i16,
    pub Domaine: String, // cpu, dram, igpu
    pub File: BufReader<File>
    //max_range: for energy counter wrapping
}
#[derive(Debug)]
pub struct RAPL_Energy {
    pub Socket: i16,
    pub Domaine: String, // cpu, dram, igpu
    pub Energy: f64
    
}

#[derive(Debug)]
pub struct RAPL_Energy_Delta {
    pub Socket: i16,
    pub Domaine: String, // cpu, dram, igpu
    pub DeltaEnergy: f64
}

pub fn get_all_energies(rapl_readers: &mut Vec<RAPL_File>) -> Result<Vec<RAPL_Energy>, Box<dyn Error>> {
    let mut rapl_energies = Vec::new();
    for reader in rapl_readers {
        let energy = get_energy(&mut reader.File)?; //energy in microjoules
        let rapl_energy = RAPL_Energy {
            Socket: reader.Socket,
            Domaine: reader.Domaine.clone(),
            Energy: energy
        };
        rapl_energies.push(rapl_energy);
    }
    Ok(rapl_energies)
}



pub fn scan_rapl_files() -> Result<Vec<RAPL_File>, Box<dyn Error>>{
    let mut rapl_files = Vec::new();

    let re_socket = Regex::new(r"^intel-rapl:(\d+)$")?;
    let re_domain = Regex::new(r"^intel-rapl:\d+:\d+$")?;

    let entries = read_dir(RAPL_PATH)?;
    
    for entry in entries {
        //socket level lookout
        let entry = entry.unwrap().path();

        let dir_name = entry.file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| "Failed to convert directory name to string")?; //good one for using ?

        let socket: i16 = match re_socket
            .captures(dir_name)
            .and_then(|c| c.get(1))
        {
            Some(m) => m.as_str().parse()?,
            None => continue, // skip non-socket dirs
        };

        //domain level lookout
        for sub_entry in entry.read_dir()? {
            let mut ent  = sub_entry.unwrap().path();
            let entf = ent.file_name().unwrap().to_str().unwrap();
            if ent.is_dir() && re_domain.is_match(&entf) {

                let name = match entf  {
                    "intel-rapl:0:0" => "cpu".to_string(),
                    "intel-rapl:0:2" => "dram".to_string(),
                    "intel-rapl:0:1" => "igpu".to_string(),
                    _ => "unknown".to_string(),
                };

                ent.push("energy_uj");
                dbg!(&ent);
                let file = File::open(&ent)?;
                let file = BufReader::new(file);

                let file = RAPL_File {
                    Socket: socket,// FIXED: to get the socket in the future for clusters and multisocket servers
                    Domaine: name,
                    File: file,
                };

                rapl_files.push(file);
                
            }
        }
        
    }
    rapl_files.sort_by(|a, b| a.Domaine.cmp(&b.Domaine)); // FIXED: but this vector is unpredictable each time cpu mem , mem cpu ???

    Ok(rapl_files) //rapl_files sorted
}



pub fn delta_cpu_energy_per_pid_w(
    energies_1: &[RAPL_Energy],
    energies_2: &[RAPL_Energy],
    elapced_secs: f64,
    cpu_usage: f64,
) -> Vec<RAPL_Energy_Delta>  {
    //Check the lengths
    if energies_1.len() != energies_2.len() {
        panic!("energy vectors must have the same length");
    };
    let mut delta_energy = Vec::new();
    for i in 0..energies_1.len() {
        let energy_1 = energies_1[i].Energy;
        let energy_2 = energies_2[i].Energy;
        let delta_energy_per_pid = (((energy_2 - energy_1) / 1000.0 )/ elapced_secs) * (cpu_usage / 100.0); // (mJ -> J) / s = W

        delta_energy.push(RAPL_Energy_Delta {
            Socket: energies_1[i].Socket,
            Domaine: energies_1[i].Domaine.clone(),
            DeltaEnergy: delta_energy_per_pid,
        });
    }

    delta_energy
}


        // let cpu_energy_w = (delta_cpu_energy_mj / 1000.0) / elapsed_secs; // (mJ -> J) / s = W
        // let cpu_energy_per_pid = cpu_energy_w * (cpu_usage as f64 / 100.0); // Normalize to 0-1


// open files(keep open) -> get energy using tokio -> close files
