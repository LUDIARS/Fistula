import type { ReactElement } from "react";
import "./App.css";
import { AppProvider, useApp } from "./app/state";
import { ImportScreen } from "./features/import/ImportScreen";
import { PlayerScreen } from "./features/player/PlayerScreen";
import { TuningPanel } from "./features/tuning/TuningPanel";
import { ExportButtons } from "./features/export/ExportButtons";
function Screen(): ReactElement {
  const [state] = useApp();
  if (state.phase === "analyzing")
    return (
      <main>
        <h1>解析中</h1>
        <progress value={state.progress} max="1" />{" "}
        <span>{Math.round(state.progress * 100)}%</span>
      </main>
    );
  if (state.phase === "ready")
    return (
      <>
        <PlayerScreen />
        <TuningPanel />
        <ExportButtons />
      </>
    );
  return <ImportScreen />;
}
export default function App(): ReactElement {
  return (
    <AppProvider>
      <Screen />
    </AppProvider>
  );
}
