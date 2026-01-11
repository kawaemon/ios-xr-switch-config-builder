import {
  ActionIcon,
  Button,
  Group,
  Stack,
  Text,
  Title,
  useComputedColorScheme,
  useMantineColorScheme,
} from "@mantine/core";
import { IconMoon, IconSun } from "@tabler/icons-react";

type LintStatusHeaderProps = {
  showLintDetailButton: boolean;
  onOpenLintModal: () => void;
  onOpenConfigModal: () => void;
};

export function LintStatusHeader({
  showLintDetailButton,
  onOpenLintModal,
  onOpenConfigModal,
}: LintStatusHeaderProps) {
  const computedColorScheme = useComputedColorScheme("light");
  const { colorScheme, setColorScheme } = useMantineColorScheme();
  const isDark = computedColorScheme === "dark";
  const toggleLabel = isDark ? "ライトモード" : "ダークモード";
  const handleToggleScheme = () => {
    if (colorScheme === "auto") {
      setColorScheme(isDark ? "light" : "dark");
      return;
    }

    setColorScheme(colorScheme === "dark" ? "light" : "dark");
  };

  return (
    <Group justify="space-between" align="flex-start" wrap="wrap">
      <Stack gap={4} maw={600}>
        <Title order={1}>NCS Config Builder</Title>
        <Text c="dimmed">
          NCS configを一般スイッチ向けの見やすい形式で確認できます。
        </Text>
      </Stack>
      <Group gap="sm" align="center">
        <ActionIcon
          variant="subtle"
          size="lg"
          onClick={handleToggleScheme}
          aria-label={toggleLabel}
        >
          {isDark ? <IconMoon size={20} /> : <IconSun size={20} />}
        </ActionIcon>
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
