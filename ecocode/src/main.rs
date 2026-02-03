use clap::Parser;
use std::env;
use std::io::Read;
use std::process::Command;
use std::result;

use sysinfo::{CpuRefreshKind, Pid, ProcessesToUpdate, RefreshKind, System};

use std::thread;
use std::time::Duration;

use std::fs;
use std::io;

mod sensor;


//todo: add modularity to each sensing part

// fn parse_string_to_number(s: &String) -> u64 {
//     let c: Vec<&str> = s.split_whitespace().collect(); // tied to the lifetime of s so can't be a static literal(live forever)
//     let usage = c
//         .get(8)
//         .unwrap_or_else(|| &"0")
//         .parse::<u64>()
//         .unwrap_or_default(); //the unwrap_or_else method in Rust takes a closure (a function-like object) as an argument, not a direct value.
//     usage //let add = |x, y| x + y;
//           //println!("{}", add(2, 3)); // Outputs: 5 in our case we have a function (closure) that takes nothing and always return a reference to 0
// } // or just used unwrap_or(&"0")

// #[derive(Parser, Debug)]
// #[command(version, about, long_about = None)]
// struct Args {
//     #[arg(short, long)]
//     cmd: Vec<String>,
// }

fn main() -> Result<(), io::Error> {
    const POWER_FILE_PATH: &str = "/sys/class/powercap/intel-rapl/intel-rapl:0/energy_uj"; // Path for energy usage in microjoules

    //TODO: Get the args of the process get the name then run the command normally

    // let args: Args = Args::parse();
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);

    let mut command = Command::new(&args[1])
        .args(&args[2..])
        .spawn()
        .expect("failed to execute process");

    println!("Process ID: {:?}", command.id()); // this will give me the PID directly

    let pid = Pid::from(command.id() as usize);

    //TODO: Get the usage of the process with the pid

    let mut sys = System::new_with_specifics(RefreshKind::everything());

    let num_cores = sys.cpus().len();
    println!("Number of cores: {}", num_cores);

    //TODO: Continuously monitor the process's CPU usage
    loop {
        // Refresh process data
        sys.refresh_processes(ProcessesToUpdate::All, true);

        // Check if the process is still running
        if let Some(process) = sys.process(pid) {
            // Print the CPU usage of the process
            if process.cpu_usage() > 0.0 {
                println!(
                    "CPU usage of process {}: {}%",
                    pid,
                    process.cpu_usage() / num_cores as f32
                );
            } else {
                // If CPU usage is 0%, the process may be done, break out of the loop
                println!("Process {} has finished or is idle.", pid);
                break;
            }
            // Wait for 1 second before updating the usage again
            thread::sleep(Duration::from_secs(1));
        } else {
            println!("Process with PID {} has terminated.", pid);
            break; // Exit the loop if the process is no longer running
        }
    }

    //TODO: Get the cpu power consumption at the end //fixme put it in the loop
    // Use sudo to read the power consumption file with elevated privileges 
    let output = Command::new("sudo") //todo: add chmod for file access
        .arg("cat")
        .arg(POWER_FILE_PATH)
        .output()
        .expect("Failed to execute sudo command");

    // Check if the sudo command succeeded
    if !output.status.success() {
        eprintln!("Failed to read the power consumption file with sudo.");
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Permission denied",
        ));
    }

    let content = String::from_utf8_lossy(&output.stdout);

    // Parse the energy consumption from the file (in microjoules)
    let energy_consumed: u64 = content.trim().parse().unwrap_or(0);

    // Print the energy consumption
    println!("CPU Energy Consumption: {} microjoules", energy_consumed);

    sensor::energy::refresh_energy();

    return Ok(());
}
