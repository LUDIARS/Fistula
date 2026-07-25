pub fn hann_window(size: usize) -> Vec<f32> { (0..size).map(|index| 0.5 - 0.5 * (2.0 * std::f32::consts::PI * index as f32 / (size.saturating_sub(1).max(1)) as f32).cos()).collect() }
pub fn frames(samples: &[f32], frame_size: usize, hop_size: usize) -> Result<Vec<(usize, Vec<f32>)>, String> {
    if frame_size == 0 || hop_size == 0 { return Err("Frame size and hop size must be positive".into()); }
    if samples.len() < frame_size { return Ok(Vec::new()); }
    let window = hann_window(frame_size);
    Ok((0..=samples.len() - frame_size).step_by(hop_size).map(|start| (start, samples[start..start + frame_size].iter().zip(&window).map(|(sample, weight)| sample * weight).collect())).collect())
}
