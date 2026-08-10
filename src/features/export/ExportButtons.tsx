import type { ReactElement } from "react";
import { save } from "@tauri-apps/plugin-dialog";
import { writeFile } from "@tauri-apps/plugin-fs";
import { PDFDocument } from "pdf-lib";
import { useApp } from "../../app/state";
import { useScoreNotes } from "../../app/useScoreNotes";
import type { ScoreNote } from "../../lib/score";

/** 譜面の段組: 1 段 = ROW_SECONDS 秒を ROW_WIDTH x ROW_HEIGHT px (2x スケール) で描く。 */
const ROW_SECONDS = 10;
const ROW_WIDTH = 1600;
const ROW_HEIGHT = 400;
/** A4 (pt) と描画マージン。 */
const PAGE_WIDTH = 595;
const PAGE_HEIGHT = 842;
const PAGE_MARGIN = 20;

function renderScoreCanvas(
  notes: ScoreNote[],
  duration: number,
): OffscreenCanvas {
  const rows = Math.max(1, Math.ceil(duration / ROW_SECONDS));
  const canvas = new OffscreenCanvas(ROW_WIDTH, rows * ROW_HEIGHT);
  const context = canvas.getContext("2d");
  if (!context) throw new Error("Canvas 2D is unavailable");
  context.fillStyle = "white";
  context.fillRect(0, 0, canvas.width, canvas.height);
  context.fillStyle = "#1e3a8a";
  notes.forEach((note) => {
    const row = Math.floor(note.startSec / ROW_SECONDS);
    const x = ((note.startSec % ROW_SECONDS) / ROW_SECONDS) * 1500 + 50;
    const y = row * ROW_HEIGHT + (1 - note.neckPos) * 320 + 40;
    context.fillRect(
      x,
      y,
      Math.max(4, note.durationSec * (1500 / ROW_SECONDS)),
      18,
    );
  });
  return canvas;
}

async function canvasToPngBytes(canvas: OffscreenCanvas): Promise<Uint8Array> {
  const blob = await canvas.convertToBlob({ type: "image/png" });
  return new Uint8Array(await blob.arrayBuffer());
}

/** 全段キャンバスを A4 ページ単位に分割し、各ページへラスタ embed する (design §9)。 */
async function buildPdf(
  notes: ScoreNote[],
  duration: number,
): Promise<Uint8Array> {
  const full = renderScoreCanvas(notes, duration);
  const totalRows = full.height / ROW_HEIGHT;
  const scale = (PAGE_WIDTH - PAGE_MARGIN * 2) / ROW_WIDTH;
  const rowsPerPage = Math.max(
    1,
    Math.floor((PAGE_HEIGHT - PAGE_MARGIN * 2) / (ROW_HEIGHT * scale)),
  );
  const pdf = await PDFDocument.create();
  for (let firstRow = 0; firstRow < totalRows; firstRow += rowsPerPage) {
    const rows = Math.min(rowsPerPage, totalRows - firstRow);
    const slice = new OffscreenCanvas(ROW_WIDTH, rows * ROW_HEIGHT);
    const context = slice.getContext("2d");
    if (!context) throw new Error("Canvas 2D is unavailable");
    context.drawImage(
      full,
      0,
      firstRow * ROW_HEIGHT,
      ROW_WIDTH,
      rows * ROW_HEIGHT,
      0,
      0,
      ROW_WIDTH,
      rows * ROW_HEIGHT,
    );
    const image = await pdf.embedPng(await canvasToPngBytes(slice));
    const page = pdf.addPage([PAGE_WIDTH, PAGE_HEIGHT]);
    const height = rows * ROW_HEIGHT * scale;
    page.drawImage(image, {
      x: PAGE_MARGIN,
      y: PAGE_HEIGHT - PAGE_MARGIN - height,
      width: ROW_WIDTH * scale,
      height,
    });
  }
  return pdf.save();
}

export function ExportButtons(): ReactElement {
  const [state] = useApp();
  const notes = useScoreNotes();

  const exportPng = async (): Promise<void> => {
    if (!state.result) return;
    const path = await save({
      defaultPath: "fistula-score.png",
      filters: [{ name: "PNG", extensions: ["png"] }],
    });
    if (!path) return;
    const canvas = renderScoreCanvas(notes, state.result.durationSec);
    await writeFile(path, await canvasToPngBytes(canvas));
  };

  const exportPdf = async (): Promise<void> => {
    if (!state.result) return;
    const path = await save({
      defaultPath: "fistula-score.pdf",
      filters: [{ name: "PDF", extensions: ["pdf"] }],
    });
    if (!path) return;
    await writeFile(path, await buildPdf(notes, state.result.durationSec));
  };

  return (
    <section>
      <button onClick={() => void exportPng()}>PNG 保存</button>
      <button onClick={() => void exportPdf()}>PDF 保存</button>
    </section>
  );
}
