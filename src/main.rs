use std::path::PathBuf;

use analysis::AnalyzerController;
use anyhow::Result;
use clap::Parser;
use engine::{Engine, EngineEvent};
use wallpaper::Wallpaper;

mod analysis;
mod consts;
mod engine;
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
	#[arg(short, long)]
	watch: bool,
}

fn main() -> Result<()> {
	let cli = Cli::parse();

	let engine = futures::executor::block_on(Engine::new());
	let event_proxy = engine.proxy();
	let wallpaper = Wallpaper::new(&cli.path)?;
	let analyzer = AnalyzerController::start(&wallpaper.config.signals).unwrap();

	let watcher = if cli.watch {
		Some(notify::recommended_watcher(move |res| match res {
			Ok(_) => {
				if let Ok(wallpaper) = Wallpaper::new(&cli.path) {
					event_proxy
						.send_event(EngineEvent::SetWallpaper(wallpaper))
						.ok();
				}
			}
			Err(e) => println!("watch error: {:?}", e),
		})?)
	} else {
		None
	};

	engine.run(wallpaper, analyzer, watcher);

	Ok(())
}
