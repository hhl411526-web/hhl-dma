import { useState, useCallback } from "react";
import { Layout } from "antd";
import Sidebar from "./Sidebar";
import StatusBar from "./StatusBar";
import { useUIStore } from "../../stores/uiStore";

const { Sider, Content } = Layout;

interface AppLayoutProps {
  children: React.ReactNode;
}

export default function AppLayout({ children }: AppLayoutProps) {
  const { sidebarWidth, sidebarCollapsed, setSidebarWidth } = useUIStore();
  const [resizing, setResizing] = useState(false);

  const handleMouseDown = useCallback(() => {
    setResizing(true);
    const handleMouseMove = (e: MouseEvent) => {
      const newWidth = Math.max(180, Math.min(500, e.clientX));
      setSidebarWidth(newWidth);
    };
    const handleMouseUp = () => {
      setResizing(false);
      document.removeEventListener("mousemove", handleMouseMove);
      document.removeEventListener("mouseup", handleMouseUp);
    };
    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);
  }, [setSidebarWidth]);

  return (
    <Layout style={{ height: "100vh", userSelect: resizing ? "none" : "auto" }}>
      <Layout>
        <Sider
          width={sidebarCollapsed ? 48 : sidebarWidth}
          style={{ background: "#fff", borderRight: "1px solid #f0f0f0", overflow: "auto" }}
          collapsed={sidebarCollapsed}
          collapsedWidth={48}
          trigger={null}
        >
          <Sidebar />
        </Sider>
        <div
          style={{ width: 4, cursor: "col-resize", background: resizing ? "#1677ff" : "transparent", zIndex: 10 }}
          onMouseDown={handleMouseDown}
        />
        <Content style={{ display: "flex", flexDirection: "column", overflow: "hidden" }}>
          {children}
        </Content>
      </Layout>
      <StatusBar />
    </Layout>
  );
}
