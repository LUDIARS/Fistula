import { createContext, useContext, useReducer, type Dispatch, type ReactElement, type ReactNode } from 'react';
import type { AnalysisResult, AnalyzeOptions } from '../lib/tauri';
const profile = { midiMin: 57, midiMax: 81, calibration: Array.from({ length: 25 }, (_, index) => [57 + index, index / 24] as [number, number]) };
export const defaultOptions: AnalyzeOptions = { frameSize: 2048, hopSize: 512, clarityThreshold: 0.7, rmsThresholdDb: -40, minNoteMs: 80, mergeGapMs: 40, transpose: 0, profile };
export type AppState = { phase: 'import' | 'analyzing' | 'ready'; path: string | null; result: AnalysisResult | null; options: AnalyzeOptions; progress: number; error: string | null };
type Action = { type: 'start'; path: string } | { type: 'progress'; progress: number } | { type: 'ready'; result: AnalysisResult } | { type: 'error'; error: string } | { type: 'options'; options: AnalyzeOptions };
const initial: AppState = { phase: 'import', path: null, result: null, options: defaultOptions, progress: 0, error: null };
function reducer(state: AppState, action: Action): AppState { switch (action.type) { case 'start': return { ...state, phase: 'analyzing', path: action.path, progress: 0, error: null }; case 'progress': return { ...state, progress: action.progress }; case 'ready': return { ...state, phase: 'ready', result: action.result, progress: 1 }; case 'error': return { ...state, phase: 'import', error: action.error }; case 'options': return { ...state, options: action.options }; } }
const StateContext = createContext<AppState | null>(null); const DispatchContext = createContext<Dispatch<Action> | null>(null);
export function AppProvider({ children }: { children: ReactNode }): ReactElement { const [state, dispatch] = useReducer(reducer, initial); return <StateContext.Provider value={state}><DispatchContext.Provider value={dispatch}>{children}</DispatchContext.Provider></StateContext.Provider>; }
export function useApp(): [AppState, Dispatch<Action>] { const state = useContext(StateContext); const dispatch = useContext(DispatchContext); if (!state || !dispatch) throw new Error('AppProvider is required'); return [state, dispatch]; }
