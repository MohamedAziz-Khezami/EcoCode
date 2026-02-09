use crate::exporter::{Exporter, ExporterType, Record};
use prometheus::{Encoder, Gauge, Registry, TextEncoder};
use std::error::Error;
use warp::Filter;

pub struct PrometheusExporter {
    registry: Registry,
    cpu_usage: Gauge,
    cpu_energy: Gauge,
    gpu_usage: Gauge,
    gpu_energy: Gauge,
}

impl PrometheusExporter {
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

        registry.register(Box::new(cpu_usage.clone())).unwrap();
        registry.register(Box::new(cpu_energy.clone())).unwrap();
        registry.register(Box::new(gpu_usage.clone())).unwrap();
        registry.register(Box::new(gpu_energy.clone())).unwrap();

        let exporter = PrometheusExporter {
            registry,
            cpu_usage,
            cpu_energy,
            gpu_usage,
            gpu_energy,
        };

        // Start warp server in a background thread
        let registry_clone = exporter.registry.clone();
        tokio::spawn(async move {
            let metrics_route = warp::path("metrics").map(move || {
                let mut buffer = Vec::new();
                let encoder = TextEncoder::new();
                let metric_families = registry_clone.gather();
                encoder.encode(&metric_families, &mut buffer).unwrap();
                String::from_utf8(buffer).unwrap()
            });

            println!("Prometheus metrics available at http://localhost:9091/metrics");
            warp::serve(metrics_route).run(([0, 0, 0, 0], 9091)).await;
        });

        exporter
    }
}

impl Exporter for PrometheusExporter {
    fn exporter_type(&self) -> ExporterType {
        ExporterType::Prometheus
    }

    fn add_record(&mut self, record: Record) -> Result<(), Box<dyn Error>> {
        self.cpu_usage.set(record.cpu_usage);
        self.cpu_energy.set(record.cpu_energy);
        self.gpu_usage.set(record.gpu_usage);
        self.gpu_energy.set(record.gpu_energy);
        Ok(())
    }

    fn export(&mut self) -> Result<(), Box<dyn Error>> {
        println!("\n[PROMETHEUS EXPORT]");
        println!("Prometheus server is running on http://localhost:9091/metrics");
        Ok(())
    }

    fn export_line(&mut self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}