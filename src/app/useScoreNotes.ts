import { useMemo } from "react";
import { remapNotes, type ScoreNote } from "../lib/score";
import { useApp } from "./state";

/**
 * 解析結果 (移調 0 基準の生ノート列) に現在の移調設定とプロファイルを
 * 適用した表示用ノート列を返す。演奏ビューとエクスポートの共通入力
 * (design D5: 移調はTS側再マッピングで即時反映)。
 */
export function useScoreNotes(): ScoreNote[] {
  const [state] = useApp();
  return useMemo(
    () =>
      state.result
        ? remapNotes(
            state.result.notes,
            state.options.transpose,
            state.options.profile,
          )
        : [],
    [state.result, state.options.transpose, state.options.profile],
  );
}
