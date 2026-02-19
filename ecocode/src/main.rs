use clap::Parser;
use nvml_wrapper::Nvml;
use sqlx::types::chrono::Utc;
use std::fs::File;
use std::io::BufReader;
use std::process::Command;
use std::time::{Duration, Instant};
use sysinfo::{Pid, ProcessesToUpdate, RefreshKind, System};
use tokio::time::sleep;
mod exporter;
mod sensor;
use sensor::RAPL_PATH;
use sensor::cpu::get_energy;
use sensor::gpu::DEFAULT_GPU_DEVICE_INDEX;
use sensor::gpu::{get_gpu_energy, get_gpu_energy_by_pid};

use exporter::csv::CsvExporter;
use exporter::json::JsonExporter;
use exporter::online::OnlineExporter;
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

    // --- Exporter setup ---
    let mut exporter: Box<dyn Exporter> = match args.output.as_str() {
        "terminal" => Box::new(TerminalExporter::new()),
        "csv" => Box::new(CsvExporter::new(args.file.unwrap())?),
        "json" => Box::new(JsonExporter::new(args.file.unwrap())?),
        "sqlite" => Box::new(SqliteExporter::new(args.file.unwrap()).await?),
        "online" => Box::new(OnlineExporter::new().await?), // Does't need an input, it will read the .env
        _ => Box::new(TerminalExporter::new()),
    };
    println!("Exporter type: {:?}", exporter.exporter_type());


    //TODO: before running the command run sudo chmod +r /sys/class/powercap/intel-rapl/intel-rapl:0/energy_uj
    


    // --- Spawn the target process ---
    let command = Command::new(&args.command[0])
        .args(&args.command[1..])
        .spawn()
        .expect("failed to execute process");

    let pid = Pid::from(command.id() as usize);

    // --- System setup ---
    let mut sys = System::new_with_specifics(RefreshKind::everything());
    let num_cores = sys.cpus().len();

    let mut cpu_usage;

    // --- NVML / GPU setup ---
    let nvml = Nvml::init()?;
    // Get the GPU device (default index 0)
    let device = nvml.device_by_index(DEFAULT_GPU_DEVICE_INDEX)?;
    // --- Measurement state ---
    let mut iteration = 0;

    // Initial timestamp in microseconds for NVML (0 targets all samples initially)
    let mut timestamp: u64 = 0;

    //sudo chmod +r /sys/class/powercap/intel-rapl/intel-rapl:0/energy_uj
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
        sleep(Duration::from_secs(interval)).await; // Sleep for the specified interval

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

        // --- CPU energy calculation ---
        let cpu_energy_2 = get_energy(&mut rapl_file).unwrap();
        let elapsed_secs = start_time.elapsed().as_secs_f64();

        // dbg!(elapsed_secs);

        let delta_cpu_energy_mj = (cpu_energy_2 - cpu_energy_1) / 1000.0; // µJ → mJ
        let cpu_energy_w = (delta_cpu_energy_mj / 1000.0) / elapsed_secs; // (mJ -> J) / s = W
        let cpu_energy_per_pid = cpu_energy_w * (cpu_usage as f64 / 100.0); // Normalize to 0-1

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
            pid.as_u32(),
            Utc::now().to_rfc3339(),
            cpu_usage as f64,
            cpu_energy_per_pid,
            gpu_util_pid,
            gpu_power_pid,
        );
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
