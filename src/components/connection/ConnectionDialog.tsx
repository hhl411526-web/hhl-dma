import { Modal } from "antd";
import ConnectionForm from "./ConnectionForm";

interface ConnectionDialogProps {
  open: boolean;
  onClose: () => void;
}

export default function ConnectionDialog({ open, onClose }: ConnectionDialogProps) {
  return (
    <Modal title="新建连接" open={open} onCancel={onClose} footer={null} width={520} destroyOnClose>
      <ConnectionForm onSuccess={onClose} />
    </Modal>
  );
}
