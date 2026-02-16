//! EcoCode — a tool to measure per-process CPU and GPU energy consumption.
//!
//! Spawns a user-specified command, then periodically measures its CPU and GPU
//! energy consumption until the process terminates. Results are exported via
//! a configurable backend (terminal, CSV, JSON, SQLite, or Prometheus).

use clap::Parser;
use nvml_wrapper::Nvml;
use std::fs::File;
use std::io::BufReader;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant, SystemTime};
use sysinfo::{Pid, ProcessesToUpdate, RefreshKind, System};

mod exporter;
mod sensor;

use exporter::csv::CsvExporter;
use exporter::json::JsonExporter;
use exporter::prometheus::PrometheusExporter;
use exporter::sqlite::SqliteExporter;
use exporter::terminal::TerminalExporter;
use exporter::{Exporter, Record};
use sensor::RAPL_PATH;
use sensor::cpu::get_energy;
use sensor::gpu::DEFAULT_GPU_DEVICE_INDEX;
use sensor::gpu::{get_gpu_energy, get_gpu_energy_by_pid};

/// Command-line arguments for EcoCode.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Output format: "terminal", "csv", "json", "sqlite", or "prometheus"
    #[arg(short, long, default_value = "terminal")]
    output: String,

    /// Output file path (required for csv, json, and sqlite formats)
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

    // --- Exporter setup ---
    let mut exporter: Box<dyn Exporter> = match args.output.as_str() {
        "terminal" => Box::new(TerminalExporter::new()),
        "csv" => Box::new(CsvExporter::new(args.file.unwrap())?),
        "json" => Box::new(JsonExporter::new(args.file.unwrap())?),
        "sqlite" => Box::new(SqliteExporter::new(args.file.unwrap())),
        "prometheus" => Box::new(PrometheusExporter::new()),
        _ => Box::new(TerminalExporter::new()),
    };
    println!("Exporter type: {:?}", exporter.exporter_type());

    // --- Spawn the target process ---
    let command = Command::new(&args.command[0])
        .args(&args.command[1..])
        .spawn()
        .expect("failed to execute process");
    let pid = Pid::from(command.id() as usize);

    // --- System info setup ---
    let mut sys = System::new_with_specifics(RefreshKind::everything());
    let num_cores = sys.cpus().len();
    let mut cpu_usage;

    // --- NVML / GPU setup ---
    let nvml = Nvml::init().map_err(|e| {
        eprintln!("Error initializing NVML: {}", e);
        Box::new(e) as Box<dyn std::error::Error>
    })?;
    let device = nvml
        .device_by_index(DEFAULT_GPU_DEVICE_INDEX)
        .map_err(|e| {
            eprintln!("Error getting GPU device: {}", e);
            Box::new(e) as Box<dyn std::error::Error>
        })?;

    // --- Measurement state ---
    let mut iteration = 0;
    // NVML sample timestamp in µs — 0 fetches all buffered samples on the first call
    let mut timestamp: u64 = 0;

    // Open the RAPL energy file once and reuse across iterations
    let mut rapl_file = BufReader::new(File::open(RAPL_PATH)?);

    // Take initial energy readings before the loop so that the second reading
    // of each iteration can be reused as the first reading of the next one.
    sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
    let mut cpu_energy_1 = get_energy(&mut rapl_file)?;
    let mut gpu_energy_1 = get_gpu_energy(&device)?;

    // --- Main measurement loop ---
    loop {
        iteration += 1;

        let start_time = Instant::now();
        thread::sleep(Duration::from_secs(interval));

        // Refresh process data after the sleep interval
        sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);

        // Check if the process is still running
        cpu_usage = if sys.process(pid).is_none() {
            println!("Process {} finished", pid);
            break;
        } else {
            // cpu_usage() returns total % across all cores; divide by num_cores
            // to get a normalized 0–100% value for the whole system
            sys.process(pid).unwrap().cpu_usage() / num_cores as f32
        };

        // --- CPU energy calculation ---
        let cpu_energy_2 = get_energy(&mut rapl_file).unwrap();
        let elapsed_secs = start_time.elapsed().as_secs_f64();

        let delta_cpu_energy_mj = (cpu_energy_2 - cpu_energy_1) / 1000.0; // µJ → mJ
        let cpu_energy_w = (delta_cpu_energy_mj / 1000.0) / elapsed_secs; // mJ → J, / s = W
        let cpu_energy_per_pid = cpu_energy_w * (cpu_usage as f64 / 100.0); // Scale by CPU share

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
        println!(
            "{:?}", SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_millis()
        );
        // --- Export the measurement record ---
        let record = Record::new(
            iteration,
            pid.as_u32(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_millis() as i64,
            cpu_usage as f64,
            cpu_energy_per_pid,
            gpu_util_pid,
            gpu_power_pid,
        );
        exporter.add_record(record)?;
        exporter.export_line()?;

        // Exit if the process has no activity on either CPU or GPU
        if cpu_usage <= 0.0 && gpu_util_pid <= 0.0 && gpu_power_pid <= 0.0 {
            println!("Process finished");
            break;
        }
    }

    // Export final aggregated results
    exporter.export()?;

    Ok(())
}
