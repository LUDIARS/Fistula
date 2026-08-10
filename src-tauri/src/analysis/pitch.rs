use crate::model::PitchFrame;
use pitch_detection::detector::{mcleod::McLeodDetector, PitchDetector};
pub fn detect(
    frames: &[(usize, Vec<f32>)],
    sample_rate: u32,
    frame_size: usize,
    clarity_threshold: f64,
    rms_threshold_db: f64,
) -> Vec<PitchFrame> {
    let mut detector = McLeodDetector::new(frame_size, frame_size / 2);
    frames
        .iter()
        .filter_map(|(start, frame)| {
            let rms = (frame.iter().map(|v| v * v).sum::<f32>() / frame.len() as f32).sqrt() as f64;
            let db = 20.0 * rms.max(1e-12).log10();
            let pitch =
                detector.get_pitch(frame, sample_rate as usize, 0.001, clarity_threshold as f32)?;
            if db < rms_threshold_db {
                return None;
            }
            Some(PitchFrame {
                time_sec: (*start + frame_size / 2) as f64 / sample_rate as f64,
                freq_hz: pitch.frequency as f64,
                clarity: pitch.clarity as f64,
                rms,
            })
        })
        .collect()
}
