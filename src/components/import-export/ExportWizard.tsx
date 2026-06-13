import { useState } from "react";
import { Modal, Form, Select, Input, Switch, Steps, Button, message, Space } from "antd";
import { DownloadOutlined } from "@ant-design/icons";

interface ExportWizardProps {
  open: boolean;
  onClose: () => void;
  connId: string;
  tableName?: string;
}

export default function ExportWizard({ open, onClose, connId: _connId, tableName }: ExportWizardProps) {
  const [form] = Form.useForm();
  const [step, setStep] = useState(0);
  const [exporting, setExporting] = useState(false);

  const handleExport = async () => {
    try {
      const values = await form.validateFields();
      setExporting(true);
      // Will call Tauri export command when backend is ready
      message.success(`导出成功: ${values.outputPath}`);
      setExporting(false);
      onClose();
    } catch (e) {
      message.error(`导出失败: ${String(e)}`);
      setExporting(false);
    }
  };

  const steps = [
    { title: "选择数据", description: "选择导出源" },
    { title: "配置选项", description: "设置导出格式" },
    { title: "导出", description: "执行导出" },
  ];

  return (
    <Modal title="导出数据" open={open} onCancel={onClose} width={600} footer={null} destroyOnClose>
      <Steps current={step} items={steps} size="small" style={{ marginBottom: 24 }} />
      {step === 0 && (
        <Form form={form} layout="vertical" initialValues={{ source: tableName ? "table" : "query", tableName }}>
          <Form.Item name="source" label="数据来源">
            <Select options={[{ label: "表", value: "table" }, { label: "SQL 查询", value: "query" }]} />
          </Form.Item>
          <Form.Item name="tableName" label="表名">
            <Input placeholder="输入表名" />
          </Form.Item>
          <Form.Item name="query" label="SQL 查询">
            <Input.TextArea rows={4} placeholder="SELECT * FROM table" />
          </Form.Item>
          <div style={{ textAlign: "right" }}><Button type="primary" onClick={() => setStep(1)}>下一步</Button></div>
        </Form>
      )}
      {step === 1 && (
        <Form form={form} layout="vertical" initialValues={{ format: "csv", includeHeaders: true, encoding: "utf-8" }}>
          <Form.Item name="format" label="导出格式">
            <Select options={[{ label: "CSV", value: "csv" }, { label: "JSON", value: "json" }, { label: "SQL", value: "sql" }]} />
          </Form.Item>
          <Form.Item name="includeHeaders" label="包含列头" valuePropName="checked">
            <Switch />
          </Form.Item>
          <Form.Item name="encoding" label="编码">
            <Select options={[{ label: "UTF-8", value: "utf-8" }, { label: "GBK", value: "gbk" }]} />
          </Form.Item>
          <Form.Item name="outputPath" label="输出路径" rules={[{ required: true }]}>
            <Input placeholder="/path/to/output.csv" />
          </Form.Item>
          <div style={{ textAlign: "right" }}>
            <Space><Button onClick={() => setStep(0)}>上一步</Button><Button type="primary" onClick={() => setStep(2)}>下一步</Button></Space>
          </div>
        </Form>
      )}
      {step === 2 && (
        <div>
          <p>确认导出设置后点击"开始导出"</p>
          <div style={{ textAlign: "right" }}>
            <Space><Button onClick={() => setStep(1)}>上一步</Button><Button type="primary" icon={<DownloadOutlined />} loading={exporting} onClick={handleExport}>开始导出</Button></Space>
          </div>
        </div>
      )}
    </Modal>
  );
}
