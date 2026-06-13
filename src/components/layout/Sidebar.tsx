import { Tree, Button, message } from "antd";
import { PlusOutlined, DatabaseOutlined } from "@ant-design/icons";
import { useConnectionStore } from "../../stores/connectionStore";
import { useQueryStore } from "../../stores/queryStore";
import { useEffect, useState } from "react";
import ConnectionDialog from "../connection/ConnectionDialog";

export default function Sidebar() {
  const { connections, activeConnections, metadata, loadConnections, connect, disconnect, loadMetadata } = useConnectionStore();
  const { addTab } = useQueryStore();
  const [showNewConnection, setShowNewConnection] = useState(false);

  useEffect(() => { loadConnections(); }, [loadConnections]);

  const handleConnect = async (config: typeof connections[0]) => {
    try {
      const connId = await connect(config);
      await loadMetadata(connId);
      message.success(`已连接: ${config.name}`);
    } catch (e) {
      message.error(`连接失败: ${String(e)}`);
    }
  };

  const handleDisconnect = async (connId: string) => {
    try {
      await disconnect(connId);
      message.success("已断开连接");
    } catch (e) {
      message.error(`断开失败: ${String(e)}`);
    }
  };

  const buildTreeData = () => {
    return Array.from(activeConnections.entries()).map(([connId, config]) => {
      const meta = metadata.get(connId);
      return {
        key: connId,
        title: config.name,
        icon: <DatabaseOutlined />,
        children: meta ? [
          {
            key: `${connId}-tables`,
            title: `表 (${meta.tables.filter(t => t.table_type === "BASE TABLE").length})`,
            children: meta.tables
              .filter(t => t.table_type === "BASE TABLE")
              .map(t => ({ key: `${connId}-table-${t.name}`, title: t.name, isLeaf: true })),
          },
          {
            key: `${connId}-views`,
            title: `视图 (${meta.views.length})`,
            children: meta.views.map(v => ({ key: `${connId}-view-${v.name}`, title: v.name, isLeaf: true })),
          },
        ] : [],
      };
    });
  };

  return (
    <div style={{ height: "100%", display: "flex", flexDirection: "column" }}>
      <div style={{ padding: "8px 12px", borderBottom: "1px solid #f0f0f0", display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <span style={{ fontWeight: 600, fontSize: 13 }}>连接</span>
        <Button type="text" size="small" icon={<PlusOutlined />} onClick={() => setShowNewConnection(true)} />
      </div>
      <div style={{ flex: 1, overflow: "auto", padding: "4px 8px" }}>
        <Tree treeData={buildTreeData()} showIcon defaultExpandAll />
      </div>
      <div style={{ padding: "8px 12px", borderTop: "1px solid #f0f0f0" }}>
        {connections.map((conn) => {
          const isActive = Array.from(activeConnections.values()).some(ac => ac.id === conn.id);
          const activeConnId = Array.from(activeConnections.entries()).find(([, v]) => v.id === conn.id)?.[0];
          return (
            <div key={conn.id} style={{ display: "flex", justifyContent: "space-between", alignItems: "center", padding: "4px 0" }}>
              <span style={{ fontSize: 12 }}>{conn.name}</span>
              {isActive && activeConnId ? (
                <div>
                  <Button type="link" size="small" onClick={() => addTab(activeConnId)}>查询</Button>
                  <Button type="link" size="small" danger onClick={() => handleDisconnect(activeConnId)}>断开</Button>
                </div>
              ) : (
                <Button type="link" size="small" onClick={() => handleConnect(conn)}>连接</Button>
              )}
            </div>
          );
        })}
      </div>
      <ConnectionDialog open={showNewConnection} onClose={() => setShowNewConnection(false)} />
    </div>
  );
}
