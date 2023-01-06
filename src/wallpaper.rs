use std::{
	ffi::OsStr,
	path::{Path, PathBuf},
};

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

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
	pub signals: Vec<Signal>,
	pub main: PathBuf,
	#[serde(default)]
	pub buffers: Vec<PathBuf>,
}

#[derive(Clone)]
pub struct Wallpaper {
	pub config: Config,
	pub main: ShaderSource,
	pub buffers: Vec<ShaderSource>,
}

impl Wallpaper {
	pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
		let source = std::fs::read_to_string(&path)?;
		let config: Config = ron::from_str(&source)?;

		let dir = path.as_ref().parent().ok_or(anyhow!("invalid path"))?;

		Ok(Self {
			main: ShaderSource::load(dir.join(&config.main))?,
			buffers: config
				.buffers
				.iter()
				.map(|p| ShaderSource::load(dir.join(p)))
				.collect::<Result<_, _>>()?,
			config,
		})
	}
}

#[derive(Clone)]
pub enum ShaderSource {
	Wgsl(String),
	Glsl(String),
}

impl ShaderSource {
	pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
		let source = std::fs::read_to_string(&path)?;
		match path.as_ref().extension().and_then(OsStr::to_str) {
			Some("frag") => Ok(Self::Glsl(source)),
			Some("wgsl") => Ok(Self::Wgsl(source)),
			_ => Err(anyhow!("unsupported shader format")),
		}
	}

	pub fn get_wgpu_shader_source<'a>(&self) -> wgpu::ShaderSource {
		match self {
			ShaderSource::Wgsl(src) => wgpu::ShaderSource::Wgsl(src.into()),
			ShaderSource::Glsl(src) => wgpu::ShaderSource::Glsl {
				shader: src.into(),
				stage: naga::ShaderStage::Fragment,
				defines: Default::default(),
			},
		}
	}
}
