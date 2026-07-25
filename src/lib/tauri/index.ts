import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Profile, ScoreNote } from "../score";
export interface AnalyzeOptions {
  frameSize: number;
  hopSize: number;
  clarityThreshold: number;
  rmsThresholdDb: number;
  minNoteMs: number;
  mergeGapMs: number;
  transpose: number;
  profile: Profile;
}
export interface AnalysisResult {
  frames: { timeSec: number; freqHz: number; clarity: number; rms: number }[];
  notes: ScoreNote[];
  durationSec: number;
  sampleRate: number;
  suggestedTranspose: number;
  outOfRangeCount: number;
}
export interface FetchedAudio {
  path: string;
  videoId: string;
  title: string;
  durationSec: number;
}
export const analyzeAudio = (
  path: string,
  options: AnalyzeOptions,
): Promise<AnalysisResult> => invoke("analyze_audio", { path, options });
export const fetchYoutubeAudio = (url: string): Promise<FetchedAudio> =>
  invoke("fetch_youtube_audio", { url });
export const onProgress = (
  event: "fistula://analyze-progress" | "fistula://fetch-progress",
  callback: (value: number) => void,
): Promise<UnlistenFn> =>
  listen<number>(event, (message) => callback(message.payload));
