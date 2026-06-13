import { Table } from "antd";
import type { ColumnDef } from "../../types";

interface ResultTableProps {
  columns: ColumnDef[];
  rows: Record<string, unknown>[];
}

export default function ResultTable({ columns, rows }: ResultTableProps) {
  const tableColumns = columns.map((col) => ({
    title: col.name,
    dataIndex: col.name,
    key: col.name,
    ellipsis: true,
    width: 150,
    render: (value: unknown) => {
      if (value === null || value === undefined) return <span style={{ color: "#bfbfbf" }}>NULL</span>;
      return String(value);
    },
  }));

  return (
    <Table
      columns={tableColumns}
      dataSource={rows.map((row, i) => ({ ...row, _key: i }))}
      rowKey="_key"
      size="small"
      pagination={{ pageSize: 100, showSizeChanger: true, pageSizeOptions: ["50", "100", "200", "500"], showTotal: (total) => `共 ${total} 行` }}
      scroll={{ x: "max-content", y: "calc(100% - 40px)" }}
      style={{ fontSize: 13 }}
    />
  );
}
