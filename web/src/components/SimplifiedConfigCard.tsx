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
    <Paper withBorder radius="md" p="lg" style={{ height: "100%" }}>
      <Stack gap="sm" style={{ height: "100%" }}>
        <div>
          <Text fw={600}>現在の設定</Text>
        </div>
        <Textarea
          value={value}
          readOnly
          autosize={false}
          minRows={16}
          placeholder={placeholderMessage}
          spellCheck={false}
          style={{ flex: 1 }}
          styles={{
            input: {
              fontFamily: "var(--mantine-font-family-monospace)",
              height: "100%",
              minHeight: 0,
              overflowY: "auto",
            },
          }}
        />
      </Stack>
    </Paper>
  );
}
