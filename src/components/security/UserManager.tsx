import { useState } from "react";
import { Table, Button, Modal, Form, Input, Select, Switch, message, Space, Popconfirm } from "antd";
import { PlusOutlined, DeleteOutlined, EditOutlined } from "@ant-design/icons";

interface UserRecord {
  key: string;
  username: string;
  host: string;
  roles: string[];
  active: boolean;
}

interface UserManagerProps {
  connId: string;
}

export default function UserManager({ connId: _connId }: UserManagerProps) {
  const [users, setUsers] = useState<UserRecord[]>([]);
  const [modalOpen, setModalOpen] = useState(false);
  const [editingUser, setEditingUser] = useState<UserRecord | null>(null);
  const [form] = Form.useForm();

  const addUser = () => {
    setEditingUser(null);
    form.resetFields();
    setModalOpen(true);
  };

  const editUser = (user: UserRecord) => {
    setEditingUser(user);
    form.setFieldsValue(user);
    setModalOpen(true);
  };

  const handleSave = async () => {
    try {
      const values = await form.validateFields();
      if (editingUser) {
        setUsers(users.map(u => u.key === editingUser.key ? { ...u, ...values } : u));
        message.success("用户已更新");
      } else {
        const newUser: UserRecord = { key: Date.now().toString(), ...values, roles: values.roles || [] };
        setUsers([...users, newUser]);
        message.success("用户已创建");
      }
      setModalOpen(false);
    } catch {
      // validation error
    }
  };

  const deleteUser = (key: string) => {
    setUsers(users.filter(u => u.key !== key));
    message.success("用户已删除");
  };

  return (
    <div style={{ padding: 16 }}>
      <div style={{ marginBottom: 12, display: "flex", justifyContent: "space-between" }}>
        <span style={{ fontWeight: 600, fontSize: 16 }}>用户管理</span>
        <Button type="primary" icon={<PlusOutlined />} onClick={addUser}>新建用户</Button>
      </div>
      <Table
        columns={[
          { title: "用户名", dataIndex: "username" },
          { title: "主机", dataIndex: "host" },
          { title: "角色", dataIndex: "roles", render: (roles: string[]) => roles.join(", ") },
          { title: "活跃", dataIndex: "active", render: (v: boolean) => v ? "是" : "否" },
          {
            title: "操作", render: (_: unknown, record: UserRecord) => (
              <Space>
                <Button type="link" size="small" icon={<EditOutlined />} onClick={() => editUser(record)}>编辑</Button>
                <Popconfirm title="确定删除此用户？" onConfirm={() => deleteUser(record.key)}>
                  <Button type="link" size="small" danger icon={<DeleteOutlined />}>删除</Button>
                </Popconfirm>
              </Space>
            ),
          },
        ]}
        dataSource={users}
        size="small"
      />

      <Modal title={editingUser ? "编辑用户" : "新建用户"} open={modalOpen} onCancel={() => setModalOpen(false)} onOk={handleSave}>
        <Form form={form} layout="vertical" initialValues={{ host: "%", active: true }}>
          <Form.Item name="username" label="用户名" rules={[{ required: true }]}>
            <Input placeholder="username" />
          </Form.Item>
          <Form.Item name="host" label="主机" rules={[{ required: true }]}>
            <Input placeholder="%" />
          </Form.Item>
          <Form.Item name="password" label="密码">
            <Input.Password placeholder="密码" />
          </Form.Item>
          <Form.Item name="roles" label="角色">
            <Select mode="multiple" placeholder="选择角色" options={[{ label: "DBA", value: "dba" }, { label: "读写", value: "readwrite" }, { label: "只读", value: "readonly" }]} />
          </Form.Item>
          <Form.Item name="active" label="活跃" valuePropName="checked">
            <Switch />
          </Form.Item>
        </Form>
      </Modal>
    </div>
  );
}
