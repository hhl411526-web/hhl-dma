# HHL-DMA 数据库管理软件设计文档

## 1. 整体架构

插件化三层架构：前端层 + 核心服务层 + 插件层

```
┌─────────────────────────────────────────────────────────────────┐
│                        前端层 (React + TS)                       │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────────┐   │
│  │ 连接管理  │ │ SQL编辑器 │ │ 对象浏览器│ │ 表设计器/ER图等   │   │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └───────┬──────────┘   │
│       └────────────┼────────────┼────────────────┘              │
│                    │  Tauri IPC (invoke / events)               │
├────────────────────┼────────────────────────────────────────────┤
│                    ▼                                             │
│              核心服务层 (Rust)                                    │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────────┐   │
│  │ 连接池管理│ │ SQL执行引擎│ │ 元数据服务│ │ 插件管理器       │   │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └───────┬──────────┘   │
│       └────────────┼────────────┼────────────────┘              │
│                    │  DatabaseDriver trait                       │
├────────────────────┼────────────────────────────────────────────┤
│                    ▼                                             │
│               插件层 (Rust dyn plugin)                           │
│  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌──────────────┐  │
│  │ MySQL  │ │PgSQL   │ │SQLite  │ │Oracle  │ │ 达梦/ODBC/...│  │
│  │ Plugin │ │Plugin  │ │Plugin  │ │Plugin  │ │ Plugin       │  │
│  └───┬────┘ └───┬────┘ └───┬────┘ └───┬────┘ └──────┬───────┘  │
│      └──────────┴──────────┴──────────┴──────────────┘          │
│                         │ 原生驱动 / ODBC / JDBC                 │
├─────────────────────────┼───────────────────────────────────────┤
│                    ▼                                             │
│              数据库实例                                          │
│         MySQL  PostgreSQL  SQLite  Oracle  达梦 ...              │
└─────────────────────────────────────────────────────────────────┘
```

### 设计要点

- **前端层**：React + TypeScript，通过 Tauri IPC 与后端通信，不直接访问数据库
- **核心服务层**：Rust 实现，管理连接池、SQL 执行、元数据缓存，定义 DatabaseDriver trait
- **插件层**：每个数据库类型实现 DatabaseDriver trait，独立编译为动态库

### 关键约束

- 前端永远不直连数据库，所有操作经核心层
- 插件必须实现完整的 Driver trait 才能注册
- 连接信息加密存储在本地

---

## 2. 插件系统设计

### DatabaseDriver Trait

```rust
/// 所有数据库插件必须实现的核心接口
#[async_trait]
pub trait DatabaseDriver: Send + Sync {
    /// 插件元信息
    fn info(&self) -> DriverInfo;

    /// 建立连接
    async fn connect(&self, config: &ConnectionConfig) -> Result<Connection>;

    /// 断开连接
    async fn disconnect(&self, conn_id: &str) -> Result<()>;

    /// 执行 SQL（支持查询和变更）
    async fn execute(&self, conn_id: &str, sql: &str) -> Result<QueryResult>;

    /// 获取元数据
    async fn metadata(&self, conn_id: &str) -> Result<DatabaseMetadata>;

    /// 获取表结构
    async fn table_schema(&self, conn_id: &str, table: &str) -> Result<TableSchema>;

    /// 获取执行计划
    async fn explain(&self, conn_id: &str, sql: &str) -> Result<ExplainResult>;

    /// 导出数据
    async fn export(&self, conn_id: &str, config: &ExportConfig) -> Result<ExportResult>;

    /// 导入数据
    async fn import(&self, conn_id: &str, config: &ImportConfig) -> Result<ImportResult>;

    /// 获取数据库特有的 SQL 方言提示
    fn dialect(&self) -> SqlDialect;
}
```

### 插件生命周期

