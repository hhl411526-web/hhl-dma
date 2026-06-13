import { useConnectionStore } from "../../stores/connectionStore";
import { useQueryStore } from "../../stores/queryStore";

export default function StatusBar() {
  const { activeConnections } = useConnectionStore();
  const { tabs, activeTabId } = useQueryStore();
  const activeTab = tabs.find(t => t.id === activeTabId);
  const activeConn = activeTab ? activeConnections.get(activeTab.connId) : undefined;

  return (
    <div style={{ height: 24, background: "#1677ff", color: "#fff", display: "flex", alignItems: "center", padding: "0 12px", fontSize: 12, gap: 16 }}>
      {activeConn && <span>已连接: {activeConn.name}@{activeConn.host}</span>}
      {activeTab?.result && (<><span>耗时: {activeTab.result.execution_time_ms}ms</span><span>行数: {activeTab.result.rows.length}</span></>)}
      <span style={{ marginLeft: "auto" }}>HHL-DMA v0.1.0</span>
    </div>
  );
}
