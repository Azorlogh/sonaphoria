use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use encase::ShaderType;
use render::Renderer;
use wallpaper::Wallpaper;

mod analysis;
mod consts;
mod render;
mod wallpaper;

#[derive(Parser)]
#[command(
	author = "Azorlogh",
	version = "0.1.0",
	about = "An shader wallpaper engine"
)]
pub struct Cli {
	path: PathBuf,
}

fn main() -> Result<()> {
	let cli = Cli::parse();

	let wallpaper = Wallpaper::new(cli.path)?;

	let (stream, signals) = analysis::run(&wallpaper.config.signals).unwrap();

	let renderer = futures::executor::block_on(Renderer::new(wallpaper.clone(), signals));

	renderer.run();

	println!("{:?}", cpal::traits::StreamTrait::pause(&stream));

	Ok(())
}

#[derive(Debug, Default, ShaderType)]
struct Globals {
	resolution: glam::Vec2,
	time: f32,
	frame: u32,
}
