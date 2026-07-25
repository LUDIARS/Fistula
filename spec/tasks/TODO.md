# 実装タスク分解

[design.md](../design.md) 準拠。MVP 縦切りではなく設計済み機能のフルセットで実装する。
T1〜T6 は 1 実装 PR に集約、T7 のみ別リポ PR。

- [ ] **T1. Rust コア: デコード + フレーム分割** — symphonia デコード
      (`audio/decode.rs`)、mono ミックスダウン、Hann 窓フレーム分割
      (`analysis/framing.rs`)。対応外コーデックは即エラー (§7.1)。
- [ ] **T2. Rust コア: ピッチ検出 + 後処理** — McLeod 推定 + clarity/rms
      フィルタ (`analysis/pitch.rs`)、メディアン/オクターブ補正/半音量子化
      (ヒステリシス)/セグメンテーション (`analysis/postprocess.rs`)。
      合成波形 fixture のユニットテスト同梱 (design §11)。
- [ ] **T3. マッピング + 自動移調 + command 公開** — OtamatoneProfile と
      neck_pos 補間 (`mapping/otamatone.rs`)、音域フィット推奨
      (`analysis/transpose.rs`)、`analyze_audio` command + 進捗 event
      (`commands.rs`, `model.rs`)。
- [ ] **T4. フロント: 画面フロー + Import + 解析呼び出し** — app シェル
      (Context/Reducer)、ファイル選択/D&D、進捗表示、`lib/tauri/` 型付ラッパ。
- [ ] **T5. フロント: 演奏ビュー** — Canvas スクロール譜面 (横=時間/縦=ネック位置)、
      audio.currentTime 同期、再生/一時停止/シーク/速度 0.5〜1.0x/A-B ループ、
      調整パネル (移調スライダ = `lib/score/` TS 再マッピング + ゴールデンテスト)。
- [ ] **T6. エクスポート** — 段組 PNG (OffscreenCanvas 2x)、pdf-lib で A4 PDF、
      save dialog/fs plugin 配線。
- [ ] **T7. PROJECT-CODES.md 登録 (別 PR / LUDIARS リポ)** — 略称 `Fi` 案で
      アセット/ツール節に追記。

## 完了条件

- `npm run tauri dev` で 音源選択 → 解析 → スクロール演奏 → PNG/PDF 保存が一気通貫
  (ビルド・実行確認はユーザ指示のタイミングで実施)。
- Rust ユニットテスト + TS ゴールデンテストが green。
