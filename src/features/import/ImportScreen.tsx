import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { useCallback, useEffect, useState, type ReactElement } from "react";
import { analyzeAudio, fetchYoutubeAudio, onProgress } from "../../lib/tauri";
import { useApp } from "../../app/state";

export function ImportScreen(): ReactElement {
  const [state, dispatch] = useApp();
  const [url, setUrl] = useState("");

  const run = useCallback(
    async (path: string): Promise<void> => {
      dispatch({ type: "start", path });
      const unlisten = await onProgress(
        "fistula://analyze-progress",
        (progress) => dispatch({ type: "progress", progress }),
      );
      try {
        // 解析は常に移調 0 基準で行い、表示側で移調を適用する (design D5)
        dispatch({
          type: "ready",
          result: await analyzeAudio(path, { ...state.options, transpose: 0 }),
        });
      } catch (error) {
        dispatch({
          type: "error",
          error: error instanceof Error ? error.message : String(error),
        });
      } finally {
        unlisten();
      }
    },
    [dispatch, state.options],
  );

  // ブラウザの File オブジェクトからは実パスが取れないため、
  // D&D は Tauri webview の drag-drop イベントで受ける
  useEffect(() => {
    const unlisten = getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === "drop" && event.payload.paths.length > 0) {
        void run(event.payload.paths[0]);
      }
    });
    return () => {
      void unlisten.then((dispose) => dispose());
    };
  }, [run]);

  const choose = async (): Promise<void> => {
    const result = await open({
      multiple: false,
      filters: [
        { name: "Audio", extensions: ["mp3", "wav", "flac", "ogg", "m4a"] },
      ],
    });
    if (typeof result === "string") await run(result);
  };

  const youtube = async (): Promise<void> => {
    dispatch({ type: "start", path: url });
    const unlisten = await onProgress("fistula://fetch-progress", (progress) =>
      dispatch({ type: "progress", progress }),
    );
    try {
      const audio = await fetchYoutubeAudio(url);
      await run(audio.path);
    } catch (error) {
      dispatch({
        type: "error",
        error: error instanceof Error ? error.message : String(error),
      });
    } finally {
      unlisten();
    }
  };

  return (
    <main className="import">
      <h1>Fistula</h1>
      <p>楽曲からオタマトーン運指譜を生成します。</p>
      <button onClick={() => void choose()}>音声ファイルを選ぶ</button>
      <p>またはファイルをここへドロップ</p>
      <label>
        YouTube URL
        <input
          value={url}
          onChange={(event) => setUrl(event.target.value)}
          placeholder="https://www.youtube.com/watch?..."
        />
      </label>
      <button disabled={!url} onClick={() => void youtube()}>
        音声を取得して解析
      </button>
      {state.error && <p role="alert">{state.error}</p>}
    </main>
  );
}