```
注册 → 发现 → 加载 → 初始化 → 使用 → 卸载
  │      │      │       │       │      │
  │      │      │       │       │      └─ 释放资源、断开连接
  │      │      │       │       └─ 调用 Driver trait 方法
  │      │      │       └─ 验证驱动可用性
  │      │      └─ 动态加载 .so/.dylib/.dll
  │      └─ 扫描 plugins/ 目录
  └─ 声明插件元信息（名称、版本、支持的数据库类型）
```

### 插件目录结构

```
plugins/
├── mysql/
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs          # 实现 DatabaseDriver
├── postgresql/
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
├── sqlite/
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
├── oracle/
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
├── dameng/                   # 达梦
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
└── odbc/                     # 通用 ODBC 驱动
    ├── Cargo.toml
    └── src/
        └── lib.rs
```

---

## 3. 前端模块设计

### 整体布局

```
┌──────────────────────────────────────────────────────────┐
│  菜单栏  [文件] [编辑] [视图] [工具] [帮助]                  │
├────────┬─────────────────────────────────────────────────┤
│        │  标签页栏  [查询1] [查询2] [表结构] [+]            │
│ 连接   ├─────────────────────────────────────────────────┤
│ 面板   │                                                 │
│        │              主工作区                              │
│ ┌────┐ │     （SQL编辑器 / 表设计器 / ER图 / 数据对比）      │
│ │MySQL│ │                                                 │
│ │ ┌dev│ │                                                 │
│ │ └tst│ │                                                 │
│ │PgSQL│ │                                                 │
│ │ ┌db1│ │                                                 │
│ └────┘ ├─────────────────────────────────────────────────┤
│        │  结果面板  [结果] [消息] [统计]                      │
│        │  ┌─────┬─────┬─────┬─────┐                      │
│        │  │ id  │ name│ age │ ... │  ← 可排序/筛选/编辑     │
│        │  ├─────┼─────┼─────┼─────┤                      │
│        │  │ 1   │ ... │ ... │ ... │                      │
│        │  └─────┴─────┴─────┴─────┘                      │
├────────┴─────────────────────────────────────────────────┤
│  状态栏  [已连接: MySQL@dev] [查询耗时: 0.03s] [行数: 42]   │
└──────────────────────────────────────────────────────────┘
```

### 核心模块

| 模块 | 功能 | 优先级 |
|------|------|--------|
| 连接管理 | 新建/编辑/删除连接，连接测试，分组管理 | P0 |
| SQL 编辑器 | 语法高亮、自动补全、格式化、多标签 | P0 |
| 对象浏览器 | 树形浏览数据库/表/字段/索引，右键菜单 | P0 |
| 查询结果面板 | 表格展示、排序、筛选、内联编辑、导出 | P0 |
| 表结构设计器 | 可视化建表/改表，索引/约束管理 | P1 |
| 数据导入导出 | CSV/JSON/SQL 格式导入导出，数据迁移 | P1 |
| ER 图 | 自动生成 ER 图，关系可视化 | P2 |
| 可视化查询构建器 | 拖拽生成 SQL | P2 |
| 性能分析 | 执行计划可视化，慢查询分析 | P2 |
| 数据对比同步 | 结构对比、数据对比、同步脚本生成 | P2 |
| 安全管理 | 用户/角色/权限管理 | P2 |

---

## 4. 核心服务层设计

### 模块划分

```
src-tauri/
├── src/
│   ├── main.rs                 # Tauri 入口
│   ├── core/
│   │   ├── mod.rs
│   │   ├── connection/         # 连接池管理
│   │   │   ├── manager.rs      # 连接生命周期管理
│   │   │   ├── pool.rs         # 连接池实现
│   │   │   └── config.rs       # 连接配置（加密存储）
│   │   ├── executor/           # SQL 执行引擎
│   │   │   ├── engine.rs       # 执行调度
│   │   │   ├── result.rs       # 结果处理
│   │   │   └── history.rs      # 执行历史
│   │   ├── metadata/           # 元数据服务
│   │   │   ├── cache.rs        # 元数据缓存
│   │   │   └── schema.rs       # Schema 信息
│   │   ├── plugin/             # 插件管理器
│   │   │   ├── loader.rs       # 插件加载/卸载
│   │   │   ├── registry.rs     # 插件注册表
│   │   │   └── driver.rs       # DatabaseDriver trait 定义
│   │   └── security/           # 安全模块
│   │       ├── crypto.rs       # 加密/解密
│   │       └── vault.rs        # 密钥存储
│   ├── commands/               # Tauri IPC 命令
│   │   ├── connection.rs       # 连接相关命令
│   │   ├── query.rs            # 查询相关命令
│   │   ├── metadata.rs         # 元数据相关命令
│   │   ├── import_export.rs    # 导入导出命令
│   │   └── plugin.rs           # 插件管理命令
│   └── errors.rs               # 统一错误处理
```

