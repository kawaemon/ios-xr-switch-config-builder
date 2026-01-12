import { Button, Flex, Group, Paper, Text } from "@mantine/core";
import type { Completion, CompletionSource } from "@codemirror/autocomplete";
import { CodeMirrorTextarea } from "./CodeMirrorTextarea";

type ChangeInputCardProps = {
  value: string;
  onChange: (value: string) => void;
  onOpenExamples: () => void;
};

const changeCommandCompletions: Completion[] = [
  {
    label: "interface FortyGigE0/0/0/",
    type: "variable",
    detail: "40G ポートの設定",
  },
  {
    label: "interface HundredGigE0/0/0/",
    type: "variable",
    detail: "100G ポートの設定",
  },
  {
    label: "interface BVI",
    type: "variable",
    detail: "BVI の作成",
  },
  {
    label: "description ",
    type: "text",
    detail: "インターフェースの説明",
  },
  {
    label: "switchport mode trunk",
    type: "keyword",
    detail: "トランクモード",
  },
  {
    label: "switchport trunk allowed vlan add ",
    type: "keyword",
    detail: "トランクに VLAN を追加",
  },
  {
    label: "switchport trunk allowed vlan remove ",
    type: "keyword",
    detail: "トランクから VLAN を除外",
  },
  {
    label: "vlan database",
    type: "keyword",
    detail: "VLAN 定義の開始",
  },
  {
    label: "vlan 300 name ",
    type: "variable",
    detail: "VLAN 名の登録",
  },
];

const changeCommandCompletionSource: CompletionSource = (context) => {
  const word = context.matchBefore(/[-A-Za-z0-9/._\t ]*$/);
  if (word?.text === "" && !context.explicit) {
    return null;
  }

  return {
    from: word ? word.from : context.pos,
    options: changeCommandCompletions,
    validFor: /^[-A-Za-z0-9/._\t ]*$/,
  };
};

export function ChangeInputCard({
  value,
  onChange,
  onOpenExamples,
}: ChangeInputCardProps) {
  return (
    <Paper withBorder radius="md" p="lg" h="100%">
      <Flex direction="column" h="100%" gap="sm">
        <Group justify="space-between" align="center">
          <Text fw={600}>変更コマンド</Text>
          <Button variant="light" size="xs" onClick={onOpenExamples}>
            コマンド例
          </Button>
        </Group>
        <CodeMirrorTextarea
          value={value}
          placeholder="変更を入力してください"
          onChange={onChange}
          showLineNumbers
          completionSource={changeCommandCompletionSource}
        />
      </Flex>
    </Paper>
  );
}
