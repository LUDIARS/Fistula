use std::{fs::File, path::Path};
use symphonia::{core::{audio::{AudioBufferRef, SampleBuffer}, codecs::DecoderOptions, errors::Error, formats::FormatOptions, io::MediaSourceStream, meta::MetadataOptions, probe::Hint}, default::{get_codecs, get_probe}};

pub struct DecodedAudio { pub samples: Vec<f32>, pub sample_rate: u32 }

pub fn decode_mono(path: &Path) -> Result<DecodedAudio, String> {
    if !path.is_file() { return Err(format!("Audio file does not exist: {}", path.display())); }
    let file = File::open(path).map_err(|error| format!("Could not open audio file: {error}"))?;
    let mut hint = Hint::new();
    if let Some(extension) = path.extension().and_then(|value| value.to_str()) { hint.with_extension(extension); }
    let source = MediaSourceStream::new(Box::new(file), Default::default());
    let probed = get_probe().format(&hint, source, &FormatOptions::default(), &MetadataOptions::default()).map_err(|error| format!("Unsupported or undecodable audio format: {error}"))?;
    let mut format = probed.format;
    let track = format.default_track().ok_or_else(|| "Audio file contains no default audio track".to_owned())?;
    let sample_rate = track.codec_params.sample_rate.ok_or_else(|| "Audio sample rate is unavailable".to_owned())?;
    let track_id = track.id;
    let mut decoder = get_codecs().make(&track.codec_params, &DecoderOptions::default()).map_err(|error| format!("Unsupported audio codec: {error}"))?;
    let mut mono = Vec::new();
    loop {
        let packet = match format.next_packet() { Ok(packet) => packet, Err(Error::IoError(_)) => break, Err(error) => return Err(format!("Audio packet decode failed: {error}")) };
        if packet.track_id() != track_id { continue; }
        match decoder.decode(&packet) {
            Ok(buffer) => append_mono(&mut mono, buffer),
            Err(Error::DecodeError(error)) => return Err(format!("Audio decode failed: {error}")),
            Err(error) => return Err(format!("Audio decode failed: {error}")),
        }
    }
    if mono.is_empty() { return Err("Audio file contains no decodable samples".to_owned()); }
    Ok(DecodedAudio { samples: mono, sample_rate })
}

fn append_mono(destination: &mut Vec<f32>, buffer: AudioBufferRef<'_>) {
    let spec = *buffer.spec(); let frames = buffer.frames(); let channels = spec.channels.count();
    let mut samples = SampleBuffer::<f32>::new(frames as u64, spec); samples.copy_interleaved_ref(buffer);
    for frame in samples.samples().chunks(channels) { destination.push(frame.iter().sum::<f32>() / channels as f32); }
}
