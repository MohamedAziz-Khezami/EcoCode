
use std::fs::{DirEntry, read_dir};
use std::path::Path;
use std::vec;

use clap::{Error, error};



//TODO: make a scan that search for the energy resources CPU, DRAM, and IGPU.
fn list_energy_dir(){ 

    const GLOBAL_PATH: &'static str = "/sys/class/powercap/intel-rapl/";

    let dir_list: Vec<Result<DirEntry, std::io::Error>> =  read_dir(Path::new(GLOBAL_PATH)).unwrap().collect();
    
    for i in dir_list{
        let piece = match i {
            Ok(i) => i,
            Err(_) => panic!(),
        };
        println!("{:?}", piece);
    }
}


/// Refreshes the energy data from the system.
///
/// This function is responsible for fetching the latest energy data from the system.
/// It should be called periodically to ensure the energy data is up to date.
///
/// # Example
///
/// 
pub fn refresh_energy(){
    const POWER_PATH: &str = "/sys/class/powercap/intel-rapl/intel-rapl:0/energy_uj";  // Path for energy usage in microjoules


}