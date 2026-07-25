# Fistula 設計書

楽曲 (音声ファイル) を波形解析し、オタマトーンの運指譜を自動生成して
リズムゲーム風 UI で練習できるデスクトップアプリ。

- Status: **設計 (実装前レビュー中)**
- 前身: 実装指示書 (2026-07-24 scaffold PR #1) を本設計書に改稿
- 関連: [tasks/TODO.md](tasks/TODO.md) — 実装タスク分解

---

## 1. 目的と重視点 (RULE_CODE 第 I 部宣言)

**目的**: 好きな曲の音源から「オタマトーンでどこを押さえるか」を自動抽出し、
音に合わせて練習できるようにする。耳コピ・譜面起こしの手間をゼロにする。

**重視点 (優先順)**:

1. **運指の実用性** — 検出ピッチの生値ではなく「人が押さえられる」ノート列に
   整形する (短すぎるノートの除去、音域外の自動移調、半音量子化)。
2. **練習体験** — 再生と譜面スクロールのズレを知覚限界以下 (±30ms 目標) に保つ。
3. **解析の決定性** — 同じ入力 + 同じパラメータなら同じ譜面。乱数・時刻非依存。
4. **デスクトップ完結** — ネットワーク不要。音源はローカルファイルのみ。

**非目標 (v1)**: マイク入力によるピッチ判定 (採点)、和音対応、
ボーカル分離 (ステム分離)、DAW 連携。→ §12 将来拡張。

## 2. 技術スタック (採用観点つき)

| レイヤー | 技術 | 採用観点 |
|---|---|---|
| フレームワーク | Tauri v2 (Rust) | デスクトップ完結・解析を Rust で高速に。Ars-editor と同系統でノウハウ流用 |
| フロントエンド | React 19 + TypeScript + Vite | scaffold 済み (create-tauri-app react-ts) |
| 音声デコード | symphonia | mp3/wav/flac/ogg/m4a を pure Rust でデコード。ネイティブ依存なし |
| ピッチ検出 | pitch-detection crate (McLeod) | pure Rust でビルド軽量。単音メロディ用途には十分の見込み |
| ピッチ検出 fallback | aubio-rs | pitch-detection の精度不足が**実測で確認された場合のみ**導入 (C ビルド依存が重いため) |
| PDF 出力 | pdf-lib (JS) | フロント側で完結。譜面 Canvas を高解像度ラスタで埋め込む方式 (§9) |

決定方針: fallback 導入は「精度不足の実測」が条件。先回りで両方入れない (§18 依存最小)。

## 3. ドメインモデル

Rust 側が正本。TS へは serde JSON で同形を渡す (camelCase に rename)。

```rust
/// フレーム毎のピッチ推定結果 (解析の生出力)
struct PitchFrame {
    time_sec: f64,      // フレーム中心時刻
    freq_hz: f64,       // 推定基本周波数 (無声フレームは配列から除外)
    clarity: f64,       // 検出信頼度 0..1 (McLeod の clarity)
    rms: f64,           // フレームの音量 (無音判定用)
}

/// 整形後のノートイベント (譜面の最小単位)
struct NoteEvent {
    start_sec: f64,
    duration_sec: f64,
    midi_note: u8,      // 半音量子化後 (移調適用済み)
    cents_offset: f64,  // 量子化前の平均偏差 (参考表示用)
    neck_pos: f64,      // 0.0(開放=最低音)〜1.0(ネック先端=最高音)
}

/// オタマトーン実機プロファイル (製品差をパラメータ化)
struct OtamatoneProfile {
    name: String,
    midi_min: u8,           // 実機で出せる最低音
    midi_max: u8,           // 実機で出せる最高音
    calibration: Vec<(u8, f64)>,  // (midi_note, neck_pos) の較正点列。間は線形補間
}

/// 解析パラメータ (UI から調整可能、全て既定値あり)
struct AnalyzeOptions {
    frame_size: usize,      // 既定 2048
    hop_size: usize,        // 既定 512
    clarity_threshold: f64, // 既定 0.7
    rms_threshold_db: f64,  // 既定 -40dB
    min_note_ms: f64,       // 既定 80ms (これ未満のノートは棄却)
    merge_gap_ms: f64,      // 既定 40ms (同音の隙間はつなぐ)
    transpose: i8,          // 半音単位の移調 (既定 0 = 自動推奨値を UI が設定)
    profile: OtamatoneProfile,
}

/// analyze_audio の戻り値
struct AnalysisResult {
    frames: Vec<PitchFrame>,     // デバッグ/波形オーバレイ表示用
    notes: Vec<NoteEvent>,       // 譜面本体
    duration_sec: f64,
    sample_rate: u32,
    suggested_transpose: i8,     // 音域フィットの自動推奨 (§6)
    out_of_range_count: usize,   // 移調後も音域外だったノート数 (UI 警告用)
}
```

既定プロファイル: 標準オタマトーン相当として **midi_min=57 (A3) 〜 midi_max=81 (A5)、
較正点は半音等間隔 (MIDI 線形)** を仮置きする。実機のネック上の音程配置は
非線形の可能性があるため、較正点列で表現しておき実測後に差し替える
(コードは線形補間のみ知っていればよい)。

## 4. 解析パイプライン (Rust)

```
音声ファイル
  │ symphonia デコード (対応外コーデック→即エラー。無言スキップ禁止 §7.1)
  ▼
f32 PCM (ステレオはミックスダウンで mono 化。リサンプルはしない=元 SR のまま処理)
  │ フレーム分割: frame_size=2048, hop=512, Hann 窓
  ▼
pitch-detection McLeodDetector → freq + clarity
  │ フィルタ: clarity < 0.7 or rms < -40dB のフレームを無声化
  ▼
PitchFrame 列
  │ 後処理 (順序固定):
  │  a. メディアンフィルタ (窓 5 フレーム) — 単発の外れ値除去
  │  b. オクターブ跳び補正 — 前後フレームと 12±1 半音差の孤立区間を寄せる
  │  c. 半音量子化 (ヒステリシス: 現ノートから ±0.6 半音超えたら遷移)
  │  d. セグメンテーション — 同一半音の連続を 1 ノートへ。
  │     merge_gap_ms 以下の同音途切れは結合、min_note_ms 未満は棄却
  ▼
NoteEvent 列 (移調適用 → ネック位置マッピング §5)
```

- 処理は全て純関数のチェーンで、`AnalyzeOptions` 以外の外部状態を持たない (§16 決定性)。
- 長尺ファイル対策: 解析中は Tauri event `fistula://analyze-progress` で進捗 (0..1) を emit。
- 失敗系: ファイル不在 / デコード不能 / 全フレーム無声 (「メロディが検出できない」) は
  それぞれ区別されたエラーで返す (§9 fail-fast)。

## 5. オタマトーン運指マッピング

- `midi_note` → `neck_pos`: プロファイルの較正点列を線形補間。範囲外は clamp せず
  **範囲外フラグ付きで返す** (UI で赤表示。無言で丸めない)。
- 譜面の縦軸はネック位置 (`neck_pos`)。演奏 UI・エクスポートとも同じマッピングを使う。

## 6. 自動移調 (音域フィット)

曲のメロディがオタマトーンの音域に収まらないことは常態なので、
**オクターブ単位の自動移調推奨**を解析結果に含める:

- 候補 shift ∈ {-24, -12, 0, +12, +24} について音域外ノート数を数え、
  最小の shift を `suggested_transpose` とする (同数なら絶対値の小さい方)。
- UI は初回解析時に推奨値を transpose に適用し、ユーザは ±12 半音の範囲で微調整可
  (半音単位。カラオケ的キー変更)。変更時は再解析ではなく **ノート列の再マッピングのみ**
  (frames は不変なので高速)。

## 7. Tauri command API

| command | シグネチャ | 説明 |
|---|---|---|
| `analyze_audio` | `(path: String, options: AnalyzeOptions) -> AnalysisResult` | 解析本体。進捗は event で |
| `remap_notes` | `(frames: 省略) -> Vec<NoteEvent>` | **設けない**。移調・プロファイル変更の再マッピングは TS 側に同じ純関数を持たず、`analyze_audio` の frames を入力に Rust `remap` を呼ぶ… ではなく、**セグメント済み中間 (量子化前の連続ピッチ区間) を AnalysisResult に含め、TS 側で移調＋位置マッピングを行う** (ロジックは軽い算術のみ。二重実装は移調とマッピングの 2 関数だけに限定し、テストで両者の一致を担保) |

→ 簡潔化のため v1 の command は `analyze_audio` 1 本。音声再生はフロントの
`<audio>` + `convertFileSrc` で行い、Rust 側は再生に関与しない。

## 8. フロントエンド設計

### 画面フロー

```
[Import 画面] --ファイル選択/D&D--> [解析中 (進捗バー)] --> [Player 画面]
                                                              ├─ 演奏ビュー (スクロール譜面)
                                                              ├─ 調整パネル (移調/しきい値→再解析)
                                                              └─ エクスポート (PNG/PDF)
```

### 演奏ビュー (リズムゲーム風)

- **横軸=時間、縦軸=ネック位置**。画面左寄りに固定の判定線 (playhead)。
  ノートバーが右から左へ流れる。判定線上に来た瞬間が押さえるタイミング。
- 縦軸には半音ごとのガイド線 + 音名ラベル (C4 等 / ドレミ切替)。
- 描画は Canvas 2D + requestAnimationFrame。ノート数は高々数千なので
  毎フレーム可視範囲のみ描画で足りる (仮想化不要)。
- 再生: `<audio src={convertFileSrc(path)}>`。同期は
  「rAF 毎に `audio.currentTime` を読む」を唯一の時刻源とする
  (自前クロックとの二重管理をしない。§ズレ対策はこの一本化で足りる)。
- 練習支援: 再生速度 0.5〜1.0x (`playbackRate` + `preservesPitch`)、
  区間ループ (A-B リピート)、シークバー。

### 状態管理

- 外部ライブラリなし。`useReducer` + Context で
  `{ phase: 'import'|'analyzing'|'ready', result, playerState, options }` を持つ。
  規模的に十分で、依存を増やさない (§18)。

## 9. エクスポート

- **PNG**: 譜面を段組 (1 段 = N 秒、既定 10 秒) で OffscreenCanvas に 2x スケール描画
  → 1 枚の縦長 PNG。
- **PDF**: 同じ段組描画を A4 ページ単位に割り、pdf-lib でページごとに
  PNG embed。ベクタ描画はしない (演奏ビューと描画コードを共有するため。
  印刷解像度は 2x ラスタで実用十分)。
- 保存は Tauri の save dialog (`@tauri-apps/plugin-dialog`) + fs plugin。

## 10. モジュール構成 (SRP)

```
src-tauri/src/
  main.rs / lib.rs        # Tauri 起動・command 登録のみ
  commands.rs             # analyze_audio command (入口検証 + 進捗 emit)
  model.rs                # §3 のドメイン型 + serde
  audio/decode.rs         # symphonia デコード + mono ミックスダウン
  analysis/framing.rs     # フレーム分割 + 窓関数
  analysis/pitch.rs       # McLeod ピッチ推定 + clarity/rms フィルタ
  analysis/postprocess.rs # メディアン/オクターブ補正/量子化/セグメンテーション
  analysis/transpose.rs   # 音域フィット推奨 (§6)
  mapping/otamatone.rs    # プロファイル + neck_pos マッピング

src/
  app/                    # ルート・画面フロー・Context/Reducer
  features/import/        # ファイル選択・D&D・解析呼び出し
  features/player/        # 演奏ビュー (Canvas 描画・再生同期・ループ/速度)
  features/tuning/        # 移調・パラメータ調整パネル
  features/export/        # PNG/PDF エクスポート
  lib/score/              # ノート再マッピング・座標変換 (純関数、テスト対象)
  lib/tauri/              # invoke/event の薄いラッパ (型付け)
```

## 11. テスト方針

- **Rust**: 合成波形 fixture (正弦波・周波数スイープ・休符入りメロディを
  コード生成) で決定的にユニットテスト。
  - pitch: 440Hz 正弦波 → A4、スイープ → 単調増加
  - postprocess: 外れ値 1 フレーム除去、短ノート棄却、gap 結合
  - transpose: 音域外曲の推奨 shift
  - mapping: 較正点補間・範囲外フラグ
- **TS**: `lib/score/` の座標変換・再マッピングを vitest。Rust `mapping` と
  同一入力→同一出力のゴールデンテーブルで二重実装の一致を担保 (§7 の注記)。
- 実音源での精度確認は手動 (ユーザ確認フェーズ)。CI は合成 fixture のみ。

## 12. 将来拡張 (v1 スコープ外)

- マイク入力のリアルタイムピッチ判定 (採点・チューナー)
- ボーカル/メロディのステム分離 (現状は「メロディが主役の音源を入れる」運用)
- 実機較正ウィザード (マイクで実機音を拾って calibration 点列を作る)
- aubio-rs 導入 (pitch-detection の精度不足が実測された場合)

## 13. 決定ログ

| # | 決定 | 代替案 | 理由 (AI学習量/作業コスト/目的達成度/主目的一致) |
|---|---|---|---|
| D1 | ピッチ検出は pitch-detection 単独で開始 | aubio-rs 併用比較 | ビルド依存最小で最速。単音メロディなら McLeod で達成度十分の見込み。不足の実測が出た時のみ fallback (§2) |
| D2 | 描画は Canvas 2D | SVG / WebGL | 数千ノートのスクロールに十分・実装コスト最小。SVG は DOM 更新が同期精度目標に不利 |
| D3 | PDF はラスタ embed | pdf-lib ベクタ描画 | 演奏ビューと描画コード共有で実装半減。印刷品質は 2x で足りる |
| D4 | 時刻源は audio.currentTime 一本 | 自前クロック補正 | 二重管理はズレの温床。要件 ±30ms は currentTime 直読みで達成可 |
| D5 | 再マッピングは TS 側軽量関数 | 毎回 Rust 呼び出し | 移調スライダの即時反映 (ドラッグ中連続更新) を IPC 往復なしで実現。一致はゴールデンテストで担保 |
| D6 | 状態管理は React 標準のみ | zustand 等 | 画面 3 つ規模に外部依存不要 (§18) |

## 14. 開発フロー上の注意

- `main` 直 push 禁止。feature branch → PR。AI 実装は 1 PR に集約。
- テスト・ビルドの実行はユーザ指示があるまで自動実行しない
  (worktree/セッションでの動作テスト禁止ルール準拠)。
- `LUDIARS/PROJECT-CODES.md` への登録は別 PR (LUDIARS リポ側)。略称案: **`Fi`**
  (F 系既存: Fg/Fd/Fm/Fa と非衝突)。
