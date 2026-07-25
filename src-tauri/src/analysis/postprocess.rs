use crate::mapping::otamatone::neck_position;
use crate::model::{NoteEvent, OtamatoneProfile, PitchFrame};

/// 量子化・ノート化済みの中間セグメント (移調適用前)。
#[derive(Clone, Debug)]
pub struct Segment {
    pub start_sec: f64,
    pub duration_sec: f64,
    pub midi_note: u8,
    pub cents_offset: f64,
}

/// ヒステリシス量子化の遷移しきい値 (design §4-c: 現ノートから ±0.6 半音)。
const HYSTERESIS_SEMITONES: f64 = 0.6;
/// オクターブ跳び補正の対象とみなす孤立区間の最大フレーム数 (design §4-b)。
const OCTAVE_JUMP_MAX_RUN: usize = 6;

pub fn midi_for_hz(freq: f64) -> f64 {
    69.0 + 12.0 * (freq / 440.0).log2()
}

/// 窓 5 フレームのメディアンフィルタで単発の外れ値を除去する (design §4-a)。
pub fn median_filter(frames: &[PitchFrame]) -> Vec<PitchFrame> {
    frames
        .iter()
        .enumerate()
        .map(|(index, frame)| {
            let window = &frames[index.saturating_sub(2)..(index + 3).min(frames.len())];
            let mut values: Vec<f64> = window.iter().map(|value| value.freq_hz).collect();
            values.sort_by(f64::total_cmp);
            let mut filtered = frame.clone();
            filtered.freq_hz = values[values.len() / 2];
            filtered
        })
        .collect()
}

/// 前後の音程と 12±1 半音差の短い孤立区間をオクターブ寄せする (design §4-b)。
pub fn correct_octave_jumps(frames: &[PitchFrame]) -> Vec<PitchFrame> {
    let midi: Vec<f64> = frames
        .iter()
        .map(|frame| midi_for_hz(frame.freq_hz))
        .collect();
    let runs = rounded_runs(&midi);
    let mut corrected: Vec<PitchFrame> = frames.to_vec();
    for run_index in 1..runs.len().saturating_sub(1) {
        let (start, end, level) = runs[run_index];
        let previous_level = runs[run_index - 1].2;
        let next_level = runs[run_index + 1].2;
        if end - start > OCTAVE_JUMP_MAX_RUN {
            continue;
        }
        if (previous_level - next_level).abs() > 1.0 {
            continue;
        }
        let jump = level - previous_level;
        if (jump.abs() - 12.0).abs() <= 1.0 {
            let factor = if jump > 0.0 { 0.5 } else { 2.0 };
            for frame in &mut corrected[start..end] {
                frame.freq_hz *= factor;
            }
        }
    }
    corrected
}

/// 半音丸めが同値の連続区間 (start, end, level) を列挙する。
fn rounded_runs(midi: &[f64]) -> Vec<(usize, usize, f64)> {
    let mut runs: Vec<(usize, usize, f64)> = Vec::new();
    for (index, value) in midi.iter().enumerate() {
        let level = value.round();
        match runs.last_mut() {
            Some((_, end, last)) if *last == level => *end = index + 1,
            _ => runs.push((index, index + 1, level)),
        }
    }
    runs
}

/// ヒステリシス付きで各フレームを半音へ量子化する (design §4-c)。
/// 現ノートから ±0.6 半音以内のゆらぎでは遷移しない。
fn quantize_with_hysteresis(frames: &[PitchFrame]) -> Vec<(u8, &PitchFrame)> {
    let mut current: Option<f64> = None;
    frames
        .iter()
        .map(|frame| {
            let midi = midi_for_hz(frame.freq_hz);
            let level = match current {
                Some(level) if (midi - level).abs() <= HYSTERESIS_SEMITONES => level,
                _ => midi.round(),
            };
            current = Some(level);
            (level.clamp(0.0, 127.0) as u8, frame)
        })
        .collect()
}

/// PitchFrame 列をノートセグメントへ変換する (design §4-d)。
/// メディアン → オクターブ補正 → ヒステリシス量子化 → 無音跨ぎ分割つき
/// グルーピング → 同音短ギャップ結合 → 最短長フィルタ、の順で処理する。
pub fn segments(frames: &[PitchFrame], min_note_ms: f64, merge_gap_ms: f64) -> Vec<Segment> {
    if frames.is_empty() {
        return Vec::new();
    }
    let filtered = correct_octave_jumps(&median_filter(frames));
    let step = frame_step(&filtered);
    let merge_gap_sec = merge_gap_ms / 1000.0;

    let mut grouped: Vec<(u8, Vec<&PitchFrame>)> = Vec::new();
    for (midi, frame) in quantize_with_hysteresis(&filtered) {
        let continues = grouped.last().is_some_and(|(last_midi, values)| {
            let last_time = values.last().map(|value| value.time_sec).unwrap_or(0.0);
            // 無声区間 (フレーム欠落) が merge_gap を超えたら同音でも別ノートにする
            *last_midi == midi && frame.time_sec - last_time <= merge_gap_sec.max(step * 1.5)
        });
        match grouped.last_mut() {
            Some((_, values)) if continues => values.push(frame),
            _ => grouped.push((midi, vec![frame])),
        }
    }

    let mut result: Vec<Segment> = Vec::new();
    for (midi, values) in grouped {
        let start = values[0].time_sec;
        let end = values.last().map(|value| value.time_sec).unwrap_or(start);
        let duration = (end - start).max(0.0) + step;
        let cents = values
            .iter()
            .map(|value| (midi_for_hz(value.freq_hz) - midi as f64) * 100.0)
            .sum::<f64>()
            / values.len() as f64;
        let merges = result.last().is_some_and(|previous| {
            previous.midi_note == midi
                && (start - (previous.start_sec + previous.duration_sec)) * 1000.0 <= merge_gap_ms
        });
        if merges {
            let previous = result.last_mut().expect("checked by merges");
            previous.duration_sec = start + duration - previous.start_sec;
        } else {
            result.push(Segment {
                start_sec: start,
                duration_sec: duration,
                midi_note: midi,
                cents_offset: cents,
            });
        }
    }
    result
        .into_iter()
        .filter(|segment| segment.duration_sec * 1000.0 >= min_note_ms)
        .collect()
}

fn frame_step(frames: &[PitchFrame]) -> f64 {
    if frames.len() > 1 {
        (frames[1].time_sec - frames[0].time_sec).max(0.0)
    } else {
        0.0
    }
}

/// セグメント列へ移調とネック位置マッピングを適用し、音域外ノート数を数える。
pub fn map_segments(
    segments: &[Segment],
    transpose: i8,
    profile: &OtamatoneProfile,
) -> (Vec<NoteEvent>, usize) {
    let mut outside = 0;
    let notes = segments
        .iter()
        .filter_map(|segment| {
            let midi = (segment.midi_note as i16 + transpose as i16).clamp(0, 127) as u8;
            match neck_position(profile, midi) {
                Some(neck_pos) => Some(NoteEvent {
                    start_sec: segment.start_sec,
                    duration_sec: segment.duration_sec,
                    midi_note: midi,
                    cents_offset: segment.cents_offset,
                    neck_pos,
                }),
                None => {
                    outside += 1;
                    None
                }
            }
        })
        .collect();
    (notes, outside)
}
