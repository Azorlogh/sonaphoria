pub const SAMPLE_RATE: usize = 48000;
pub const BUFFER_SIZE: usize = 256;
pub const HOP_SIZE: usize = 256;
pub const DT: f32 = BUFFER_SIZE as f32 / SAMPLE_RATE as f32;
