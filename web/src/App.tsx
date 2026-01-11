import { useMemo, useState } from "react";

import defaultConfig from "../.kprivate/config.txt?raw";
import { Container, Stack } from "@mantine/core";
import * as wasm from "./wasm/pkg/ncs_wasm";
import { LintStatusHeader } from "./components/LintStatusHeader";
import { ConfigEditorModal } from "./components/ConfigEditorModal";
import { LintResultModal } from "./components/LintResultModal";
import { SimplifiedConfigCard } from "./components/SimplifiedConfigCard";

function App() {
  const [src, setSrc] = useState(defaultConfig);
  const [isConfigModalOpen, setConfigModalOpen] = useState(false);
  const [draftConfig, setDraftConfig] = useState(defaultConfig);
  const [isLintModalOpen, setLintModalOpen] = useState(false);
  const isConfigEmpty = src.trim().length === 0;
  const currentConfig = useMemo(() => {
    return wasm.analyze_config(src);
  }, [src]);
  const lintOutput = currentConfig.lintOutput.trim();
  const hasLintIssues = lintOutput.length > 0;
  const openConfigModal = () => {
    setDraftConfig(src);
    setConfigModalOpen(true);
  };
  const showLintDetailButton = !isConfigEmpty && hasLintIssues;
  const simplifiedPlaceholderMessage = isConfigEmpty
    ? "Configを入力すると、簡略化されたconfigが表示されます。"
    : hasLintIssues
    ? "Lint指摘をすべて解消すると、簡略化されたconfigが表示されます。"
    : "変換結果がここに表示されます。";
  const simplifiedConfig =
    !isConfigEmpty && !hasLintIssues ? currentConfig.simplifiedConfig : "";

  return (
    <>
      <Container size="lg" py="xl">
        <Stack gap="xl">
          <LintStatusHeader
            showLintDetailButton={showLintDetailButton}
            onOpenLintModal={() => setLintModalOpen(true)}
            onOpenConfigModal={openConfigModal}
          />

          <SimplifiedConfigCard
            value={simplifiedConfig}
            placeholderMessage={simplifiedPlaceholderMessage}
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
