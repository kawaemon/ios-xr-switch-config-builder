import { Modal, ScrollArea, Stack, Text } from "@mantine/core";

type LintResultModalProps = {
  opened: boolean;
  lintOutput: string;
  onClose: () => void;
};

export function LintResultModal({
  opened,
  lintOutput,
  onClose,
}: LintResultModalProps) {
  return (
    <Modal opened={opened} onClose={onClose} title="Lint結果" size="lg">
      <Stack gap="sm">
        <Text size="sm" c="dimmed">
          下記の指摘を解消すると Bridge VLAN 情報が表示されます。
        </Text>
        <ScrollArea h={300} type="always" offsetScrollbars>
          <Text
            component="pre"
            fz="sm"
            style={{
              margin: 0,
              whiteSpace: "pre-wrap",
              fontFamily: "var(--mantine-font-family-monospace)",
            }}
          >
            {lintOutput}
          </Text>
        </ScrollArea>
      </Stack>
    </Modal>
  );
}
