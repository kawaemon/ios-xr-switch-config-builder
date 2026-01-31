import { Code, Modal, Stack } from "@mantine/core";
import { demoChangeInput } from "../demoData";

type ChangeCommandExamplesModalProps = {
  opened: boolean;
  onClose: () => void;
};

export function ChangeCommandExamplesModal({ opened, onClose }: ChangeCommandExamplesModalProps) {
  return (
    <Modal opened={opened} onClose={onClose} title="コマンド例" size="lg">
      <Stack gap="md">
        <Code block>{demoChangeInput}</Code>
      </Stack>
    </Modal>
  );
}
