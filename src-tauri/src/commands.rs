use crate::{
    analysis::{
        framing::frames,
        pitch::detect,
        postprocess::{map_segments, segments},
        transpose::suggested_transpose,
    },
    audio::decode::decode_mono,
    ingest::youtube,
    model::{AnalysisResult, AnalyzeOptions, FetchedAudio},
};
use std::path::Path;
use tauri::Emitter;

#[tauri::command]
pub fn analyze_audio(
    window: tauri::Window,
    path: String,
    options: AnalyzeOptions,
) -> Result<AnalysisResult, String> {
    let decoded = decode_mono(Path::new(&path))?;
    let duration_sec = decoded.samples.len() as f64 / decoded.sample_rate as f64;
    let windowed = frames(&decoded.samples, options.frame_size, options.hop_size)?;
    if windowed.is_empty() {
        return Err("Audio is shorter than the configured analysis frame size".into());
    }
    window
        .emit("fistula://analyze-progress", 0.25_f64)
        .map_err(|error| format!("Could not emit analysis progress: {error}"))?;
    let pitch_frames = detect(
        &windowed,
        decoded.sample_rate,
        options.frame_size,
        options.clarity_threshold,
        options.rms_threshold_db,
    );
    if pitch_frames.is_empty() {
        return Err(
            "No melody could be detected with the current clarity and volume thresholds".into(),
        );
    }
    window
        .emit("fistula://analyze-progress", 0.75_f64)
        .map_err(|error| format!("Could not emit analysis progress: {error}"))?;
    let untransposed = segments(&pitch_frames, options.min_note_ms, options.merge_gap_ms);
    if untransposed.is_empty() {
        return Err("No playable melody notes remained after post-processing".into());
    }
    let pitches: Vec<u8> = untransposed
        .iter()
        .map(|segment| segment.midi_note)
        .collect();
    let suggested = suggested_transpose(&pitches, &options.profile);
    let (notes, out_of_range_count) =
        map_segments(&untransposed, options.transpose, &options.profile);
    window
        .emit("fistula://analyze-progress", 1.0_f64)
        .map_err(|error| format!("Could not emit analysis progress: {error}"))?;
    Ok(AnalysisResult {
        frames: pitch_frames,
        notes,
        duration_sec,
        sample_rate: decoded.sample_rate,
        suggested_transpose: suggested,
        out_of_range_count,
    })
}

#[tauri::command]
pub fn fetch_youtube_audio(window: tauri::Window, url: String) -> Result<FetchedAudio, String> {
    window
        .emit("fistula://fetch-progress", 0.0_f64)
        .map_err(|error| format!("Could not emit fetch progress: {error}"))?;
    let result = youtube::fetch(&url)?;
    window
        .emit("fistula://fetch-progress", 1.0_f64)
        .map_err(|error| format!("Could not emit fetch progress: {error}"))?;
    Ok(result)
}
