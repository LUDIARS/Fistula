mod analysis;
mod audio;
mod commands;
mod ingest;
mod mapping;
mod model;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![commands::analyze_audio, commands::fetch_youtube_audio])
        .run(tauri::generate_context!())
        .expect("error while running Fistula");
}

#[cfg(test)]
mod tests {
    use crate::{analysis::{framing::frames, pitch::detect, postprocess::{map_segments, segments}, transpose::suggested_transpose}, mapping::otamatone::neck_position, model::OtamatoneProfile};
    fn sine(freq: f32, seconds: f32, rate: usize) -> Vec<f32> { (0..(seconds * rate as f32) as usize).map(|index| (2.0 * std::f32::consts::PI * freq * index as f32 / rate as f32).sin()).collect() }
    #[test] fn detects_a4_from_sine_fixture() { let data = sine(440.0, 1.0, 44_100); let framed = frames(&data, 2048, 512).expect("valid framing"); let detected = detect(&framed, 44_100, 2048, 0.7, -40.0); assert!(detected.iter().any(|frame| (frame.freq_hz - 440.0).abs() < 3.0)); }
    #[test] fn sweep_produces_multiple_pitches() { let data: Vec<f32> = (0..44_100).map(|index| { let freq = 220.0 + 220.0 * index as f32 / 44_100.0; (2.0 * std::f32::consts::PI * freq * index as f32 / 44_100.0).sin() }).collect(); let detected = detect(&frames(&data, 2048, 512).expect("valid framing"), 44_100, 2048, 0.5, -40.0); assert!(detected.last().expect("pitch").freq_hz > detected.first().expect("pitch").freq_hz); }
    #[test] fn rejects_short_notes_and_merges_gaps() { let frames = vec![crate::model::PitchFrame { time_sec: 0.0, freq_hz: 440.0, clarity: 1.0, rms: 1.0 }, crate::model::PitchFrame { time_sec: 0.03, freq_hz: 440.0, clarity: 1.0, rms: 1.0 }, crate::model::PitchFrame { time_sec: 0.06, freq_hz: 440.0, clarity: 1.0, rms: 1.0 }]; assert_eq!(segments(&frames, 80.0, 40.0).len(), 1); assert!(segments(&frames[..1], 80.0, 40.0).is_empty()); }
    #[test] fn recommends_octave_fit() { assert_eq!(suggested_transpose(&[93, 95], &OtamatoneProfile::default()), -24); }
    #[test] fn interpolates_calibration() { let profile = OtamatoneProfile { name: "test".into(), midi_min: 60, midi_max: 72, calibration: vec![(60, 0.0), (72, 1.0)] }; assert_eq!(neck_position(&profile, 66), Some(0.5)); assert_eq!(neck_position(&profile, 73), None); let _ = map_segments(&[], 0, &profile); }
}
