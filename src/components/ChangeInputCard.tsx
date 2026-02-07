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
    label: "interface Bundle-Ether",
    type: "variable",
    detail: "LAG ポートの設定",
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
    label: "switchport trunk allowed vlan none",
    type: "keyword",
    detail: "トランクリストを全削除",
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
  const word = context.matchBefore(/\S*$/);
  if (word?.text === "" && !context.explicit) {
    return null;
  }

  const line = context.state.doc.lineAt(context.pos);
  const lineText = line.text.trimStart();

  const filteredOptions = changeCommandCompletions.filter((opt) => {
    const optLabel = opt.label;

    // 行が空の場合、すべての候補を表示
    if (lineText.trim() === "") {
      return true;
    }

    // 候補が行のテキストで始まっている場合のみ表示
    if (!optLabel.startsWith(lineText)) {
      return false;
    }

    // 完全一致の場合は除外
    if (lineText.trim() === optLabel.trim()) {
      return false;
    }

    return true;
  });

  const lineContentFrom = line.from + line.text.search(/\S|$/);

  return {
    from: lineContentFrom,
    options: filteredOptions,
    validFor: /^.*$/,
  };
};

export function ChangeInputCard({ value, onChange, onOpenExamples }: ChangeInputCardProps) {
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
