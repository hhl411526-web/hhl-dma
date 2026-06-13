import { Form, Input, Select, InputNumber, Button, message } from "antd";
import { useConnectionStore } from "../../stores/connectionStore";
import { useState } from "react";

const DB_TYPE_OPTIONS = [
  { label: "MySQL / MariaDB", value: "mysql" },
  { label: "PostgreSQL", value: "postgresql" },
  { label: "SQLite", value: "sqlite" },
  { label: "Oracle", value: "oracle" },
  { label: "SQL Server", value: "sqlserver" },
  { label: "达梦 (DM)", value: "dm" },
  { label: "人大金仓", value: "kingbasees" },
  { label: "通用 ODBC", value: "odbc" },
];

const DEFAULT_PORTS: Record<string, number> = {
  mysql: 3306, postgresql: 5432, oracle: 1521, sqlserver: 1433, dm: 5236, kingbasees: 54321,
};

interface ConnectionFormProps {
  onSuccess?: () => void;
}

export default function ConnectionForm({ onSuccess }: ConnectionFormProps) {
  const [form] = Form.useForm();
  const { addConnection, testConnection } = useConnectionStore();
  const [testing, setTesting] = useState(false);
  const [saving, setSaving] = useState(false);

  const handleDbTypeChange = (dbType: string) => {
    const defaultPort = DEFAULT_PORTS[dbType];
    if (defaultPort) form.setFieldsValue({ port: defaultPort });
    if (dbType === "sqlite") form.setFieldsValue({ host: "", port: 0, username: "", password: "" });
  };

  const handleTest = async () => {
    try {
      const values = await form.validateFields();
      setTesting(true);
      const success = await testConnection({
        dbType: values.db_type, host: values.host, port: values.port,
        username: values.username, password: values.password, database: values.database,
      });
      if (success) message.success("连接测试成功");
    } catch (e) {
      message.error(`连接测试失败: ${String(e)}`);
    } finally { setTesting(false); }
  };

  const handleSave = async () => {
    try {
      const values = await form.validateFields();
      setSaving(true);
      await addConnection({
        name: values.name, dbType: values.db_type, host: values.host, port: values.port,
        username: values.username, password: values.password, database: values.database,
        schema: values.schema, group: values.group,
      });
      message.success("连接已保存");
      onSuccess?.();
    } catch (e) {
      message.error(`保存失败: ${String(e)}`);
    } finally { setSaving(false); }
  };

  const dbType = Form.useWatch("db_type", form);

  return (
    <Form form={form} layout="vertical" initialValues={{ db_type: "mysql", port: 3306 }}>
      <Form.Item name="name" label="连接名称" rules={[{ required: true }]}>
        <Input placeholder="例如: 开发环境 MySQL" />
      </Form.Item>
      <Form.Item name="db_type" label="数据库类型" rules={[{ required: true }]}>
        <Select options={DB_TYPE_OPTIONS} onChange={handleDbTypeChange} />
      </Form.Item>
      {dbType !== "sqlite" && (
        <>
          <Form.Item name="host" label="主机" rules={[{ required: true }]}>
            <Input placeholder="localhost" />
          </Form.Item>
          <Form.Item name="port" label="端口" rules={[{ required: true }]}>
            <InputNumber style={{ width: "100%" }} min={1} max={65535} />
          </Form.Item>
          <Form.Item name="username" label="用户名" rules={[{ required: true }]}>
            <Input placeholder="root" />
          </Form.Item>
          <Form.Item name="password" label="密码">
            <Input.Password placeholder="密码" />
          </Form.Item>
        </>
      )}
      <Form.Item name="database" label="数据库" rules={[{ required: true }]}>
        <Input placeholder={dbType === "sqlite" ? "数据库文件路径" : "数据库名"} />
      </Form.Item>
      <Form.Item name="group" label="分组">
        <Input placeholder="可选分组" />
      </Form.Item>
      <div style={{ display: "flex", gap: 8, justifyContent: "flex-end" }}>
        <Button onClick={handleTest} loading={testing}>测试连接</Button>
        <Button type="primary" onClick={handleSave} loading={saving}>保存</Button>
      </div>
    </Form>
  );
}
