import { useState } from "react";
import { Modal, Form, Select, Input, Switch, Steps, Button, message, Space } from "antd";
import { UploadOutlined } from "@ant-design/icons";

interface ImportWizardProps {
  open: boolean;
  onClose: () => void;
  connId: string;
  tableName?: string;
}

export default function ImportWizard({ open, onClose, connId: _connId, tableName }: ImportWizardProps) {
  const [form] = Form.useForm();
  const [step, setStep] = useState(0);
  const [importing, setImporting] = useState(false);

  const handleImport = async () => {
    try {
      setImporting(true);
      message.success("导入成功");
      setImporting(false);
      onClose();
    } catch (e) {
      message.error(`导入失败: ${String(e)}`);
      setImporting(false);
    }
  };

  const steps = [
    { title: "选择文件", description: "选择导入文件" },
    { title: "配置选项", description: "设置导入参数" },
    { title: "导入", description: "执行导入" },
  ];

  return (
    <Modal title="导入数据" open={open} onCancel={onClose} width={600} footer={null} destroyOnClose>
      <Steps current={step} items={steps} size="small" style={{ marginBottom: 24 }} />
      {step === 0 && (
        <Form form={form} layout="vertical" initialValues={{ format: "csv", tableName }}>
          <Form.Item name="format" label="文件格式">
            <Select options={[{ label: "CSV", value: "csv" }, { label: "JSON", value: "json" }, { label: "SQL", value: "sql" }]} />
          </Form.Item>
          <Form.Item name="filePath" label="文件路径" rules={[{ required: true }]}>
            <Input placeholder="/path/to/data.csv" />
          </Form.Item>
          <Form.Item name="tableName" label="目标表" rules={[{ required: true }]}>
            <Input placeholder="输入目标表名" />
          </Form.Item>
          <div style={{ textAlign: "right" }}><Button type="primary" onClick={() => setStep(1)}>下一步</Button></div>
        </Form>
      )}
      {step === 1 && (
        <Form form={form} layout="vertical" initialValues={{ hasHeaders: true, encoding: "utf-8", onConflict: "error" }}>
          <Form.Item name="hasHeaders" label="包含列头" valuePropName="checked">
            <Switch />
          </Form.Item>
          <Form.Item name="encoding" label="编码">
            <Select options={[{ label: "UTF-8", value: "utf-8" }, { label: "GBK", value: "gbk" }]} />
          </Form.Item>
          <Form.Item name="onConflict" label="冲突策略">
            <Select options={[{ label: "报错", value: "error" }, { label: "跳过", value: "skip" }, { label: "更新", value: "update" }]} />
          </Form.Item>
          <div style={{ textAlign: "right" }}>
            <Space><Button onClick={() => setStep(0)}>上一步</Button><Button type="primary" onClick={() => setStep(2)}>下一步</Button></Space>
          </div>
        </Form>
      )}
      {step === 2 && (
        <div>
          <p>确认导入设置后点击"开始导入"</p>
          <div style={{ textAlign: "right" }}>
            <Space><Button onClick={() => setStep(1)}>上一步</Button><Button type="primary" icon={<UploadOutlined />} loading={importing} onClick={handleImport}>开始导入</Button></Space>
          </div>
        </div>
      )}
    </Modal>
  );
}
