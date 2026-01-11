import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "@mantine/core/styles.css";
import App from "./App.tsx";
import { createTheme, Loader, MantineProvider, Stack, Text } from "@mantine/core";
import { initWasm } from "./wasm/index.ts";

const theme = createTheme({});

function LoadingScreen() {
  return (
    <MantineProvider theme={theme} defaultColorScheme="auto">
      <Stack
        align="center"
        justify="center"
        style={{ height: "100vh" }}
        gap="md"
      >
        <Loader size="xl" />
        <Text size="lg" c="dimmed">
          WASM モジュールを読み込み中...
        </Text>
      </Stack>
    </MantineProvider>
  );
}

async function main() {
  const root = createRoot(document.getElementById("root")!);

  root.render(
    <StrictMode>
      <LoadingScreen />
    </StrictMode>
  );

  await initWasm();

  root.render(
    <StrictMode>
      <MantineProvider theme={theme} defaultColorScheme="auto">
        <App />
      </MantineProvider>
    </StrictMode>
  );
}

main().catch(console.error);
