# Fistula 実装指示書

## 目的

楽曲（音声ファイル）を波形解析し、オタマトーンで弾けるように運指譜（ピッチの
時系列 → ネック位置）を自動生成するアプリを作る。単なる静止画の譜面だけでなく、
リズムゲーム風のスクロール演奏 UI で「今どこを押さえるか」を音に合わせて
ガイドできることを目指す。

## オタマトーンの前提知識

- フレットレスの単音楽器。ネック（茎の部分）を指で押さえる位置で連続的に
  音程が変わる（ギターのような離散フレットはない）。
- 一般的な運指譜は「ネック上のどのあたりを押さえるか」を音程に応じた位置で
  示す。本アプリでは便宜上 **クロマチック（半音単位）** でネック位置を
  離散化し、位置 → 周波数の対応テーブルを持つ。
- 実機の音域・ネック長は製品差があるため、パラメータ化しておき、
  そのうち UI から調整できるようにする（本指示書では固定値でよい）。

## 技術スタック（決定済み）

| レイヤー | 技術 | 理由 |
|---|---|---|
| フレームワーク | Tauri v2 (Rust) | デスクトップ完結、Ars-editor と同系統でノウハウ流用可 |
| フロントエンド | React 19 + TypeScript + Vite | scaffold 済み (create-tauri-app react-ts) |
| 音声デコード | [symphonia](https://github.com/pdeljanov/Symphonia) | mp3/wav/flac 等を pure Rust でデコード |
| ピッチ検出 | [pitch-detection](https://crates.io/crates/pitch-detection) を第一候補 | pure Rust (YIN / McLeod 実装)。まず試して精度不足なら以下へ |
| ピッチ検出 (fallback) | [aubio-rs](https://crates.io/crates/aubio-rs) | C 実装 aubio のバインディング。onset 検出や高精度ピッチ (yinfft 等) が必要な場合 |

pitch-detection と aubio-rs は両方試して比較し、精度・ビルドの手軽さ
（aubio-rs は libaubio の C ビルドが必要でクロスプラットフォームのビルド
コストが高い）を見て最終判断すること。

## アーキテクチャ概要

```
[音声ファイル] --symphonia--> PCMサンプル
                                  |
                      フレーム分割 (窓関数)
                                  |
                  pitch-detection / aubio-rs でピッチ推定
                                  |
                     (time, frequency, amplitude) の時系列
                                  |
              オタマトーン ネック位置マッピング (クロマチック量子化)
                                  |
                   ノートイベント列 (start, duration, position)
                                  |
        +----------------------+----------------------+
        |                                             |
  リズムゲーム風スクロールUI                    静止画/PDF譜面エクスポート
  (音声再生と同期してスクロール)
```

Rust 側 (`src-tauri/src`) に解析ロジックを置き、Tauri command として
フロントエンドへ `NoteEvent[]` を返す。フロントエンドは React + Canvas
(または SVG) でスクロール表示と再生同期を行う。

## 残タスク（詳細）

### 1. ピッチ検出ライブラリ選定
- `pitch-detection` crate を Cargo.toml に追加し、サンプル音声（単音・
  和音なしのメロディ）で McLeod / YIN アルゴリズムを試す。
- 精度が不十分な場合のみ `aubio-rs` を追加検討する（追加のネイティブ
  ビルド依存が増えるため、必要性を確認してから導入する）。

### 2. 音声デコード + ピッチ検出 Rust コア実装
- `symphonia` で音声ファイルを PCM にデコード。
- 一定フレーム長（例: 2048 サンプル、ホップ 512 サンプル）でピッチ推定を
  繰り返し、`(time_sec, frequency_hz, amplitude)` の配列を生成。
- 無音/非音程区間（打楽器的なノイズ等）は amplitude 閾値でフィルタ。
- Tauri command `analyze_audio(path: String) -> Vec<PitchFrame>` として
  公開。

### 3. オタマトーン運指マッピング設計
- 周波数 → 最も近い半音（MIDI note number）に量子化。
- MIDI note number → ネック位置（0.0〜1.0 の正規化位置、もしくは mm）
  への変換テーブルを定義（音域は仮に 2〜3 オクターブ分を想定）。
- 連続するフレームをまとめてノートイベント（開始時刻・長さ・位置）に
  グルーピングするロジック（同一半音が一定時間続いたら 1 ノートとする）。

### 4. リズムゲーム風スクロール演奏 UI
- ノートイベント列を Canvas か SVG でタイムライン表示。
- 音声再生位置（`<audio>` の currentTime 等）に合わせてノートバーが
  流れてくる演出（太鼓さん大魔王 / 音ゲーのレーン UI をイメージ）。
- 再生・一時停止・シークのコントロール。

### 5. 音声再生とスクロールUIの同期
- フロントエンドの再生クロックと解析結果のタイムスタンプを同期させる。
- ズレが出やすいので、再生開始時刻を基準にした経過時間計算で統一する。

### 6. 運指譜のプレビュー / 静止画・PDF エクスポート
- ノートイベント列を横スクロール譜面として画像化（Canvas → PNG）。
- 印刷用に PDF 出力（`pdf-lib` 等 JS ライブラリ、または Rust 側で
  `printpdf` crate を利用）。

### 7. LUDIARS/PROJECT-CODES.md への登録（別PR、LUDIARS リポジトリ側）
- `PROJECT-CODES.md` の適切なカテゴリ（アセット/ツール、もしくはゲーム
  隣接）に Fistula の略称・役割を追記する PR を LUDIARS リポジトリに出す。
- 略称は未定。命名時は既存の C 系との重複に注意
  （`Cc/Ci/Cl/Cn/Co/Cr/Cs/Ca/Cu/Cx` は使用済み）。

## 開発フロー上の注意

- LUDIARS org 共通ルールにより `main` への直 push は pre-push hook で
  ブロックされる。全ての実装は feature branch → PR で進めること。
- テスト・ビルド確認はユーザーが明示的に指示するまで自動実行しない。
- 実装が一段落したら commit → push → PR 作成まで行い、そこで停止する。
