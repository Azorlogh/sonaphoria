use aubio::{OnsetMode, Tempo};

use super::{Analyzer, BUFFER_SIZE, HOP_SIZE};

pub struct BeatDetector {
	tempo: Tempo,
	last_beat: f32,
}

impl BeatDetector {
	pub fn new() -> Self {
		let tempo = Tempo::new(OnsetMode::Complex, BUFFER_SIZE, HOP_SIZE, 48000).unwrap();
		Self {
			tempo,
			last_beat: 0.0,
		}
	}
}

impl Analyzer for BeatDetector {
	fn process(&mut self, buf: &[f32; BUFFER_SIZE]) -> f32 {
		let beat = self.tempo.do_result(buf).unwrap_or(0.0);
		if beat != 0.0 {
			self.last_beat = 0.0;
		} else {
			self.last_beat += BUFFER_SIZE as f32 / 48000.0;
		}
		self.last_beat
	}
}
