import {
  Alert,
  Paper,
  Stack,
  Text,
  Textarea,
  ActionIcon,
  Tooltip,
} from "@mantine/core";
import { IconAlertCircle, IconCopy } from "@tabler/icons-react";

type GeneratedChangeCardProps = {
  value: string;
  errorMessage: string;
};

export function GeneratedChangeCard({
  value,
  errorMessage,
}: GeneratedChangeCardProps) {
  return (
    <Paper withBorder radius="md" p="lg" style={{ height: "100%" }}>
      <Stack gap="sm" style={{ height: "100%" }}>
        <div>
          <Text fw={600}>ncs config</Text>
        </div>
        {errorMessage ? (
          <Alert
            variant="light"
            color="red"
            radius="md"
            icon={<IconAlertCircle size={16} />}
            title="生成に失敗しました"
          >
            <Text size="sm">{errorMessage}</Text>
          </Alert>
        ) : null}
        <div style={{ position: "relative", flex: 1, minHeight: 0 }}>
          <Textarea
            value={value}
            readOnly
            autosize={false}
            minRows={12}
            spellCheck={false}
            style={{ height: "100%" }}
            styles={{
              input: {
                fontFamily: "var(--mantine-font-family-monospace)",
                height: "100%",
                minHeight: 0,
                overflowY: "auto",
                paddingRight: "3rem",
              },
            }}
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
              style={{ position: "absolute", top: "8px", right: "8px" }}
            >
              <IconCopy size={16} />
            </ActionIcon>
          </Tooltip>
        </div>
      </Stack>
    </Paper>
  );
}
