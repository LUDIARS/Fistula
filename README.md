# Fistula

楽曲ファイルを解析し、オタマトーン向けの運指譜を生成する個人練習用アプリです。

YouTube 取り込みは、利用者が権利を持つ、または利用が許可されたコンテンツを個人練習のために扱う用途に限定してください。ダウンロードが制限されるコンテンツの扱い、YouTube 利用規約の遵守、および法令への適合は利用者の責任です。取得音声はローカルキャッシュにのみ保存され、エクスポート成果物に音声は含まれません。

楽曲を波形解析してオタマトーンの運指譜を作るアプリ。

音声ファイル（mp3/wav 等）を読み込み、ピッチ検出でメロディを抽出し、
オタマトーン（フレットレス・単音の玩具楽器）のネック位置に変換した
「運指譜」を生成する。リズムゲーム風のスクロール演奏画面で、実際に
音を聴きながらどこを押さえるかを確認しながら練習できることを目指す。

## 特徴（予定）

- 音声ファイルの波形解析（デコード + ピッチ/オンセット検出）
- YouTube URL からの音声のみ取得（yt-dlp、ローカルキャッシュ・個人練習用途）
- 検出結果をオタマトーンのネック位置（クロマチック）にマッピング
- リズムゲーム風スクロール UI で演奏タイミングをガイド
- 運指譜のプレビュー / 静止画・PDF エクスポート

## 技術スタック

| レイヤー | 技術 |
|---|---|
| フレームワーク | Tauri v2 (Rust) |
| フロントエンド | React 19 · TypeScript · Vite |
| 音声デコード | [symphonia](https://github.com/pdeljanov/Symphonia) |
| ピッチ検出 | [pitch-detection](https://crates.io/crates/pitch-detection)（不足時は [aubio-rs](https://crates.io/crates/aubio-rs) 経由で C 実装の aubio を利用） |

## 開発

```bash
npm install
npm run tauri dev
```

## 設計ドキュメント

- [spec/design.md](spec/design.md) — 全体設計
- [spec/tasks/](spec/tasks/) — 実装タスク分解

## ライセンス

MIT License. See [LICENSE](LICENSE).
