import { useMemo, useState } from "react";

import defaultConfig from "../.kprivate/config.txt?raw";
import { Container, Stack } from "@mantine/core";
import * as wasmModule from "./wasm/pkg/ncs_wasm";
import { LintStatusHeader } from "./components/LintStatusHeader";
import { ConfigEditorModal } from "./components/ConfigEditorModal";
import { LintResultModal } from "./components/LintResultModal";
import { SimplifiedConfigCard } from "./components/SimplifiedConfigCard";
import { ChangeInputCard } from "./components/ChangeInputCard";
import { GeneratedChangeCard } from "./components/GeneratedChangeCard";

type NcsWasmModule = typeof wasmModule & {
  generate_change_config: (
    baseConfig: string,
    changeInput: string
  ) => { changeOutput: string };
};

const wasm = wasmModule as NcsWasmModule;

const demoChangeInput = [
  "vlan database",
  "  vlan 350 name demo-servers",
  "  vlan 500 name demo-mgmt",
  "",
  "interface FortyGigE0/0/0/46",
  "  switchport trunk allowed vlan add 350",
  "  switchport trunk allowed vlan remove 300",
  "",
  "interface HundredGigE0/0/0/23",
  "  description To:demo-port",
  "  switchport mode trunk",
  "  switchport trunk allowed vlan add 350 500",
  "",
  "interface HundredGigE0/0/0/24",
  "  description To:demo-access",
  "  switchport mode access",
  "  switchport access vlan 500",
  "",
  "interface BVI500",
].join("\n");

function App() {
  const [src, setSrc] = useState(defaultConfig);
  const [changeInput, setChangeInput] = useState(demoChangeInput);
  const [isConfigModalOpen, setConfigModalOpen] = useState(false);
  const [draftConfig, setDraftConfig] = useState(defaultConfig);
  const [isLintModalOpen, setLintModalOpen] = useState(false);
  const isConfigEmpty = src.trim().length === 0;
  const currentConfig = useMemo(() => {
    return wasm.analyze_config(src);
  }, [src]);
  const lintOutput = currentConfig.lintOutput.trim();
  const hasLintIssues = lintOutput.length > 0;
  const changeResult = useMemo(() => {
    if (changeInput.trim().length === 0) {
      return { changeOutput: "", errorMessage: "" };
    }

    try {
      const result = wasm.generate_change_config(src, changeInput);
      return { changeOutput: result.changeOutput, errorMessage: "" };
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      return { changeOutput: "", errorMessage };
    }
  }, [src, changeInput]);
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

          <ChangeInputCard value={changeInput} onChange={setChangeInput} />

          <GeneratedChangeCard
            value={changeResult.changeOutput}
            errorMessage={changeResult.errorMessage}
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
