import { Tabs } from "antd";
import ResultTable from "./ResultTable";
import ResultToolbar from "./ResultToolbar";
import type { QueryResult } from "../../types";

interface ResultPanelProps {
  result: QueryResult;
  onRefresh?: () => void;
}

export default function ResultPanel({ result, onRefresh }: ResultPanelProps) {
  const handleCopy = () => {
    const header = result.columns.map(c => c.name).join("\t");
    const rows = result.rows.map(row => result.columns.map(c => String(row[c.name] ?? "NULL")).join("\t"));
    navigator.clipboard.writeText([header, ...rows].join("\n"));
  };

  return (
    <div style={{ height: "100%", display: "flex", flexDirection: "column" }}>
      <ResultToolbar rowCount={result.rows.length} executionTimeMs={result.execution_time_ms} onCopy={handleCopy} onRefresh={onRefresh} />
      <div style={{ flex: 1, overflow: "auto" }}>
        <Tabs size="small" items={[
          { key: "result", label: "结果", children: <ResultTable columns={result.columns} rows={result.rows} /> },
          { key: "message", label: "消息", children: (
            <div style={{ padding: 12, fontSize: 13 }}>
              <div>查询执行成功</div>
              <div>影响行数: {result.affected_rows}</div>
              <div>执行时间: {result.execution_time_ms}ms</div>
            </div>
          )},
        ]} />
      </div>
    </div>
  );
}
