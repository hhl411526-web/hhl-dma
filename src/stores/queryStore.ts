import { create } from "zustand";
import type { QueryResult, HistoryEntry } from "../types";
import * as tauri from "../services/tauri";

interface QueryTab {
  id: string;
  connId: string;
  sql: string;
  result?: QueryResult;
  loading: boolean;
  error?: string;
}

interface QueryState {
  tabs: QueryTab[];
  activeTabId: string | null;
  history: HistoryEntry[];

  addTab: (connId: string) => string;
  closeTab: (tabId: string) => void;
  setActiveTab: (tabId: string) => void;
  updateTabSql: (tabId: string, sql: string) => void;
  executeQuery: (tabId: string) => Promise<void>;
  cancelQuery: (queryId: string) => Promise<void>;
  loadHistory: (connId?: string) => Promise<void>;
}

let tabCounter = 0;

export const useQueryStore = create<QueryState>((set, get) => ({
  tabs: [],
  activeTabId: null,
  history: [],

  addTab: (connId) => {
    tabCounter++;
    const tabId = `tab-${tabCounter}`;
    const tab: QueryTab = { id: tabId, connId, sql: "", loading: false };
    set((state) => ({ tabs: [...state.tabs, tab], activeTabId: tabId }));
    return tabId;
  },

  closeTab: (tabId) => {
    set((state) => {
      const newTabs = state.tabs.filter((t) => t.id !== tabId);
      const newActiveId =
        state.activeTabId === tabId
          ? newTabs[newTabs.length - 1]?.id || null
          : state.activeTabId;
      return { tabs: newTabs, activeTabId: newActiveId };
    });
  },

  setActiveTab: (tabId) => set({ activeTabId: tabId }),

  updateTabSql: (tabId, sql) => {
    set((state) => ({
      tabs: state.tabs.map((t) => (t.id === tabId ? { ...t, sql } : t)),
    }));
  },

  executeQuery: async (tabId) => {
    const tab = get().tabs.find((t) => t.id === tabId);
    if (!tab || !tab.sql.trim()) return;

    set((state) => ({
      tabs: state.tabs.map((t) =>
        t.id === tabId ? { ...t, loading: true, error: undefined, result: undefined } : t
      ),
    }));

    try {
      const result = await tauri.executeQuery(tab.connId, tab.sql);
      set((state) => ({
        tabs: state.tabs.map((t) =>
          t.id === tabId ? { ...t, loading: false, result } : t
        ),
      }));
    } catch (e) {
      set((state) => ({
        tabs: state.tabs.map((t) =>
          t.id === tabId ? { ...t, loading: false, error: String(e) } : t
        ),
      }));
    }
  },

  cancelQuery: async (queryId) => {
    await tauri.cancelQuery(queryId);
  },

  loadHistory: async (connId) => {
    try {
      const history = await tauri.getQueryHistory(connId);
      set({ history });
    } catch (e) {
      console.error("Failed to load query history:", e);
    }
  },
}));
