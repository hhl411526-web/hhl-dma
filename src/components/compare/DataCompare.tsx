import { useState } from "react";
import { Select, Button, Table, message } from "antd";
import { SwapOutlined } from "@ant-design/icons";
import type { DatabaseMetadata } from "../../types";

interface DataCompareProps {
  connId: string;
  metadata: DatabaseMetadata;
}

export default function DataCompare({ connId: _connId, metadata }: DataCompareProps) {
  const [sourceTable, setSourceTable] = useState<string>();
  const [targetTable, setTargetTable] = useState<string>();
  const [comparing, setComparing] = useState(false);
  const [diffResult, setDiffResult] = useState<{ type: string; source: string; target: string }[]>([]);

  const tables = metadata.tables
    .filter(t => t.table_type === "BASE TABLE")
    .map(t => ({ label: t.name, value: t.name }));

  const handleCompare = async () => {
    if (!sourceTable || !targetTable) {
      message.warning("请选择源表和目标表");
      return;
    }
    setComparing(true);
    try {
      // Will call Tauri compare command when wired up
      setDiffResult([
        { type: "仅源表", source: "column_a", target: "-" },
        { type: "仅目标表", source: "-", target: "column_b" },
        { type: "类型不同", source: "VARCHAR(255)", target: "TEXT" },
      ]);
    } catch (e) {
      message.error(`对比失败: ${String(e)}`);
    } finally {
      setComparing(false);
    }
  };

  const handleGenerateSyncScript = () => {
    message.info("同步脚本生成功能将在后续版本实现");
  };

  return (
    <div style={{ padding: 16 }}>
      <div style={{ marginBottom: 16, display: "flex", gap: 16, alignItems: "center" }}>
        <Select style={{ width: 200 }} placeholder="源表" options={tables} value={sourceTable} onChange={setSourceTable} />
        <SwapOutlined style={{ color: "#1677ff" }} />
        <Select style={{ width: 200 }} placeholder="目标表" options={tables} value={targetTable} onChange={setTargetTable} />
        <Button type="primary" onClick={handleCompare} loading={comparing}>对比</Button>
      </div>

      {diffResult.length > 0 && (
        <>
          <Table
            columns={[
              { title: "差异类型", dataIndex: "type", width: 120 },
              { title: "源表", dataIndex: "source", width: 200 },
              { title: "目标表", dataIndex: "target", width: 200 },
            ]}
            dataSource={diffResult.map((r, i) => ({ key: i, ...r }))}
            size="small"
            pagination={false}
          />
          <div style={{ marginTop: 12, textAlign: "right" }}>
            <Button onClick={handleGenerateSyncScript}>生成同步脚本</Button>
          </div>
        </>
      )}
    </div>
  );
}
