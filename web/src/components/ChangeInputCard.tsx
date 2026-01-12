import { Paper, Stack, Text } from "@mantine/core";
import { CodeMirrorTextarea } from "./CodeMirrorTextarea";

type ChangeInputCardProps = {
  value: string;
  onChange: (value: string) => void;
};

export function ChangeInputCard({ value, onChange }: ChangeInputCardProps) {
  return (
    <Paper withBorder radius="md" p="lg">
      <Stack gap="sm">
        <div>
          <Text fw={600}>変更コマンド</Text>
        </div>
        <CodeMirrorTextarea
          value={value}
          minRows={14}
          placeholder="変更を入力してください"
          onChange={onChange}
          showLineNumbers
        />
      </Stack>
    </Paper>
  );
}
