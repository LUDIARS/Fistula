use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PitchFrame { pub time_sec: f64, pub freq_hz: f64, pub clarity: f64, pub rms: f64 }

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NoteEvent { pub start_sec: f64, pub duration_sec: f64, pub midi_note: u8, pub cents_offset: f64, pub neck_pos: f64 }

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OtamatoneProfile { pub name: String, pub midi_min: u8, pub midi_max: u8, pub calibration: Vec<(u8, f64)> }

impl Default for OtamatoneProfile {
    fn default() -> Self { Self { name: "Standard Otamatone".into(), midi_min: 57, midi_max: 81, calibration: (57..=81).map(|m| (m, (m - 57) as f64 / 24.0)).collect() } }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeOptions { pub frame_size: usize, pub hop_size: usize, pub clarity_threshold: f64, pub rms_threshold_db: f64, pub min_note_ms: f64, pub merge_gap_ms: f64, pub transpose: i8, pub profile: OtamatoneProfile }

impl Default for AnalyzeOptions { fn default() -> Self { Self { frame_size: 2048, hop_size: 512, clarity_threshold: 0.7, rms_threshold_db: -40.0, min_note_ms: 80.0, merge_gap_ms: 40.0, transpose: 0, profile: OtamatoneProfile::default() } } }

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisResult { pub frames: Vec<PitchFrame>, pub notes: Vec<NoteEvent>, pub duration_sec: f64, pub sample_rate: u32, pub suggested_transpose: i8, pub out_of_range_count: usize }

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchedAudio { pub path: String, pub video_id: String, pub title: String, pub duration_sec: f64 }
