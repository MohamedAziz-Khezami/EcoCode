use clap::Parser;
use nvml_wrapper::Nvml;
use std::process::Command;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;

use sysinfo::{Pid, ProcessesToUpdate, RefreshKind, System};

mod exporter;
mod sensor;
use sensor::energy::get_energy;
use sensor::gpu::DEFAULT_GPU_DEVICE_INDEX;
use sensor::gpu::{get_gpu_energy, get_gpu_energy_by_pid};

use exporter::terminal::TerminalExporter;
use exporter::csv::CsvExporter;
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = Args::parse();

    let interval = args.interval;

    // Create exporter based on output format
    let mut exporter: Box<dyn Exporter > = match args.output.as_str() {
        "terminal" => Box::new(TerminalExporter::new()),
        "csv" => Box::new(CsvExporter::new(args.file.unwrap())?),
        _ => Box::new(TerminalExporter::new()),
    };

    let command = Command::new(&args.command[0])
        .args(&args.command[1..])
        .spawn()
        .expect("failed to execute process");

    let pid = Pid::from(command.id() as usize);

    let mut sys = System::new_with_specifics(RefreshKind::everything());
    let num_cores = sys.cpus().len();

    // Refresh process data
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let mut cpu_usage = sys.process(pid).unwrap().cpu_usage() / num_cores as f32; //first read

    let nvml = match Nvml::init() {
        Ok(n) => n,
        Err(e) => {
            eprintln!("Error initializing NVML: {}", e);
            return Err(Box::new(e));
        }
    };

    // Get the GPU device (default index 0)
    let device = match nvml.device_by_index(DEFAULT_GPU_DEVICE_INDEX) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error getting GPU device: {}", e);
            return Err(Box::new(e));
        }
    };

    let mut iteration = 0;

    loop {
        // Check if the process is still running

        if cpu_usage <= 0.0 {
            // If CPU usage is 0%, the process may be done, break out of the loop
            println!("Process {} has finished or is idle.", pid);
            break;
        }

        iteration += 1;

        let cpu_energy_1 = get_energy().unwrap();
        let gpu_energy_1 = get_gpu_energy(&device).unwrap();

        thread::sleep(Duration::from_secs(interval)); // Sleep for the specified interval

        // Refresh process data
        sys.refresh_processes(ProcessesToUpdate::All, true);

        cpu_usage = sys.process(pid).unwrap().cpu_usage() / num_cores as f32;

        let cpu_energy_2 = get_energy().unwrap();

        let delta_cpu_energy_mj = (cpu_energy_2 - cpu_energy_1) / 1000.0; // µJ → mJ
        let cpu_energy_w = delta_cpu_energy_mj / 1000.0 / interval as f64; // mJ → J, then J/s = W

        let cpu_energy_per_pid = cpu_energy_w * (cpu_usage as f64 / 100.0); // Normalize to 0-1

        let gpu_energy_2 = get_gpu_energy(&device).unwrap();

        let energy_consumed_by_pid_watt =
            get_gpu_energy_by_pid(&device, pid.as_u32(), gpu_energy_1, gpu_energy_2, interval);

        // Get current timestamp
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_millis() as u64;

        // Create and add record to exporter
        let record = Record::new(
            iteration,
            pid.as_u32(),
            timestamp,
            cpu_usage as f64,
            energy_consumed_by_pid_watt.1 * 100.0, // GPU usage percentage - can be calculated from GPU metrics
            cpu_energy_per_pid,
            energy_consumed_by_pid_watt.0,
        );
        exporter.add_record(record)?;
        exporter.export_line()?;
    }

    // Export final results
    exporter.export()?;

    return Ok(());
}
