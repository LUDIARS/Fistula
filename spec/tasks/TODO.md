# 残タスク（次回以降）

Tauri + React scaffold のみ完了。以下は未着手。

- [ ] ピッチ検出ライブラリ選定（`pitch-detection`(pure Rust) vs `aubio-rs`(C実装 aubio バインディング)）
- [ ] 音声デコード + ピッチ検出の Rust コア実装（symphonia でデコード → フレーム毎ピッチ検出 → Tauri command で公開）
- [ ] 検出結果 → オタマトーン運指（ネック位置、クロマチック）へのマッピング設計
- [ ] リズムゲーム風スクロール演奏 UI（音程バーが流れてくる形式）
- [ ] 音声再生とスクロール UI の同期
- [ ] 運指譜のプレビュー / 静止画・PDF エクスポート
- [ ] `LUDIARS/PROJECT-CODES.md` へのプロジェクトコード登録（別PR）
