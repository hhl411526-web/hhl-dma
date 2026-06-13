import { useState } from "react";
import { Table, Button, Modal, Form, Input, Select, message, Space, Popconfirm } from "antd";
import { PlusOutlined, DeleteOutlined, EditOutlined } from "@ant-design/icons";

interface RoleRecord {
  key: string;
  name: string;
  permissions: string[];
}

interface RoleManagerProps {
  connId: string;
}

export default function RoleManager({ connId: _connId }: RoleManagerProps) {
  const [roles, setRoles] = useState<RoleRecord[]>([]);
  const [modalOpen, setModalOpen] = useState(false);
  const [editingRole, setEditingRole] = useState<RoleRecord | null>(null);
  const [form] = Form.useForm();

  const addRole = () => {
    setEditingRole(null);
    form.resetFields();
    setModalOpen(true);
  };

  const editRole = (role: RoleRecord) => {
    setEditingRole(role);
    form.setFieldsValue(role);
    setModalOpen(true);
  };

  const handleSave = async () => {
    try {
      const values = await form.validateFields();
      if (editingRole) {
        setRoles(roles.map(r => r.key === editingRole.key ? { ...r, ...values } : r));
        message.success("角色已更新");
      } else {
        const newRole: RoleRecord = { key: Date.now().toString(), ...values, permissions: values.permissions || [] };
        setRoles([...roles, newRole]);
        message.success("角色已创建");
      }
      setModalOpen(false);
    } catch {
      // validation error
    }
  };

  const deleteRole = (key: string) => {
    setRoles(roles.filter(r => r.key !== key));
    message.success("角色已删除");
  };

  return (
    <div style={{ padding: 16 }}>
      <div style={{ marginBottom: 12, display: "flex", justifyContent: "space-between" }}>
        <span style={{ fontWeight: 600, fontSize: 16 }}>角色管理</span>
        <Button type="primary" icon={<PlusOutlined />} onClick={addRole}>新建角色</Button>
      </div>
      <Table
        columns={[
          { title: "角色名", dataIndex: "name" },
          { title: "权限", dataIndex: "permissions", render: (perms: string[]) => perms.join(", ") },
          {
            title: "操作", render: (_: unknown, record: RoleRecord) => (
              <Space>
                <Button type="link" size="small" icon={<EditOutlined />} onClick={() => editRole(record)}>编辑</Button>
                <Popconfirm title="确定删除此角色？" onConfirm={() => deleteRole(record.key)}>
                  <Button type="link" size="small" danger icon={<DeleteOutlined />}>删除</Button>
                </Popconfirm>
              </Space>
            ),
          },
        ]}
        dataSource={roles}
        size="small"
      />

      <Modal title={editingRole ? "编辑角色" : "新建角色"} open={modalOpen} onCancel={() => setModalOpen(false)} onOk={handleSave}>
        <Form form={form} layout="vertical">
          <Form.Item name="name" label="角色名" rules={[{ required: true }]}>
            <Input placeholder="角色名" />
          </Form.Item>
          <Form.Item name="permissions" label="权限">
            <Select mode="multiple" placeholder="选择权限" options={[
              { label: "SELECT", value: "SELECT" },
              { label: "INSERT", value: "INSERT" },
              { label: "UPDATE", value: "UPDATE" },
              { label: "DELETE", value: "DELETE" },
              { label: "CREATE", value: "CREATE" },
              { label: "DROP", value: "DROP" },
              { label: "ALTER", value: "ALTER" },
              { label: "INDEX", value: "INDEX" },
              { label: "ALL", value: "ALL" },
            ]} />
          </Form.Item>
        </Form>
      </Modal>
    </div>
  );
}
