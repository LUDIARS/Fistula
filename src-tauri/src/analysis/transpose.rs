use crate::model::OtamatoneProfile;
pub fn suggested_transpose(notes: &[u8], profile: &OtamatoneProfile) -> i8 { [-24_i8, -12, 0, 12, 24].into_iter().min_by_key(|shift| { let outside = notes.iter().filter(|note| { let shifted = **note as i16 + *shift as i16; shifted < profile.midi_min as i16 || shifted > profile.midi_max as i16 }).count(); (outside, shift.unsigned_abs()) }).unwrap_or(0) }
