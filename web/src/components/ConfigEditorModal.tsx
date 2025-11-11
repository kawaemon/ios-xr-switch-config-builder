import {
  Button,
  Group,
  Modal,
  Stack,
  Text,
  Textarea,
} from "@mantine/core";

type ConfigEditorModalProps = {
  opened: boolean;
  draftConfig: string;
  onChangeDraft: (value: string) => void;
  onCancel: () => void;
  onConfirm: () => void;
};

export function ConfigEditorModal({
  opened,
  draftConfig,
  onChangeDraft,
  onCancel,
  onConfirm,
}: ConfigEditorModalProps) {
  return (
    <Modal opened={opened} onClose={onCancel} title="Config入力" size="xl">
      <Stack gap="sm">
        <Text size="sm" c="dimmed">
          ここで編集した内容が解析対象になります。
        </Text>
        <Textarea
          value={draftConfig}
          minRows={20}
          spellCheck={false}
          onChange={(event) => onChangeDraft(event.currentTarget.value)}
          styles={{
            input: {
              minHeight: "20rem",
              maxHeight: "20rem",
              overflowY: "auto",
              fontFamily: "var(--mantine-font-family-monospace)",
            },
          }}
        />
        <Group justify="flex-end">
          <Button variant="default" onClick={onCancel}>
            キャンセル
          </Button>
          <Button onClick={onConfirm}>確定</Button>
        </Group>
      </Stack>
    </Modal>
  );
}
