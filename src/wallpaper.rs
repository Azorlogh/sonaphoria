use std::{
	ffi::OsStr,
	iter::once,
	path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use naga_oil::compose::{
	ComposableModuleDescriptor, Composer, NagaModuleDescriptor, ShaderLanguage, ShaderType,
};
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
	pub includes: Vec<PathBuf>,
	#[serde(default)]
	pub buffers: Vec<PathBuf>,
}

#[derive(Clone)]
pub struct Wallpaper {
	pub config_path: PathBuf,
	pub config: Config,
	pub main: naga::Module,
	pub buffers: Vec<naga::Module>,
}

impl Wallpaper {
	pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
		let source = std::fs::read_to_string(&path)?;
		let config: Config = ron::from_str(&source)?;

		let dir = path.as_ref().parent().ok_or(anyhow!("invalid path"))?;

		let mut composer = Composer::default();

		for include in &config.includes {
			let path = dir.join(&include);
			let language = match &path.extension().and_then(OsStr::to_str) {
				Some("frag") | Some("glsl") => ShaderLanguage::Glsl,
				Some("wgsl") => ShaderLanguage::Wgsl,
				_ => panic!("unsupported shader format"),
			};
			composer.add_composable_module(ComposableModuleDescriptor {
				source: &std::fs::read_to_string(&path)?,
				file_path: &path.to_string_lossy(),
				language,
				..Default::default()
			})?;
		}

		let mut make_naga_module = |path: &Path| -> Result<naga::Module> {
			let shader_type = match &path.extension().and_then(OsStr::to_str) {
				Some("frag") | Some("glsl") => ShaderType::GlslFragment,
				Some("wgsl") => ShaderType::Wgsl,
				_ => panic!("unsupported shader format"),
			};
			Ok(composer.make_naga_module(NagaModuleDescriptor {
				source: &std::fs::read_to_string(&path)?,
				file_path: &path.to_string_lossy(),
				shader_type,
				..Default::default()
			})?)
		};

		let main = make_naga_module(&dir.join(&config.main))?;

		let buffers = config
			.buffers
			.iter()
			.map(|p| make_naga_module(&dir.join(p)))
			.collect::<Result<_, _>>()?;

		Ok(Self {
			config_path: path.as_ref().to_owned(),
			main,
			buffers,
			config,
		})
	}

	pub fn paths(&self) -> Vec<PathBuf> {
		let dir = self.config_path.parent().unwrap();
		once(self.config_path.clone())
			.chain([self.config.main.to_owned()].iter().map(|p| dir.join(p)))
			.collect()
	}
}
