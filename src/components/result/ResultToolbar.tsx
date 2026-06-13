import { Button, Space, Typography, Tooltip } from "antd";
import { CopyOutlined, ReloadOutlined } from "@ant-design/icons";

interface ResultToolbarProps {
  rowCount: number;
  executionTimeMs: number;
  onExport?: () => void;
  onCopy?: () => void;
  onRefresh?: () => void;
}

export default function ResultToolbar({ rowCount, executionTimeMs, onCopy, onRefresh }: ResultToolbarProps) {
  return (
    <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "4px 12px", background: "#fafafa", borderBottom: "1px solid #f0f0f0" }}>
      <Space size={16}>
        <Typography.Text type="secondary" style={{ fontSize: 12 }}>{rowCount} 行</Typography.Text>
        <Typography.Text type="secondary" style={{ fontSize: 12 }}>耗时 {executionTimeMs}ms</Typography.Text>
      </Space>
      <Space size={4}>
        <Tooltip title="复制结果"><Button type="text" size="small" icon={<CopyOutlined />} onClick={onCopy} /></Tooltip>
        <Tooltip title="刷新"><Button type="text" size="small" icon={<ReloadOutlined />} onClick={onRefresh} /></Tooltip>
      </Space>
    </div>
  );
}
