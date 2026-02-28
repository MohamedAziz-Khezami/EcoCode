//! Prometheus exporter — exposes measurement metrics via an HTTP `/metrics` endpoint.
//!
//! Starts a background HTTP server on port 9091 using `warp`, serving metrics
//! in the Prometheus text exposition format.

use crate::exporter::{Exporter, ExporterType, Record};
use async_trait::async_trait;
use prometheus::{Encoder, Gauge, Registry, TextEncoder};
use std::error::Error;
use warp::Filter;

/// Exports records as Prometheus gauge metrics available at `http://localhost:9091/metrics`.
///
/// Exposed metrics:
/// - `ecocode_cpu_usage` — CPU usage percentage
/// - `ecocode_cpu_energy_watts` — CPU energy consumption in Watts
/// - `ecocode_gpu_usage` — GPU usage percentage
/// - `ecocode_gpu_energy_watts` — GPU energy consumption in Watts
pub struct PrometheusExporter {
    registry: Registry,
    cpu_usage: Gauge,
    cpu_energy: Gauge,
    gpu_usage: Gauge,
    gpu_energy: Gauge,
    mem_usage: Gauge,    // percentage (0-100) Dram
    mem_energy: Gauge,   // watts
    igpu_usage: Gauge,   //igpu
    igpu_energy: Gauge,
}

impl PrometheusExporter {
    /// Creates a new Prometheus exporter and spawns the HTTP server in the background.
    pub fn new() -> PrometheusExporter {
        let registry = Registry::new();

        let cpu_usage = Gauge::new("ecocode_cpu_usage", "CPU usage percentage").unwrap();
        let cpu_energy = Gauge::new(
            "ecocode_cpu_energy_watts",
            "CPU energy consumption in Watts",
        )
        .unwrap();
        let gpu_usage = Gauge::new("ecocode_gpu_usage", "GPU usage percentage").unwrap();
        let gpu_energy = Gauge::new(
            "ecocode_gpu_energy_watts",
            "GPU energy consumption in Watts",
        )
        .unwrap();
        let mem_usage = Gauge::new("ecocode_mem_usage", "Memory usage percentage").unwrap();
        let mem_energy = Gauge::new("ecocode_mem_energy_watts", "Memory energy consumption in Watts").unwrap();
        let igpu_usage = Gauge::new("ecocode_igpu_usage", "IGPU usage percentage").unwrap();
        let igpu_energy = Gauge::new("ecocode_igpu_energy_watts", "IGPU energy consumption in Watts").unwrap();

        registry.register(Box::new(cpu_usage.clone())).unwrap();
        registry.register(Box::new(cpu_energy.clone())).unwrap();
        registry.register(Box::new(gpu_usage.clone())).unwrap();
        registry.register(Box::new(gpu_energy.clone())).unwrap();
        registry.register(Box::new(mem_usage.clone())).unwrap();
        registry.register(Box::new(mem_energy.clone())).unwrap();
        registry.register(Box::new(igpu_usage.clone())).unwrap();
        registry.register(Box::new(igpu_energy.clone())).unwrap();

        let exporter = PrometheusExporter {
            registry,
            cpu_usage,
            cpu_energy,
            gpu_usage,
            gpu_energy,
            mem_usage,
            mem_energy,
            igpu_usage,
            igpu_energy,
        };

        // Spawn a warp HTTP server in the background to serve metrics
        let registry_clone = exporter.registry.clone();
        let metrics_route = warp::path("metrics")
            .map(move || encode_metrics(&registry_clone))
            .boxed();

        tokio::spawn(async move {
            println!("Prometheus metrics available at http://localhost:9091/metrics");
            warp::serve(metrics_route).run(([0, 0, 0, 0], 9091)).await;
        });

        exporter
    }
}

#[async_trait(?Send)]
impl Exporter for PrometheusExporter {
    fn exporter_type(&self) -> ExporterType {
        ExporterType::Prometheus
    }

    /// Updates the Prometheus gauge values with the latest measurement.
    async fn add_record(&mut self, record: Record) -> Result<(), Box<dyn Error>> {
        self.cpu_usage.set(record.cpu_usage);
        self.cpu_energy.set(record.cpu_energy);
        self.gpu_usage.set(record.gpu_usage);
        self.gpu_energy.set(record.gpu_energy);
        self.mem_usage.set(record.mem_usage);
        self.mem_energy.set(record.mem_energy);
        self.igpu_usage.set(record.igpu_usage);
        self.igpu_energy.set(record.igpu_energy);
        Ok(())
    }

    async fn export(&mut self) -> Result<(), Box<dyn Error>> {
        println!("\n[PROMETHEUS EXPORT]");
        println!("Prometheus server is running on http://localhost:9091/metrics");
        Ok(())
    }

    async fn export_line(&mut self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    async fn project_exists(
        &mut self,
        project_name: &str,
    ) -> Result<i64, Box<dyn std::error::Error>> {
        Ok(0)
    }
    async fn create_project(
        &mut self,
        project_name: &str,
    ) -> Result<i64, Box<dyn std::error::Error>> {
        Ok(0)
    }
    async fn create_run(
        &mut self,
        run_name: &str,
        project_id: i64,
    ) -> Result<i64, Box<dyn std::error::Error>> {
        Ok(0)
    }
}



fn encode_metrics(registry: &Registry) -> String {
    let mut buffer = Vec::new();
    let encoder = TextEncoder::new();
    let metric_families = registry.gather();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}