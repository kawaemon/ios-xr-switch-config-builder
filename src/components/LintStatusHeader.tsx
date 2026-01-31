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
import { IconBrandGithub, IconMoon, IconSun } from "@tabler/icons-react";

type LintStatusHeaderProps = {
  showLintDetailButton: boolean;
  onOpenLintModal: () => void;
};

export function LintStatusHeader({ showLintDetailButton, onOpenLintModal }: LintStatusHeaderProps) {
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
        <Title order={1}>IOS XR Switch Config Builder</Title>
        <Text c="dimmed">
          IOS XR での L2 設定をを一般スイッチ向けの見やすい形式で確認できます。
        </Text>
      </Stack>
      <Group gap="sm" align="center">
        <ActionIcon
          component="a"
          href="https://github.com/kawaemon/ios-xr-switch-config-builder"
          target="_blank"
          rel="noopener noreferrer"
          variant="subtle"
          size="lg"
          aria-label="GitHub repository"
        >
          <IconBrandGithub size={18} />
        </ActionIcon>
        <ActionIcon
          variant="subtle"
          size="lg"
          onClick={handleToggleScheme}
          aria-label={toggleLabel}
        >
          {isDark ? <IconMoon size={20} /> : <IconSun size={20} />}
        </ActionIcon>
        {showLintDetailButton ? (
          <Button variant="subtle" size="compact-sm" onClick={onOpenLintModal}>
            Lint詳細
          </Button>
        ) : null}
      </Group>
    </Group>
  );
}
