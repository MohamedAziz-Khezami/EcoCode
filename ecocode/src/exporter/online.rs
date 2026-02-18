// This will be the exporter for online platform EcoCode.com to save run informations online.
// It will push the data to TimeScaleDB to be stored on the cloud.

use crate::exporter::{Exporter, ExporterType, Record};

pub struct OnlineExporter {}
