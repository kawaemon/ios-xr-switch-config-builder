import { Button, Group, Modal, Stack, Text, Textarea } from "@mantine/core";

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
          show running-config の結果をコピペしてください。ここで入力した内容がに基づいて config
          が生成されます。すべての処理はローカルで行われ、インターネット接続を必要としません。
        </Text>
        <Textarea
          value={draftConfig}
          autosize={false}
          rows={30}
          spellCheck={false}
          onChange={(event) => onChangeDraft(event.currentTarget.value)}
          styles={{
            input: {
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
