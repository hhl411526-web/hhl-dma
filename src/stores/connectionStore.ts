import { create } from "zustand";
import type { ConnectionConfig, DatabaseMetadata } from "../types";
import * as tauri from "../services/tauri";

interface ConnectionState {
  connections: ConnectionConfig[];
  activeConnections: Map<string, ConnectionConfig>;
  metadata: Map<string, DatabaseMetadata>;
  loading: boolean;
  error: string | null;

  loadConnections: () => Promise<void>;
  addConnection: (params: {
    name: string;
    dbType: string;
    host: string;
    port: number;
    username: string;
    password: string;
    database: string;
    schema?: string;
    group?: string;
  }) => Promise<string>;
  removeConnection: (id: string) => Promise<void>;
  connect: (config: ConnectionConfig) => Promise<string>;
  disconnect: (connId: string) => Promise<void>;
  testConnection: (params: {
    dbType: string;
    host: string;
    port: number;
    username: string;
    password: string;
    database: string;
  }) => Promise<boolean>;
  loadMetadata: (connId: string, refresh?: boolean) => Promise<void>;
}

export const useConnectionStore = create<ConnectionState>((set, get) => ({
  connections: [],
  activeConnections: new Map(),
  metadata: new Map(),
  loading: false,
  error: null,

  loadConnections: async () => {
    set({ loading: true, error: null });
    try {
      const connections = await tauri.listSavedConnections();
      set({ connections, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  addConnection: async (params) => {
    set({ loading: true, error: null });
    try {
      const connId = await tauri.createConnection(params);
      await get().loadConnections();
      set({ loading: false });
      return connId;
    } catch (e) {
      set({ error: String(e), loading: false });
      throw e;
    }
  },

  removeConnection: async (id) => {
    try {
      await tauri.deleteConnection(id);
      await get().loadConnections();
    } catch (e) {
      set({ error: String(e) });
    }
  },

  connect: async (config) => {
    const connId = await tauri.createConnection({
      name: config.name,
      dbType: config.db_type,
      host: config.host,
      port: config.port,
      username: config.username,
      password: config.password,
      database: config.database,
      schema: config.schema,
      group: config.group,
    });
    const newActive = new Map(get().activeConnections);
    newActive.set(connId, config);
    set({ activeConnections: newActive });
    return connId;
  },

  disconnect: async (connId) => {
    await tauri.closeConnection(connId);
    const newActive = new Map(get().activeConnections);
    newActive.delete(connId);
    const newMetadata = new Map(get().metadata);
    newMetadata.delete(connId);
    set({ activeConnections: newActive, metadata: newMetadata });
  },

  testConnection: async (params) => {
    return tauri.testConnection(params);
  },

  loadMetadata: async (connId, refresh) => {
    try {
      const metadata = await tauri.getDatabaseMetadata(connId, refresh);
      const newMetadata = new Map(get().metadata);
      newMetadata.set(connId, metadata);
      set({ metadata: newMetadata });
    } catch (e) {
      set({ error: String(e) });
    }
  },
}));
