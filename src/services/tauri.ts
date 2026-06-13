import { invoke } from "@tauri-apps/api/core";
import type {
  ConnectionConfig,
  QueryResult,
  PaginatedResult,
  DatabaseMetadata,
  TableSchema,
  HistoryEntry,
} from "../types";

export async function createConnection(params: {
  name: string;
  dbType: string;
  host: string;
  port: number;
  username: string;
  password: string;
  database: string;
  schema?: string;
  group?: string;
}): Promise<string> {
  return invoke("create_connection", {
    name: params.name,
    dbType: params.dbType,
    host: params.host,
    port: params.port,
    username: params.username,
    password: params.password,
    database: params.database,
    schema: params.schema,
    group: params.group,
  });
}

export async function closeConnection(connId: string): Promise<void> {
  return invoke("close_connection", { connId });
}

export async function testConnection(params: {
  dbType: string;
  host: string;
  port: number;
  username: string;
  password: string;
  database: string;
}): Promise<boolean> {
  return invoke("test_connection", {
    dbType: params.dbType,
    host: params.host,
    port: params.port,
    username: params.username,
    password: params.password,
    database: params.database,
  });
}

export async function listSavedConnections(): Promise<ConnectionConfig[]> {
  return invoke("list_saved_connections");
}

export async function deleteConnection(id: string): Promise<void> {
  return invoke("delete_connection", { id });
}

export async function executeQuery(
  connId: string,
  sql: string
): Promise<QueryResult> {
  return invoke("execute_query", { connId, sql });
}

export async function executeQueryPaginated(
  connId: string,
  sql: string,
  page: number,
  pageSize: number
): Promise<PaginatedResult> {
  return invoke("execute_query_paginated", {
    connId,
    sql,
    page,
    pageSize,
  });
}

export async function cancelQuery(queryId: string): Promise<void> {
  return invoke("cancel_query", { queryId });
}

export async function getQueryHistory(
  connId?: string
): Promise<HistoryEntry[]> {
  return invoke("get_query_history", { connId });
}

export async function getDatabaseMetadata(
  connId: string,
  refresh?: boolean
): Promise<DatabaseMetadata> {
  return invoke("get_database_metadata", { connId, refresh });
}

export async function getTableSchema(
  connId: string,
  table: string
): Promise<TableSchema> {
  return invoke("get_table_schema", { connId, table });
}
