import { Paper, Stack, Text, Textarea } from "@mantine/core";

type SimplifiedConfigCardProps = {
  placeholderMessage: string;
  value: string;
};

export function SimplifiedConfigCard({
  placeholderMessage,
  value,
}: SimplifiedConfigCardProps) {
  return (
    <Paper withBorder radius="md" p="lg">
      <Stack gap="sm">
        <div>
          <Text fw={600}>現在の設定</Text>
        </div>
        <Textarea
          value={value}
          readOnly
          minRows={16}
          placeholder={placeholderMessage}
          spellCheck={false}
          styles={{
            input: {
              fontFamily: "var(--mantine-font-family-monospace)",
              minHeight: "22rem",
              maxHeight: "22rem",
              overflowY: "auto",
            },
          }}
        />
      </Stack>
    </Paper>
  );
}