### 连接池管理

- 每个活跃连接维护一个连接池，支持最小/最大连接数配置
- 连接空闲超时自动回收
- 连接健康检查（心跳）
- 支持连接复用，多查询共享同一连接池

### SQL 执行引擎

- 支持单条/批量执行
- 支持取消正在执行的查询
- 大结果集分页/流式返回
- 执行超时控制
- 执行历史记录（可配置保留条数）

### 元数据缓存

- 缓存数据库/表/字段/索引信息
- 支持手动刷新和定时刷新
- 增量更新（监听数据库变更事件）

---

## 5. 数据库支持矩阵

### 第一版（MVP）— 必须支持

| 数据库 | 驱动方式 | 连接方式 | 备注 |
|--------|----------|----------|------|
| MySQL / MariaDB | 原生 Rust 驱动 | TCP | 最常用，优先支持 |
| PostgreSQL | 原生 Rust 驱动 | TCP | 最常用，优先支持 |
| SQLite | Rust 驱动 | 本地文件 | 无需服务端 |

### 第二版 — 扩展支持

| 数据库 | 驱动方式 | 连接方式 | 备注 |
|--------|----------|----------|------|
| Oracle | ODBC / OCI | TCP | 商业数据库 |
| SQL Server | ODBC / TDS | TCP | 商业数据库 |
| 达梦 (DM) | ODBC / 专用驱动 | TCP | 国产数据库 |
| 人大金仓 (KingbaseES) | ODBC | TCP | 国产数据库 |
| 神舟通用 (Oscar) | ODBC | TCP | 国产数据库 |
| 南大通用 (GBase) | ODBC | TCP | 国产数据库 |
| 通用 ODBC | ODBC | 可配置 | 兜底方案 |

---

## 6. 数据流设计

### 查询执行流程

```
用户输入 SQL
    │
    ▼
前端 SQL 编辑器 ──(Tauri invoke)──▶ commands::query::execute()
    │                                      │
    │                                      ▼
    │                              执行引擎获取连接
    │                                      │
    │                                      ▼
    │                              从连接池取连接
    │                                      │
    │                                      ▼
    │                              路由到对应插件 Driver
    │                                      │
    │                                      ▼
    │                              Driver.execute(sql)
    │                                      │
    │                                      ▼
    │                              数据库返回结果
    │                                      │
    │                                      ▼
    │                              结果分页/流式处理
    │                                      │
    ◀────────(Tauri event)──────── 返回结果到前端
    │
    ▼
结果面板渲染表格
```

### 连接建立流程

```
用户新建连接
    │
    ▼
前端连接对话框 ──(invoke)──▶ commands::connection::create()
    │                                │
    │                                ▼
    │                        加密存储连接配置
    │                                │
    │                                ▼
    │                        查找匹配的 Driver 插件
    │                                │
    │                                ▼
    │                        Driver.connect(config)
    │                                │
    │                                ▼
    │                        连接成功 → 加入连接池
    │                                │
    ◀────────(返回连接ID)───────────┘
    │
    ▼
对象浏览器加载元数据
```

---

## 7. 安全设计

