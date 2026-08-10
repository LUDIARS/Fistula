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
        .invoke_handler(tauri::generate_handler![
            commands::analyze_audio,
            commands::fetch_youtube_audio
        ])
        .run(tauri::generate_context!())
        .expect("error while running Fistula");
}

#[cfg(test)]
mod tests {
    use crate::analysis::framing::frames;
    use crate::analysis::pitch::detect;
    use crate::analysis::postprocess::{correct_octave_jumps, map_segments, segments};
    use crate::analysis::transpose::suggested_transpose;
    use crate::mapping::otamatone::neck_position;
    use crate::model::{OtamatoneProfile, PitchFrame};

    fn sine(freq: f32, seconds: f32, rate: usize) -> Vec<f32> {
        (0..(seconds * rate as f32) as usize)
            .map(|index| (2.0 * std::f32::consts::PI * freq * index as f32 / rate as f32).sin())
            .collect()
    }

    fn pitch_frames(specs: &[(f64, f64)]) -> Vec<PitchFrame> {
        specs
            .iter()
            .map(|(time_sec, freq_hz)| PitchFrame {
                time_sec: *time_sec,
                freq_hz: *freq_hz,
                clarity: 1.0,
                rms: 1.0,
            })
            .collect()
    }

    #[test]
    fn detects_a4_from_sine_fixture() {
        let data = sine(440.0, 1.0, 44_100);
        let framed = frames(&data, 2048, 512).expect("valid framing");
        let detected = detect(&framed, 44_100, 2048, 0.7, -40.0);
        assert!(detected
            .iter()
            .any(|frame| (frame.freq_hz - 440.0).abs() < 3.0));
    }

    #[test]
    fn sweep_produces_multiple_pitches() {
        let data: Vec<f32> = (0..44_100)
            .map(|index| {
                let freq = 220.0 + 220.0 * index as f32 / 44_100.0;
                (2.0 * std::f32::consts::PI * freq * index as f32 / 44_100.0).sin()
            })
            .collect();
        let detected = detect(
            &frames(&data, 2048, 512).expect("valid framing"),
            44_100,
            2048,
            0.5,
            -40.0,
        );
        assert!(detected.last().expect("pitch").freq_hz > detected.first().expect("pitch").freq_hz);
    }

    #[test]
    fn rejects_short_notes_and_merges_gaps() {
        let frames = pitch_frames(&[(0.0, 440.0), (0.03, 440.0), (0.06, 440.0)]);
        assert_eq!(segments(&frames, 80.0, 40.0).len(), 1);
        assert!(segments(&frames[..1], 80.0, 40.0).is_empty());
    }

    #[test]
    fn corrects_isolated_octave_jump() {
        // A4 が続く中に 2 フレームだけ A5 (octave error) が混ざるケース
        let mut specs: Vec<(f64, f64)> = (0..8).map(|i| (i as f64 * 0.01, 440.0)).collect();
        specs.extend([(0.08, 880.0), (0.09, 880.0)]);
        specs.extend((10..18).map(|i| (i as f64 * 0.01, 440.0)));
        let corrected = correct_octave_jumps(&pitch_frames(&specs));
        assert!(corrected
            .iter()
            .all(|frame| (frame.freq_hz - 440.0).abs() < 1.0));
        let segs = segments(&pitch_frames(&specs), 80.0, 40.0);
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].midi_note, 69);
    }

    #[test]
    fn hysteresis_keeps_wobbling_pitch_on_one_note() {
        // ±30 cents 程度のゆらぎ (440Hz ±8Hz) は 1 ノートに留まる
        let specs: Vec<(f64, f64)> = (0..12)
            .map(|i| (i as f64 * 0.01, if i % 2 == 0 { 448.0 } else { 432.0 }))
            .collect();
        let segs = segments(&pitch_frames(&specs), 80.0, 40.0);
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].midi_note, 69);
    }

    #[test]
    fn splits_same_note_across_silence() {
        // 同じ A4 でも 300ms 超の無声区間を跨いだら別ノートになる
        let mut specs: Vec<(f64, f64)> = (0..8).map(|i| (i as f64 * 0.01, 440.0)).collect();
        specs.extend((0..8).map(|i| (0.4 + i as f64 * 0.01, 440.0)));
        let segs = segments(&pitch_frames(&specs), 40.0, 40.0);
        assert_eq!(segs.len(), 2);
    }

    #[test]
    fn recommends_octave_fit() {
        assert_eq!(
            suggested_transpose(&[93, 95], &OtamatoneProfile::default()),
            -24
        );
    }

    #[test]
    fn interpolates_calibration() {
        let profile = OtamatoneProfile {
            name: "test".into(),
            midi_min: 60,
            midi_max: 72,
            calibration: vec![(60, 0.0), (72, 1.0)],
        };
        assert_eq!(neck_position(&profile, 66), Some(0.5));
        assert_eq!(neck_position(&profile, 73), None);
        let _ = map_segments(&[], 0, &profile);
    }
}
