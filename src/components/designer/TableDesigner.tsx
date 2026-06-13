import { useState, useEffect } from "react";
import { Table, Button, Input, Select, Switch, InputNumber, message, Popconfirm } from "antd";
import { PlusOutlined, DeleteOutlined, SaveOutlined } from "@ant-design/icons";
import type { ColumnInfo, IndexInfo, TableSchema } from "../../types";

interface TableDesignerProps {
  connId: string;
  tableName?: string;
  existingSchema?: TableSchema;
  onSave?: (schema: TableSchema) => void;
}

const DATA_TYPES = [
  "INT", "BIGINT", "SMALLINT", "TINYINT",
  "DECIMAL", "FLOAT", "DOUBLE",
  "VARCHAR", "CHAR", "TEXT", "LONGTEXT",
  "DATE", "DATETIME", "TIMESTAMP", "TIME",
  "BOOLEAN", "BLOB", "JSON",
];

export default function TableDesigner({ connId: _connId, tableName, existingSchema, onSave }: TableDesignerProps) {
  const [columns, setColumns] = useState<ColumnInfo[]>([]);
  const [indexes, setIndexes] = useState<IndexInfo[]>([]);
  const [name, setName] = useState(tableName || "");

  useEffect(() => {
    if (existingSchema) {
      setColumns(existingSchema.columns);
      setIndexes(existingSchema.indexes);
      setName(existingSchema.table_name);
    }
  }, [existingSchema]);

  const addColumn = () => {
    const newCol: ColumnInfo = {
      name: "",
      data_type: "VARCHAR",
      nullable: true,
      default_value: undefined,
      is_primary_key: false,
      is_auto_increment: false,
      comment: undefined,
      ordinal_position: columns.length + 1,
      max_length: undefined,
      precision: undefined,
      scale: undefined,
    };
    setColumns([...columns, newCol]);
  };

  const updateColumn = (index: number, field: keyof ColumnInfo, value: unknown) => {
    const newColumns = [...columns];
    newColumns[index] = { ...newColumns[index], [field]: value };
    setColumns(newColumns);
  };

  const removeColumn = (index: number) => {
    setColumns(columns.filter((_, i) => i !== index));
  };

  const addIndex = () => {
    const newIndex: IndexInfo = {
      name: "",
      columns: [],
      is_unique: false,
      is_primary: false,
      index_type: "BTREE",
    };
    setIndexes([...indexes, newIndex]);
  };

  const updateIndex = (index: number, field: keyof IndexInfo, value: unknown) => {
    const newIndexes = [...indexes];
    newIndexes[index] = { ...newIndexes[index], [field]: value };
    setIndexes(newIndexes);
  };

  const removeIndex = (index: number) => {
    setIndexes(indexes.filter((_, i) => i !== index));
  };

  const handleSave = () => {
    if (!name.trim()) {
      message.error("请输入表名");
      return;
    }
    if (columns.length === 0) {
      message.error("请至少添加一列");
      return;
    }
    const hasEmptyName = columns.some(c => !c.name.trim());
    if (hasEmptyName) {
      message.error("列名不能为空");
      return;
    }
    const schema: TableSchema = {
      table_name: name,
      columns,
      indexes,
      constraints: [],
    };
    onSave?.(schema);
    message.success("表结构已保存");
  };

  const columnData = columns.map((col, index) => ({
    key: index,
    ...col,
  }));

  const columnTableColumns = [
    {
      title: "列名", dataIndex: "name", width: 150,
      render: (_: unknown, __: unknown, index: number) => (
        <Input value={columns[index].name} onChange={e => updateColumn(index, "name", e.target.value)} size="small" placeholder="列名" />
      ),
    },
    {
      title: "类型", dataIndex: "data_type", width: 140,
      render: (_: unknown, __: unknown, index: number) => (
        <Select value={columns[index].data_type} onChange={v => updateColumn(index, "data_type", v)} size="small" style={{ width: "100%" }} options={DATA_TYPES.map(t => ({ label: t, value: t }))} />
      ),
    },
    {
      title: "长度", dataIndex: "max_length", width: 80,
      render: (_: unknown, __: unknown, index: number) => (
        <InputNumber value={columns[index].max_length ?? undefined} onChange={v => updateColumn(index, "max_length", v)} size="small" style={{ width: "100%" }} min={1} />
      ),
    },
    {
      title: "可空", dataIndex: "nullable", width: 60,
      render: (_: unknown, __: unknown, index: number) => (
        <Switch checked={columns[index].nullable} onChange={v => updateColumn(index, "nullable", v)} size="small" />
      ),
    },
    {
      title: "主键", dataIndex: "is_primary_key", width: 60,
      render: (_: unknown, __: unknown, index: number) => (
        <Switch checked={columns[index].is_primary_key} onChange={v => updateColumn(index, "is_primary_key", v)} size="small" />
      ),
    },
    {
      title: "自增", dataIndex: "is_auto_increment", width: 60,
      render: (_: unknown, __: unknown, index: number) => (
        <Switch checked={columns[index].is_auto_increment} onChange={v => updateColumn(index, "is_auto_increment", v)} size="small" />
      ),
    },
    {
      title: "默认值", dataIndex: "default_value", width: 120,
      render: (_: unknown, __: unknown, index: number) => (
        <Input value={columns[index].default_value ?? ""} onChange={e => updateColumn(index, "default_value", e.target.value || undefined)} size="small" placeholder="默认值" />
      ),
    },
    {
      title: "注释", dataIndex: "comment", width: 150,
      render: (_: unknown, __: unknown, index: number) => (
        <Input value={columns[index].comment ?? ""} onChange={e => updateColumn(index, "comment", e.target.value || undefined)} size="small" placeholder="注释" />
      ),
    },
    {
      title: "操作", width: 60,
      render: (_: unknown, __: unknown, index: number) => (
        <Popconfirm title="确定删除此列？" onConfirm={() => removeColumn(index)}>
          <Button type="text" size="small" danger icon={<DeleteOutlined />} />
        </Popconfirm>
      ),
    },
  ];

  return (
    <div style={{ padding: 16, height: "100%", overflow: "auto" }}>
      <div style={{ marginBottom: 16, display: "flex", gap: 16, alignItems: "center" }}>
        <span style={{ fontWeight: 600 }}>表名:</span>
        <Input value={name} onChange={e => setName(e.target.value)} style={{ width: 300 }} placeholder="输入表名" />
      </div>

      <div style={{ marginBottom: 8, display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <span style={{ fontWeight: 600 }}>列定义</span>
        <Button type="dashed" size="small" icon={<PlusOutlined />} onClick={addColumn}>添加列</Button>
      </div>
      <Table columns={columnTableColumns} dataSource={columnData} size="small" pagination={false} scroll={{ x: 900 }} />

      <div style={{ marginTop: 16, marginBottom: 8, display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <span style={{ fontWeight: 600 }}>索引</span>
        <Button type="dashed" size="small" icon={<PlusOutlined />} onClick={addIndex}>添加索引</Button>
      </div>
      <Table
        columns={[
          { title: "索引名", dataIndex: "name", width: 150, render: (_: unknown, __: unknown, index: number) => <Input value={indexes[index].name} onChange={e => updateIndex(index, "name", e.target.value)} size="small" /> },
          { title: "列", dataIndex: "columns", width: 200, render: (_: unknown, __: unknown, index: number) => <Select mode="tags" value={indexes[index].columns} onChange={v => updateIndex(index, "columns", v)} size="small" style={{ width: "100%" }} options={columns.map(c => ({ label: c.name, value: c.name }))} /> },
          { title: "唯一", dataIndex: "is_unique", width: 60, render: (_: unknown, __: unknown, index: number) => <Switch checked={indexes[index].is_unique} onChange={v => updateIndex(index, "is_unique", v)} size="small" /> },
          { title: "操作", width: 60, render: (_: unknown, __: unknown, index: number) => <Popconfirm title="确定删除？" onConfirm={() => removeIndex(index)}><Button type="text" size="small" danger icon={<DeleteOutlined />} /></Popconfirm> },
        ]}
        dataSource={indexes.map((idx, i) => ({ key: i, ...idx }))}
        size="small"
        pagination={false}
      />

      <div style={{ marginTop: 16, textAlign: "right" }}>
        <Button type="primary" icon={<SaveOutlined />} onClick={handleSave}>保存表结构</Button>
      </div>
    </div>
  );
}
