import { ConfigProvider, theme, Spin } from "antd";
import zhCN from "antd/locale/zh_CN";
import AppLayout from "./components/layout/AppLayout";
import { useConnectionStore } from "./stores/connectionStore";
import { useQueryStore } from "./stores/queryStore";
import SqlEditor from "./components/editor/SqlEditor";
import ResultPanel from "./components/result/ResultPanel";

function AppContent() {
  const { activeConnections } = useConnectionStore();
  const { tabs, activeTabId, addTab, closeTab, setActiveTab, updateTabSql, executeQuery } = useQueryStore();
  const activeTab = tabs.find(t => t.id === activeTabId);

  if (activeConnections.size === 0) {
    return (
      <div style={{ display: "flex", alignItems: "center", justifyContent: "center", flex: 1 }}>
        <div style={{ textAlign: "center" }}>
          <h2 style={{ marginBottom: 16 }}>欢迎使用 HHL-DMA</h2>
          <p style={{ color: "#999" }}>请在左侧面板中添加并连接数据库</p>
        </div>
      </div>
    );
  }

  return (
    <div style={{ display: "flex", flexDirection: "column", flex: 1, overflow: "hidden" }}>
      {/* 标签栏 */}
      <div style={{ display: "flex", background: "#fafafa", borderBottom: "1px solid #f0f0f0", overflow: "auto" }}>
        {tabs.map(tab => (
          <div key={tab.id} onClick={() => setActiveTab(tab.id)}
            style={{
              padding: "6px 16px", cursor: "pointer", borderRight: "1px solid #f0f0f0",
              background: tab.id === activeTabId ? "#fff" : "transparent",
              fontWeight: tab.id === activeTabId ? 600 : 400, fontSize: 13, whiteSpace: "nowrap",
            }}>
            查询 {tabs.indexOf(tab) + 1}
            <span onClick={e => { e.stopPropagation(); closeTab(tab.id); }} style={{ marginLeft: 8, color: "#999" }}>×</span>
          </div>
        ))}
        <div onClick={() => { const firstConnId = Array.from(activeConnections.keys())[0]; if (firstConnId) addTab(firstConnId); }}
          style={{ padding: "6px 12px", cursor: "pointer", color: "#1677ff" }}>+</div>
      </div>

      {/* 主工作区 */}
      <div style={{ flex: 1, display: "flex", flexDirection: "column" }}>
        {activeTab ? (
          <>
            <div style={{ flex: 1, minHeight: 200 }}>
              {activeTab.loading ? (
                <div style={{ display: "flex", alignItems: "center", justifyContent: "center", height: "100%" }}>
                  <Spin tip="执行中..." />
                </div>
              ) : (
                <SqlEditor value={activeTab.sql} onChange={v => updateTabSql(activeTab.id, v)} onExecute={() => executeQuery(activeTab.id)} />
              )}
            </div>
            {activeTab.result && (
              <div style={{ height: 300, borderTop: "1px solid #f0f0f0" }}>
                <ResultPanel result={activeTab.result} onRefresh={() => executeQuery(activeTab.id)} />
              </div>
            )}
            {activeTab.error && (
              <div style={{ padding: 12, color: "#ff4d4f", background: "#fff2f0", borderTop: "1px solid #ffccc7" }}>
                {activeTab.error}
              </div>
            )}
          </>
        ) : (
          <div style={{ display: "flex", alignItems: "center", justifyContent: "center", flex: 1, color: "#999" }}>
            点击 + 新建查询标签
          </div>
        )}
      </div>
    </div>
  );
}

function App() {
  return (
    <ConfigProvider locale={zhCN} theme={{ algorithm: theme.defaultAlgorithm }}>
      <AppLayout>
        <AppContent />
      </AppLayout>
    </ConfigProvider>
  );
}

export default App;
