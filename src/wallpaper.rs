use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::analysis::Smoothing;

#[derive(Clone, Serialize, Deserialize)]
pub enum Signal {
	Beat,
	BandEnergy {
		low: f32,
		high: f32,
	},
	Integrated(Box<Signal>),
	Smooth {
		attack: Smoothing,
		release: Smoothing,
		inner: Box<Signal>,
	},
}

#[derive(Serialize, Deserialize)]
pub struct Config {
	pub signals: Vec<Signal>,
	pub main: PathBuf,
	#[serde(default)]
	pub buffers: Vec<PathBuf>,
}

pub struct Wallpaper {
	pub config: Config,
	pub main: String,
	pub buffers: Vec<String>,
}

impl Wallpaper {
	pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
		let source = std::fs::read_to_string(&path)?;
		let config: Config = ron::from_str(&source)?;

		let dir = path.as_ref().parent().ok_or(anyhow!("invalid path"))?;

		Ok(Self {
			main: std::fs::read_to_string(dir.join(&config.main))?,
			buffers: config
				.buffers
				.iter()
				.map(|p| std::fs::read_to_string(dir.join(p)))
				.collect::<Result<_, _>>()?,
			config,
		})
	}
}
