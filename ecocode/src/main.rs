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
use sensor::RAPL_PATH;
use sensor::cpu::get_energy;
use sensor::gpu::DEFAULT_GPU_DEVICE_INDEX;
use sensor::gpu::{get_gpu_energy, get_gpu_energy_by_pid};

use exporter::csv::CsvExporter;
use exporter::json::JsonExporter;
use exporter::prometheus::PrometheusExporter;
use exporter::sqlite::SqliteExporter;
use exporter::terminal::TerminalExporter;
use exporter::{Exporter, Record};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Output format: "terminal" or "csv"
    #[arg(short, long, default_value = "terminal")]
    output: String,

    /// CSV output file path (required if output=csv)
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

    // Create exporter based on output format
    let mut exporter: Box<dyn Exporter> = match args.output.as_str() {
        "terminal" => Box::new(TerminalExporter::new()),
        "csv" => Box::new(CsvExporter::new(args.file.unwrap())?),
        "json" => Box::new(JsonExporter::new(args.file.unwrap())?),
        "sqlite" => Box::new(SqliteExporter::new(args.file.unwrap())),
        "prometheus" => Box::new(PrometheusExporter::new()),
        _ => Box::new(TerminalExporter::new()),
    };

    println!("Exporter type: {:?}", exporter.exporter_type());

    let command = Command::new(&args.command[0])
        .args(&args.command[1..])
        .spawn()
        .expect("failed to execute process");

    let pid = Pid::from(command.id() as usize);

    let mut sys = System::new_with_specifics(RefreshKind::everything());
    let num_cores = sys.cpus().len();

    let mut cpu_usage;

    let nvml = Nvml::init().map_err(|e| {
        eprintln!("Error initializing NVML: {}", e);
        Box::new(e) as Box<dyn std::error::Error>
    })?;

    // Get the GPU device (default index 0)
    let device = nvml
        .device_by_index(DEFAULT_GPU_DEVICE_INDEX)
        .map_err(|e| {
            eprintln!("Error getting GPU device: {}", e);
            Box::new(e) as Box<dyn std::error::Error>
        })?;

    let mut iteration = 0;
    // Initial timestamp in microseconds for NVML (0 targets all samples initially)
    let mut timestamp: u64 = 0;

    //sudo chmod +r /sys/class/powercap/intel-rapl/intel-rapl:0/energy_uj
    let mut rapl_file = BufReader::new(File::open(RAPL_PATH)?);

    loop {
        // Check if the process is still running
        // Refresh process data
        sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);

        iteration += 1;

        let cpu_energy_1 = get_energy(&mut rapl_file)?;
        let gpu_energy_1 = get_gpu_energy(&device)?;

        let start_time = Instant::now();
        thread::sleep(Duration::from_secs(interval)); // Sleep for the specified interval

        // Refresh process data
        sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);

        cpu_usage = if sys.process(pid).is_none() {
            println!("Process {} finished", pid);
            break;
        } else {
            sys.process(pid).unwrap().cpu_usage() / num_cores as f32
        };

        let cpu_energy_2 = get_energy(&mut rapl_file).unwrap();
        let elapsed_secs = start_time.elapsed().as_secs_f64();

        // dbg!(elapsed_secs);

        let delta_cpu_energy_mj = (cpu_energy_2 - cpu_energy_1) / 1000.0; // µJ → mJ

        let cpu_energy_w = (delta_cpu_energy_mj / 1000.0) / elapsed_secs; // (mJ -> J) / s = W

        let cpu_energy_per_pid = cpu_energy_w * (cpu_usage as f64 / 100.0); // Normalize to 0-1

        let gpu_energy_2 = get_gpu_energy(&device)?;

        let (gpu_power_pid, gpu_util_pid, next_timestamp) = get_gpu_energy_by_pid(
            &device,
            pid.as_u32(),
            gpu_energy_1,
            gpu_energy_2,
            timestamp,
            elapsed_secs,
        );

        // Update timestamp for next iteration
        timestamp = next_timestamp;

        // Create and add record to exporter
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

        if cpu_usage <= 0.0 && gpu_util_pid <= 0.0 && gpu_power_pid <= 0.0 {
            println!("Process finished");
            break;
        }
    }

    // Export final results
    exporter.export()?;

    Ok(())
}
