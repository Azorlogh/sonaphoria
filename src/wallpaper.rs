use std::path::Path;

use anyhow::Result;
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
}

pub struct Wallpaper {
	pub config: Config,
	pub main: String,
}

impl Wallpaper {
	pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
		let source = std::fs::read_to_string(&path)?;
		let config: Config = ron::from_str(&source)?;

		Ok(Self {
			config,
			main: std::fs::read_to_string(path.as_ref().with_extension("wgsl"))?,
		})
	}
}
