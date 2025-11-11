import { Badge, Button, Group, Stack, Text, Title } from "@mantine/core";

type LintStatusHeaderProps = {
  lintBadgeColor: string;
  lintBadgeLabel: string;
  showLintDetailButton: boolean;
  onOpenLintModal: () => void;
  onOpenConfigModal: () => void;
};

export function LintStatusHeader({
  lintBadgeColor,
  lintBadgeLabel,
  showLintDetailButton,
  onOpenLintModal,
  onOpenConfigModal,
}: LintStatusHeaderProps) {
  return (
    <Group justify="space-between" align="flex-start" wrap="wrap">
      <Stack gap={4} maw={600}>
        <Title order={1}>NCS Config Lint</Title>
        <Text c="dimmed">
          Lint結果と bridge VLAN 情報を中心にコンフィグを確認できます。
        </Text>
      </Stack>
      <Group gap="sm" align="center">
        <Badge color={lintBadgeColor} variant="light">
          {lintBadgeLabel}
        </Badge>
        {showLintDetailButton ? (
          <Button
            variant="subtle"
            size="compact-sm"
            onClick={onOpenLintModal}
          >
            Lint詳細
          </Button>
        ) : null}
        <Button onClick={onOpenConfigModal}>Configを編集</Button>
      </Group>
    </Group>
  );
}
