import { Box, Button, Flex, Group, Paper, Text } from "@mantine/core";
import { CodeMirrorTextarea } from "./CodeMirrorTextarea";

type SimplifiedConfigCardProps = {
  placeholderMessage: string;
  value: string;
  onOpenConfigModal: () => void;
};

export function SimplifiedConfigCard({
  placeholderMessage,
  value,
  onOpenConfigModal,
}: SimplifiedConfigCardProps) {
  return (
    <Paper withBorder radius="md" p="lg" h="100%">
      <Flex direction="column" h="100%" gap="sm">
        <Group justify="space-between" align="center">
          <Text fw={600}>現在の設定</Text>
          <Button size="xs" variant="light" onClick={onOpenConfigModal}>
            Import Config
          </Button>
        </Group>
        <Box flex={1} mih={0}>
          <CodeMirrorTextarea value={value} readOnly placeholder={placeholderMessage} />
        </Box>
      </Flex>
    </Paper>
  );
}
