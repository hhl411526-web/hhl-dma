import { useState } from "react";
import { Button, Input, Spin, Tree, message } from "antd";
import { PlayCircleOutlined } from "@ant-design/icons";
import type { ExplainResult } from "../../types";

interface ExplainPanelProps {
  connId: string;
}

export default function ExplainPanel({ connId: _connId }: ExplainPanelProps) {
  const [sql, setSql] = useState("");
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<ExplainResult | null>(null);

  const handleExplain = async () => {
    if (!sql.trim()) {
      message.warning("请输入 SQL 语句");
      return;
    }
    setLoading(true);
    try {
      // Will call Tauri explain command when wired up
      setResult({
        plan: "执行计划将在连接数据库后显示",
        formatted: undefined,
        cost: undefined,
      });
    } catch (e) {
      message.error(`分析失败: ${String(e)}`);
    } finally {
      setLoading(false);
    }
  };

  const parsePlanToTree = (plan: string) => {
    const lines = plan.split("\n").filter(l => l.trim());
    return lines.map((line, i) => ({
      key: String(i),
      title: line,
      children: [],
    }));
  };

  return (
    <div style={{ padding: 16, height: "100%", display: "flex", flexDirection: "column" }}>
      <div style={{ marginBottom: 12, display: "flex", gap: 8 }}>
        <Input.TextArea value={sql} onChange={e => setSql(e.target.value)} rows={3} placeholder="输入 SQL 语句查看执行计划" style={{ flex: 1 }} />
        <Button type="primary" icon={<PlayCircleOutlined />} onClick={handleExplain} loading={loading}>分析</Button>
      </div>
      {loading && <Spin />}
      {result && (
        <div style={{ flex: 1, overflow: "auto" }}>
          {result.cost !== undefined && (
            <div style={{ marginBottom: 8, color: "#1677ff" }}>预估成本: {result.cost.toFixed(2)}</div>
          )}
          <Tree treeData={parsePlanToTree(result.plan)} defaultExpandAll />
        </div>
      )}
    </div>
  );
}
