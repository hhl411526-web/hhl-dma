export type DatabaseType =
  | "mysql"
  | "postgresql"
  | "sqlite"
  | "oracle"
  | "sqlserver"
  | "dm"
  | "kingbasees"
  | "oscar"
  | "gbase"
  | "odbc";

export interface ConnectionConfig {
  id: string;
  name: string;
  db_type: DatabaseType;
  host: string;
  port: number;
  username: string;
  password: string;
  database: string;
  schema?: string;
  options: Record<string, string>;
  group?: string;
  created_at: string;
  updated_at: string;
}

export interface ColumnDef {
  name: string;
  data_type: string;
  nullable: boolean;
}

export interface QueryResult {
  columns: ColumnDef[];
  rows: Record<string, unknown>[];
  affected_rows: number;
  execution_time_ms: number;
}

export interface PaginatedResult {
  columns: ColumnDef[];
  rows: Record<string, unknown>[];
  page: number;
  page_size: number;
  total_rows: number;
  total_pages: number;
  execution_time_ms: number;
}

export interface TableInfo {
  catalog?: string;
  schema?: string;
  name: string;
  table_type: string;
  comment?: string;
  row_count?: number;
}

export interface ViewInfo {
  catalog?: string;
  schema?: string;
  name: string;
  definition?: string;
}

export interface DatabaseMetadata {
  catalogs: { name: string }[];
  schemas: { catalog?: string; name: string }[];
  tables: TableInfo[];
  views: ViewInfo[];
}

export interface ColumnInfo {
  name: string;
  data_type: string;
  nullable: boolean;
  default_value?: string;
  is_primary_key: boolean;
  is_auto_increment: boolean;
  comment?: string;
  ordinal_position: number;
  max_length?: number;
  precision?: number;
  scale?: number;
}

export interface IndexInfo {
  name: string;
  columns: string[];
  is_unique: boolean;
  is_primary: boolean;
  index_type?: string;
}

export interface ConstraintInfo {
  name: string;
  constraint_type: string;
  columns: string[];
  referenced_table?: string;
  referenced_columns?: string[];
}

export interface TableSchema {
  table_name: string;
  columns: ColumnInfo[];
  indexes: IndexInfo[];
  constraints: ConstraintInfo[];
}

export interface HistoryEntry {
  id: string;
  connection_id: string;
  sql: string;
  executed_at: string;
  execution_time_ms: number;
  affected_rows: number;
  status: "Success" | { Error: string };
}

export interface ExplainResult {
  plan: string;
  formatted?: string;
  cost?: number;
}

export interface DriverInfo {
  name: string;
  version: string;
  db_type: DatabaseType;
  description: string;
}
