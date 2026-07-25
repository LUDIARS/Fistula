use crate::model::OtamatoneProfile;
pub fn neck_position(profile: &OtamatoneProfile, midi: u8) -> Option<f64> {
    if midi < profile.midi_min || midi > profile.midi_max || profile.calibration.is_empty() { return None; }
    let points = &profile.calibration;
    if let Some((_, position)) = points.iter().find(|(note, _)| *note == midi) { return Some(*position); }
    points.windows(2).find_map(|pair| { let (left_note, left_position) = pair[0]; let (right_note, right_position) = pair[1]; if midi > left_note && midi < right_note { Some(left_position + (right_position - left_position) * (midi - left_note) as f64 / (right_note - left_note) as f64) } else { None } })
}
