use std::path::Path;

use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use serde_json::to_writer;

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Settings {
    pub logpath: String,
    pub incoming_position: (i32, i32),
    pub outgoing_position: (i32, i32),
}

impl Settings {
    pub fn new(p: &str) -> Result<Self, ConfigError> {
        let s = Config::builder().add_source(File::with_name(&p)).build()?;
        s.try_deserialize()
    }

    pub fn save(&self, p: &str) -> anyhow::Result<()> {
        let f = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(p)?;
        let buf = std::io::BufWriter::new(f);
        Ok(to_writer(buf, self)?)
    }
}
