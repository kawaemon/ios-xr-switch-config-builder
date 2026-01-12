import { Code, Modal, Paper, Stack, Text } from "@mantine/core";

type ChangeCommandExamplesModalProps = {
  opened: boolean;
  onClose: () => void;
};


const examples = [
  {
    title: "VLAN の追加",
    description: "VLAN 名を登録します。",
    lines: ["vlan database", "  vlan 350 name demo-servers"],
  },
  {
    title: "トランクポートの設定",
    description: "複数 VLAN を通す場合の例です。",
    lines: [
      "interface FortyGigE0/0/0/46",
      "  description To:server-1",
      "  switchport mode trunk",
      "  switchport trunk allowed vlan add 300 350",
      "  switchport trunk allowed vlan remove 200",
    ],
  },
  {
    title: "アクセスポートの設定",
    description: "単一 VLAN を収容する例です。",
    lines: [
      "interface FortyGigE0/0/0/48",
      "  description To:mgmt-sw",
      "  switchport mode access",
      "  switchport access vlan 500",
    ],
  },
  {
    title: "BVI の作成",
    description: "L3 設定は別途投入します。",
    lines: ["interface BVI500"],
  },
];

export function ChangeCommandExamplesModal({
  opened,
  onClose,
}: ChangeCommandExamplesModalProps) {
  return (
    <Modal opened={opened} onClose={onClose} title="コマンド例" size="lg">
      <Stack gap="md">
        <Text size="sm" c="dimmed">
          変更コマンドの構文サンプルです。必要に応じて編集してください。
        </Text>
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
