import { convertFileSrc } from "@tauri-apps/api/core";
import { useEffect, useRef, useState, type ReactElement } from "react";
import { useApp } from "../../app/state";
import { useScoreNotes } from "../../app/useScoreNotes";
import { midiNoteName, neckPosition } from "../../lib/score";

/** 1 秒あたりの横スクロール幅 (px)。 */
const PX_PER_SEC = 110;
/** ノートバーの縦描画余白 (px)。 */
const LANE_PADDING = 16;

export function PlayerScreen(): ReactElement {
  const [state] = useApp();
  const notes = useScoreNotes();
  const audio = useRef<HTMLAudioElement>(null);
  const canvas = useRef<HTMLCanvasElement>(null);
  const [time, setTime] = useState(0);
  const [rate, setRate] = useState(1);
  const [loopStart, setLoopStart] = useState<number | null>(null);
  const [loopEnd, setLoopEnd] = useState<number | null>(null);
  const profile = state.options.profile;

  useEffect(() => {
    let frame = 0;
    const draw = (): void => {
      const context = canvas.current?.getContext("2d");
      if (context && canvas.current) {
        const { width, height } = canvas.current;
        // 時刻源は audio.currentTime のみ (design D4)
        const now = audio.current?.currentTime ?? 0;
        const laneHeight = height - LANE_PADDING * 2;
        const yFor = (neckPos: number): number =>
          (1 - neckPos) * laneHeight + LANE_PADDING;

        context.clearRect(0, 0, width, height);
        context.fillStyle = "#111827";
        context.fillRect(0, 0, width, height);

        // 半音ごとのガイド線 + 音名ラベル (C とラベル行のみ強調)
        context.font = "10px sans-serif";
        for (let midi = profile.midiMin; midi <= profile.midiMax; midi += 1) {
          const neckPos = neckPosition(profile, midi);
          if (neckPos === null) continue;
          const y = yFor(neckPos);
          const isC = midi % 12 === 0;
          context.strokeStyle = isC ? "#4b5563" : "#1f2937";
          context.beginPath();
          context.moveTo(0, y);
          context.lineTo(width, y);
          context.stroke();
          if (isC || midi === profile.midiMin || midi === profile.midiMax) {
            context.fillStyle = "#9ca3af";
            context.fillText(midiNoteName(midi), 4, y - 2);
          }
        }

        const playhead = width * 0.2;
        context.strokeStyle = "#fbbf24";
        context.beginPath();
        context.moveTo(playhead, 0);
        context.lineTo(playhead, height);
        context.stroke();

        notes.forEach((note) => {
          const x = playhead + (note.startSec - now) * PX_PER_SEC;
          const noteWidth = note.durationSec * PX_PER_SEC;
          if (x + noteWidth >= 0 && x <= width) {
            const y = yFor(note.neckPos);
            context.fillStyle = "#60a5fa";
            context.fillRect(x, y - 7, noteWidth, 14);
            context.fillStyle = "#e5e7eb";
            context.fillText(midiNoteName(note.midiNote), x + 2, y - 9);
          }
        });
      }
      frame = requestAnimationFrame(draw);
    };
    frame = requestAnimationFrame(draw);
    return () => cancelAnimationFrame(frame);
  }, [notes, profile]);

  const tick = (): void => {
    const element = audio.current;
    if (!element) return;
    if (
      loopEnd !== null &&
      loopStart !== null &&
      element.currentTime >= loopEnd
    )
      element.currentTime = loopStart;
    setTime(element.currentTime);
  };

  const seek = (value: number): void => {
    if (audio.current) audio.current.currentTime = value;
    setTime(value);
  };

  return (
    <main>
      <h1>演奏ビュー</h1>
      <canvas ref={canvas} width={960} height={360} aria-label="運指譜" />
      <audio
        ref={audio}
        src={state.path ? convertFileSrc(state.path) : undefined}
        onTimeUpdate={tick}
        onLoadedMetadata={tick}
      />
      <div>
        <button onClick={() => void audio.current?.play()}>再生</button>
        <button onClick={() => audio.current?.pause()}>一時停止</button>
        <input
          type="range"
          min="0"
          max={state.result?.durationSec ?? 0}
          step="0.01"
          value={time}
          onChange={(event) => seek(Number(event.target.value))}
        />
        <label>
          速度
          <input
            type="range"
            min="0.5"
            max="1"
            step="0.1"
            value={rate}
            onChange={(event) => {
              const value = Number(event.target.value);
              setRate(value);
              if (audio.current) audio.current.playbackRate = value;
            }}
          />
        </label>
        <button onClick={() => setLoopStart(time)}>A</button>
        <button onClick={() => setLoopEnd(time)}>B</button>
      </div>
      <p>
        {loopStart !== null && loopEnd !== null
          ? `ループ: ${loopStart.toFixed(2)}–${loopEnd.toFixed(2)} 秒`
          : "A/B を設定して区間ループ"}
      </p>
    </main>
  );
}
