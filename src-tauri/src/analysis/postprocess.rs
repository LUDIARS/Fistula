use crate::{mapping::otamatone::neck_position, model::{NoteEvent, OtamatoneProfile, PitchFrame}};

#[derive(Clone, Debug)]
pub struct Segment { pub start_sec: f64, pub duration_sec: f64, pub midi_note: u8, pub cents_offset: f64 }
pub fn midi_for_hz(freq: f64) -> f64 { 69.0 + 12.0 * (freq / 440.0).log2() }
pub fn median_filter(frames: &[PitchFrame]) -> Vec<PitchFrame> { frames.iter().enumerate().map(|(index, frame)| { let mut values: Vec<f64> = frames[index.saturating_sub(2)..(index + 3).min(frames.len())].iter().map(|value| value.freq_hz).collect(); values.sort_by(f64::total_cmp); let mut filtered = frame.clone(); filtered.freq_hz = values[values.len() / 2]; filtered }).collect() }
pub fn segments(frames: &[PitchFrame], min_note_ms: f64, merge_gap_ms: f64) -> Vec<Segment> {
    if frames.is_empty() { return Vec::new(); }
    let filtered = median_filter(frames); let mut grouped: Vec<(u8, Vec<&PitchFrame>)> = Vec::new();
    for frame in &filtered { let midi = midi_for_hz(frame.freq_hz).round().clamp(0.0, 127.0) as u8; if let Some((last, values)) = grouped.last_mut().filter(|(last, _)| (midi as i16 - *last as i16).unsigned_abs() == 0) { let _ = last; values.push(frame); } else { grouped.push((midi, vec![frame])); } }
    let mut result: Vec<Segment> = Vec::new();
    for (midi, values) in grouped { let start = values[0].time_sec; let end = values.last().map(|value| value.time_sec).unwrap_or(start); let duration = (end - start).max(0.0) + frame_step(&filtered); let cents = values.iter().map(|value| (midi_for_hz(value.freq_hz) - midi as f64) * 100.0).sum::<f64>() / values.len() as f64; if let Some(previous) = result.last_mut().filter(|previous| previous.midi_note == midi && (start - (previous.start_sec + previous.duration_sec)) * 1000.0 <= merge_gap_ms) { previous.duration_sec = start + duration - previous.start_sec; } else { result.push(Segment { start_sec: start, duration_sec: duration, midi_note: midi, cents_offset: cents }); } }
    result.into_iter().filter(|segment| segment.duration_sec * 1000.0 >= min_note_ms).collect()
}
fn frame_step(frames: &[PitchFrame]) -> f64 { if frames.len() > 1 { (frames[1].time_sec - frames[0].time_sec).max(0.0) } else { 0.0 } }
pub fn map_segments(segments: &[Segment], transpose: i8, profile: &OtamatoneProfile) -> (Vec<NoteEvent>, usize) { let mut outside = 0; let notes = segments.iter().filter_map(|segment| { let midi = (segment.midi_note as i16 + transpose as i16).clamp(0, 127) as u8; match neck_position(profile, midi) { Some(neck_pos) => Some(NoteEvent { start_sec: segment.start_sec, duration_sec: segment.duration_sec, midi_note: midi, cents_offset: segment.cents_offset, neck_pos }), None => { outside += 1; None } } }).collect(); (notes, outside) }