- 连接密码使用 AES-256-GCM 加密存储，密钥派生自机器特征码
- 支持主密码模式：用户设置主密码，所有连接密码用主密码加密
- SQL 编辑器不记录敏感查询（用户可配置）
- 连接配置文件权限控制（仅当前用户可读）

---

## 8. 技术选型汇总

| 层级 | 技术 | 说明 |
|------|------|------|
| 桌面框架 | Tauri v2 | Rust 后端 + WebView 前端 |
| 前端框架 | React 18 + TypeScript | 组件化开发 |
| UI 组件库 | Ant Design 5 | 企业级组件库，功能丰富 |
| 状态管理 | Zustand | 轻量级状态管理 |
| SQL 编辑器 | Monaco Editor | VSCode 同款编辑器 |
| ER 图 | ReactFlow | 流程图/关系图 |
| 图标 | Lucide React | 轻量图标库 |
| 后端语言 | Rust | 高性能、内存安全 |
| 插件系统 | Rust动态库加载 (libloading) | 运行时加载插件 |
| MySQL驱动 | sqlx / mysql-async | Rust原生MySQL驱动 |
| PostgreSQL驱动 | sqlx / tokio-postgres | Rust原生Pg驱动 |
| SQLite驱动 | rusqlite / sqlx | Rust原生SQLite驱动 |
| ODBC | odbc-api | 通用ODBC驱动 |
| 加密 | aes-gcm + ring | AES-256-GCM加密 |

---

## 9. 项目目录结构

```
hhl-dma/
├── src/                          # 前端源码
│   ├── App.tsx
│   ├── main.tsx
│   ├── components/
│   │   ├── layout/               # 布局组件
│   │   │   ├── AppLayout.tsx
│   │   │   ├── Sidebar.tsx
│   │   │   └── StatusBar.tsx
│   │   ├── connection/           # 连接管理
│   │   │   ├── ConnectionPanel.tsx
│   │   │   ├── ConnectionDialog.tsx
│   │   │   └── ConnectionForm.tsx
│   │   ├── editor/               # SQL 编辑器
│   │   │   ├── SqlEditor.tsx
│   │   │   ├── EditorTabs.tsx
│   │   │   └── AutoComplete.tsx
│   │   ├── browser/              # 对象浏览器
│   │   │   ├── ObjectBrowser.tsx
│   │   │   └── TreeNode.tsx
│   │   ├── result/               # 结果面板
│   │   │   ├── ResultPanel.tsx
│   │   │   ├── ResultTable.tsx
│   │   │   └── ResultToolbar.tsx
│   │   ├── designer/             # 表设计器
│   │   │   ├── TableDesigner.tsx
│   │   │   └── ColumnEditor.tsx
│   │   ├── er/                   # ER 图
│   │   │   └── ErDiagram.tsx
│   │   ├── import-export/        # 导入导出
│   │   │   ├── ImportWizard.tsx
│   │   │   └── ExportWizard.tsx
│   │   ├── compare/              # 数据对比
│   │   │   └── DataCompare.tsx
│   │   └── performance/          # 性能分析
│   │       └── ExplainPanel.tsx
│   ├── stores/                   # Zustand 状态
│   │   ├── connectionStore.ts
│   │   ├── queryStore.ts
│   │   └── uiStore.ts
│   ├── hooks/                    # 自定义 Hooks
│   ├── services/                 # Tauri IPC 封装
│   │   ├── connectionService.ts
│   │   ├── queryService.ts
│   │   └── metadataService.ts
│   ├── types/                    # TypeScript 类型定义
│   └── utils/                    # 工具函数
├── src-tauri/                    # Tauri 后端源码
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── core/                 # 核心服务
│   │   ├── commands/             # IPC 命令
│   │   └── errors.rs
│   └── tauri.conf.json
├── plugins/                      # 数据库插件
│   ├── mysql/
│   ├── postgresql/
│   ├── sqlite/
│   ├── oracle/
│   ├── dameng/
│   └── odbc/
├── package.json
├── tsconfig.json
├── vite.config.ts
└── README.md
```
