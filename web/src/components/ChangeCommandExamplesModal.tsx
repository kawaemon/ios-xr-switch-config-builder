import { Code, Modal, Paper, Stack, Text } from "@mantine/core";

type ChangeCommandExamplesModalProps = {
  opened: boolean;
  onClose: () => void;
};

const examples = [
  {
    title: "トランクポートの設定",
    lines: [
      "interface FortyGigE0/0/0/46",
      "  description To:server-1",
      "  switchport mode trunk",
      "  switchport trunk allowed vlan add 300 350",
      "  switchport trunk allowed vlan remove 200",
      "  switchport trunk allowed vlan none",
    ],
  },
  {
    title: "BVI の作成",
    lines: ["interface BVI500"],
  },
  {
    title: "VLAN の追加",
    description: "VLAN 名を登録します。",
    lines: ["vlan database", "  vlan 350 name demo-servers"],
  },
];

export function ChangeCommandExamplesModal({
  opened,
  onClose,
}: ChangeCommandExamplesModalProps) {
  return (
    <Modal opened={opened} onClose={onClose} title="コマンド例" size="lg">
      <Stack gap="md">
        {examples.map((example) => (
          <Paper key={example.title} withBorder radius="md" p="md">
            <Stack gap="xs">
              <Text fw={600}>{example.title}</Text>
              <Text size="sm" c="dimmed">
                {example.description}
              </Text>
              <Code block>{example.lines.join("\n")}</Code>
            </Stack>
          </Paper>
        ))}
      </Stack>
    </Modal>
  );
}
