import { Box, Flex, Paper, Text } from "@mantine/core";
import { CodeMirrorTextarea } from "./CodeMirrorTextarea";

type SimplifiedConfigCardProps = {
  placeholderMessage: string;
  value: string;
};

export function SimplifiedConfigCard({
  placeholderMessage,
  value,
}: SimplifiedConfigCardProps) {
  return (
    <Paper withBorder radius="md" p="lg" h="100%">
      <Flex direction="column" h="100%" gap="sm">
        <Text fw={600}>現在の設定</Text>
        <Box flex={1} mih={0}>
          <CodeMirrorTextarea
            value={value}
            readOnly
            placeholder={placeholderMessage}
          />
        </Box>
      </Flex>
    </Paper>
  );
}
