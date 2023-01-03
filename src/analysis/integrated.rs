use crate::consts::{BUFFER_SIZE, DT};

use super::Analyzer;

pub struct Integrated {
	value: f32,
	inner: Box<dyn Analyzer>,
}

impl Integrated {
	pub fn new(inner: Box<dyn Analyzer>) -> Self {
		Self { value: 0.0, inner }
	}
}

impl Analyzer for Integrated {
	fn process(&mut self, buf: &[f32; BUFFER_SIZE]) -> f32 {
		self.value += self.inner.process(buf) * DT;
		self.value
	}
}
