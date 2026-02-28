use clap::Parser;
use nvml_wrapper::Nvml;
use sqlx::types::chrono::Utc;
use std::process::Command;
use std::time::Duration;
use sysinfo::{Pid, ProcessesToUpdate, RefreshKind, System};
use tokio::time::{self, Instant, MissedTickBehavior};
mod exporter;
mod sensor;
use sensor::RAPL_PATH;

use sensor::gpu::DEFAULT_GPU_DEVICE_INDEX;
use sensor::gpu::{get_gpu_energy, get_gpu_energy_by_pid};
use sensor::rapl::scan_rapl_files;

use exporter::csv::CsvExporter;
use exporter::json::JsonExporter;
use exporter::local::SqliteExporter;
use exporter::online::OnlineExporter;
use exporter::terminal::TerminalExporter;
use exporter::{Exporter, Record};

use crate::sensor::rapl::{delta_cpu_energy_per_pid_w, get_all_energies};

// --- Command-line argument parsing ---
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {// TODO: Add run name 
    //project name
    #[arg(short, long, default_value = "ecocode")]
    project: String,

    /// Output format: "terminal" or "csv"
    #[arg(short, long, default_value = "terminal")]
    output: String,

    /// CSV output file path (required if output=csv, json)
    #[arg(short, long)]
    file: Option<String>,

    /// Measurement interval in seconds
    #[arg(short, long, default_value = "1")]
    interval: u64,
    /// Command to monitor (with its arguments)
    #[arg(trailing_var_arg = true, required = true)]
    command: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = Args::parse();

    let interval = args.interval;
    let output = args.output.to_lowercase();
    let project_name = args.project;




    // Before running the command run sudo chmod -R +r /sys/class/powercap/intel-rapl/
    //promp the user for their password to access
    let _access = Command::new("sudo")
        .arg("chmod")
        .arg("-R")
        .arg("+r")
        .arg(RAPL_PATH) // FIXED: will be changed for multi-socket support
        .status();

    // --- Exporter setup ---
    let mut exporter: Box<dyn Exporter> = match output.as_str() {
        "terminal" => Box::new(TerminalExporter::new()),
        "csv" => Box::new(CsvExporter::new(args.file.unwrap())?),
        "json" => Box::new(JsonExporter::new(args.file.unwrap())?),
        "local" => Box::new(SqliteExporter::new().await?),
        "online" => Box::new(OnlineExporter::new().await?), // Does't need an input, it will read the .env
        _ => Box::new(TerminalExporter::new()),
    };
    println!("Exporter type: {:?}", exporter.exporter_type());

    // Check if the project exists, if not it will create it and return the id
    let project_id = exporter.project_exists(&project_name).await?;

    // Create a new run for this execution and get the run_id
    let run_id = exporter.create_run(&format!("run_{}", Utc::now().timestamp()), project_id).await?;


    

    // Open sys/class/powercap dir and look for intel-rapl files and make a buffer for them
    let mut rapl_readers = scan_rapl_files()?; //sorted by domain
    

    // --- Spawn the target process ---
    let command = Command::new(&args.command[0])
        .args(&args.command[1..])
        .spawn()
        .expect("failed to execute process");

    // Get the PID of the spawned process    
    let pid = Pid::from(command.id() as usize);

    // --- System setup ---
    let mut sys = System::new_with_specifics(RefreshKind::everything());
    let num_cores = sys.cpus().len();
    let sys_memo = sys.total_memory();
    let mut cpu_usage;
    let mut mem_usage;

    // --- NVML / GPU setup ---
    let nvml = Nvml::init()?;
    // Get the GPU device (default index 0)
    let device = nvml.device_by_index(DEFAULT_GPU_DEVICE_INDEX)?;


    // --- Measurement state ---
    let mut iteration = 0;
    
    // Initial timestamp in microseconds for NVML (0 targets all samples initially)
    let mut timestamp: u64 = 0;

    // Take initial energy readings before the loop so that the second reading
    // of each iteration can be reused as the first reading of the next one.
    // for all the rapl domains (cpu, dram, igpu) and sockets (in case of multisocket servers)
    sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
    let mut cpu_energy_1 = get_all_energies(&mut rapl_readers)?; //energies of the cpu sockets


    let mut gpu_energy_1 = get_gpu_energy(&device)?;

    let period = Duration::from_secs(interval);
    let mut last_sample = Instant::now();
    let mut ticker = time::interval_at(last_sample + period, period);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

    // --- Main measurement loop ---
    loop {
        iteration += 1;

        // Wait for the next sample interval, accounting for any delays to maintain consistent timing
        ticker.tick().await;
        let now = Instant::now();
        let elapsed_secs = now.duration_since(last_sample).as_secs_f64();
        // dbg!(&elapsed_secs);
        last_sample = now;

        // Refresh process data
        sys.refresh_processes(ProcessesToUpdate::All, true);

        // Check if the process is still running
        cpu_usage = if sys.process(pid).is_none() {
            println!("Process {} finished", pid);
            break;
        } else {
            // cpu_usage() returns total % across all cores; divide by num_cores
            // to get a normalized 0–100% value for the whole system
            sys.process(pid).unwrap().cpu_usage() / num_cores as f32
        };
        // TODO: Add memory and Igpu handeling
        mem_usage = (sys.process(pid).unwrap().memory() as f64 / sys_memo as f64) * 100.0;

        // --- CPU energy calculation ---
        let cpu_energy_2 = get_all_energies(&mut rapl_readers)?;

        
        // Calculate delta energy per domain (cpu, dram, igpu) and convert to watts using the elapsed time and CPU usage
        let delta_cpu_energy_w = delta_cpu_energy_per_pid_w(
            &cpu_energy_1,
            &cpu_energy_2,
            elapsed_secs, // can pass the interval duration instead of elapsed_secs if we assume the loop runs perfectly on time, but using elapsed_secs is more accurate in case of any delays
            cpu_usage as f64,
        
        );
       
        // Map the delta energies to their respective domains (cpu, dram, igpu)
        let (cpu_w, mem_w, igpu_w) =
            delta_cpu_energy_w
                .iter()
                .fold((0.0_f64, 0.0_f64, 0.0_f64), |mut acc, d| {
                    match d.domain.as_str() { // FIX: This only do the last socket not accumulate all the cpu or ram from all the sockets
                        "cpu" => acc.0 += d.delta_energy,
                        "dram" => acc.1 += d.delta_energy,
                        "igpu" => acc.2 += d.delta_energy,
                        _ => {}
                    }
                    acc
                }); 
                // TODO: handle multiple sockets and show accumulated and each socket energy
                // If you eventually want to display both per-socket and total values (useful for multi-socket server monitoring):

                // Change the fold's output structure from a tuple 

                // (f64, f64, f64)
                //  to a HashMap<(i16, String), f64> keyed by 

                // (socket, domain)
                // .
                // After the fold, you can sum across sockets for the total, or inspect individual sockets.
                // Update the Record struct to optionally carry per-socket breakdowns.


        // --- GPU energy calculation ---
        let gpu_energy_2 = get_gpu_energy(&device)?;

        let (gpu_power_pid, gpu_util_pid, next_timestamp) = get_gpu_energy_by_pid(
            &device,
            pid.as_u32(),
            gpu_energy_1,
            gpu_energy_2,
            timestamp,
            elapsed_secs,
        );

        // Carry forward: the end-of-interval reading becomes the start of the next interval
        cpu_energy_1 = cpu_energy_2;
        gpu_energy_1 = gpu_energy_2;
        timestamp = next_timestamp;

        // --- Export the measurement record ---
        let record = Record::new(
            iteration,
            run_id, // run_id will be set by the exporter if needed
            pid.as_u32(),
            Utc::now().to_rfc3339(),
            cpu_usage as f64,
            cpu_w, // replaces cpu_energy_per_pid
            gpu_util_pid,
            gpu_power_pid,
            Some((mem_usage * 100.0).round() / 100.0), //temporary fix until i test on a another system with memory domaine in rapl
            Some(mem_w),
            None,
            Some(igpu_w), // FIXME:  mem usage and igpu usage null values if not available, dead 0
        );
        // record.mem_energy = mem_w;
        // record.igpu_energy = igpu_w;

        exporter.add_record(record).await?;
        exporter.export_line().await?;

        if cpu_usage <= 0.0 && gpu_util_pid <= 0.0 && gpu_power_pid <= 0.0 {
            println!("Process finished");
            break;
        }
    }

    // Export final results
    exporter.export().await?;

    Ok(())
}


