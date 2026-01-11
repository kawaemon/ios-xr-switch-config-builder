import { useMemo, useState } from "react";

import defaultConfig from "../.kprivate/config.txt?raw";
import { Container, Stack } from "@mantine/core";
import { main } from "./a0_business/a1_semantics";
import { LintStatusHeader } from "./components/LintStatusHeader";
import {
  BridgeSummaryCard,
  type BridgeTableRow,
} from "./components/BridgeSummaryCard";
import { ConfigEditorModal } from "./components/ConfigEditorModal";
import { LintResultModal } from "./components/LintResultModal";

function App() {
  const [src, setSrc] = useState(defaultConfig);
  const [isConfigModalOpen, setConfigModalOpen] = useState(false);
  const [draftConfig, setDraftConfig] = useState(defaultConfig);
  const [isLintModalOpen, setLintModalOpen] = useState(false);
  const isConfigEmpty = src.trim().length === 0;
  const currentConfig = useMemo(() => {
    return main(src);
  }, [src]);
  const lintOutput = currentConfig.lint().trim();
  const hasLintIssues = lintOutput.length > 0;
  const bridgeTableRows = useMemo<BridgeTableRow[]>(() => {
    if (hasLintIssues) {
      return [];
    }

    const baseToVlanMap = new Map<string, Set<number>>();
    for (const domain of currentConfig.domains) {
      for (const interfaceName of domain.interfaces) {
        const splitIndex = interfaceName.indexOf(".");
        if (splitIndex === -1) {
          continue;
        }

        const baseInterface = interfaceName.slice(0, splitIndex);
        const vlanSet = baseToVlanMap.get(baseInterface) ?? new Set<number>();
        vlanSet.add(domain.vlanTag);
        baseToVlanMap.set(baseInterface, vlanSet);
      }
    }

    return Array.from(baseToVlanMap.entries())
      .map(([baseInterface, vlanSet]) => ({
        baseInterface,
        vlanTags: Array.from(vlanSet).sort((a, b) => a - b),
      }))
      .sort((a, b) => a.baseInterface.localeCompare(b.baseInterface));
  }, [currentConfig, hasLintIssues]);
  const showBridgeSummary = !hasLintIssues && bridgeTableRows.length > 0;
  const openConfigModal = () => {
    setDraftConfig(src);
    setConfigModalOpen(true);
  };
  const lintBadgeColor = isConfigEmpty
    ? "gray"
    : hasLintIssues
    ? "orange"
    : "green";
  const lintBadgeLabel = isConfigEmpty
    ? "未入力"
    : hasLintIssues
    ? "Lint要確認"
    : "Lint OK";
  const showLintDetailButton = !isConfigEmpty && hasLintIssues;
  const bridgeSectionMessage = isConfigEmpty
    ? "Configを入力すると、Bridge VLAN の詳細が表示されます。"
    : "Lint指摘をすべて解消すると、Bridge VLAN の詳細が表示されます。";

  return (
    <>
      <Container size="lg" py="xl">
        <Stack gap="xl">
          <LintStatusHeader
            lintBadgeColor={lintBadgeColor}
            lintBadgeLabel={lintBadgeLabel}
            showLintDetailButton={showLintDetailButton}
            onOpenLintModal={() => setLintModalOpen(true)}
            onOpenConfigModal={openConfigModal}
          />

          <BridgeSummaryCard
            isConfigEmpty={isConfigEmpty}
            showBridgeSummary={showBridgeSummary}
            rows={bridgeTableRows}
            placeholderMessage={bridgeSectionMessage}
          />
        </Stack>
      </Container>

      <ConfigEditorModal
        opened={isConfigModalOpen}
        draftConfig={draftConfig}
        onChangeDraft={setDraftConfig}
        onCancel={() => setConfigModalOpen(false)}
        onConfirm={() => {
          setSrc(draftConfig);
          setConfigModalOpen(false);
        }}
      />

      <LintResultModal
        opened={isLintModalOpen}
        lintOutput={lintOutput}
        onClose={() => setLintModalOpen(false)}
      />
    </>
  );
}

export default App;
