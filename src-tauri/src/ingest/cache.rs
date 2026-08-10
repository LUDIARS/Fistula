use crate::model::FetchedAudio;
use std::{env, fs, path::PathBuf};
pub fn cache_directory() -> Result<PathBuf, String> {
    let local = env::var_os("LOCALAPPDATA").ok_or_else(|| {
        "LOCALAPPDATA is unavailable; cannot determine cache directory".to_owned()
    })?;
    let path = PathBuf::from(local).join("Fistula").join("cache");
    fs::create_dir_all(&path).map_err(|error| format!("Could not create audio cache: {error}"))?;
    Ok(path)
}
pub fn cached_audio(video_id: &str) -> Result<Option<FetchedAudio>, String> {
    let dir = cache_directory()?;
    let audio = dir.join(format!("{video_id}.m4a"));
    let metadata = dir.join(format!("{video_id}.json"));
    if !audio.is_file() || !metadata.is_file() {
        return Ok(None);
    }
    let contents = fs::read_to_string(metadata)
        .map_err(|error| format!("Could not read audio cache metadata: {error}"))?;
    let mut item: FetchedAudio = serde_json::from_str(&contents)
        .map_err(|error| format!("Audio cache metadata is invalid: {error}"))?;
    item.path = audio.to_string_lossy().into_owned();
    Ok(Some(item))
}
pub fn save_metadata(item: &FetchedAudio) -> Result<(), String> {
    let dir = cache_directory()?;
    let path = dir.join(format!("{}.json", item.video_id));
    fs::write(
        path,
        serde_json::to_vec(item)
            .map_err(|error| format!("Could not encode audio cache metadata: {error}"))?,
    )
    .map_err(|error| format!("Could not write audio cache metadata: {error}"))
}
