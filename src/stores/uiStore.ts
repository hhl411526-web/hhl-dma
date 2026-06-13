import { create } from "zustand";

interface UIState {
  sidebarWidth: number;
  resultPanelHeight: number;
  sidebarCollapsed: boolean;
  theme: "light" | "dark";
  setSidebarWidth: (width: number) => void;
  setResultPanelHeight: (height: number) => void;
  toggleSidebar: () => void;
  setTheme: (theme: "light" | "dark") => void;
}

export const useUIStore = create<UIState>((set) => ({
  sidebarWidth: 260,
  resultPanelHeight: 300,
  sidebarCollapsed: false,
  theme: "light",
  setSidebarWidth: (width) => set({ sidebarWidth: width }),
  setResultPanelHeight: (height) => set({ resultPanelHeight: height }),
  toggleSidebar: () => set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed })),
  setTheme: (theme) => set({ theme }),
}));
