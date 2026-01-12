import { Alert, Box, Flex, Paper, Text, ActionIcon, Tooltip } from "@mantine/core";
import { IconAlertCircle, IconCopy } from "@tabler/icons-react";
import { CodeMirrorTextarea } from "./CodeMirrorTextarea";

type GeneratedChangeCardProps = {
  value: string;
  errorMessage: string;
};

export function GeneratedChangeCard({
  value,
  errorMessage,
}: GeneratedChangeCardProps) {
  return (
    <Paper withBorder radius="md" p="lg" h="100%">
      <Flex direction="column" h="100%" gap="sm">
        <Text fw={600}>ncs config</Text>
        {errorMessage && (
          <Alert
            variant="light"
            color="red"
            radius="md"
            icon={<IconAlertCircle size={16} />}
            title="生成に失敗しました"
          >
            <Text size="sm">{errorMessage}</Text>
          </Alert>
        )}
        <Box pos="relative" flex={1} mih={0}>
          <CodeMirrorTextarea
            value={value}
            readOnly
            placeholder="生成結果がここに表示されます"
          />
          <Tooltip
            label={
              value ? "クリップボードにコピー" : "コピーする内容がありません"
            }
          >
            <ActionIcon
              variant="light"
              color="gray"
              aria-label="コピー"
              onClick={() => {
                if (!value) return;
                void navigator.clipboard.writeText(value);
              }}
              disabled={!value}
              pos="absolute"
              top={8}
              right={8}
            >
              <IconCopy size={16} />
            </ActionIcon>
          </Tooltip>
        </Box>
      </Flex>
    </Paper>
  );
}
