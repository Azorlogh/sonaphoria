use std::{
	ffi::OsStr,
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

		println!("common");

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

		println!("main");

		let main = make_naga_module(&dir.join(&config.main))?;

		println!("buffers");

		let buffers = config
			.buffers
			.iter()
			.map(|p| make_naga_module(&dir.join(p)))
			.collect::<Result<_, _>>()?;

		Ok(Self {
			main,
			buffers,
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
