import { useMemo } from "react";
import { useComputedColorScheme } from "@mantine/core";
import CodeMirror from "@uiw/react-codemirror";
import { autocompletion } from "@codemirror/autocomplete";
import type { CompletionSource } from "@codemirror/autocomplete";
import { EditorView, lineNumbers } from "@codemirror/view";

export type CodeMirrorTextareaProps = {
  value: string;
  onChange?: (value: string) => void;
  readOnly?: boolean;
  placeholder?: string;
  minRows?: number;
  showLineNumbers?: boolean;
  completionSource?: CompletionSource;
  height?: string;
};

export function CodeMirrorTextarea({
  value,
  onChange,
  readOnly = false,
  placeholder,
  minRows = 10,
  showLineNumbers = false,
  completionSource,
  height,
}: CodeMirrorTextareaProps) {
  const resolvedHeight = useMemo(() => {
    if (height) {
      return height;
    }

    const lineHeight = 1.55;
    const heightEm = minRows * lineHeight;
    return `calc(${heightEm}em + 1rem)`;
  }, [height, minRows]);

  const colorScheme = useComputedColorScheme("light");
  const themeExtension = useMemo(
    () =>
      EditorView.theme(
        {
          "&": {
            backgroundColor: "var(--mantine-color-body)",
            color: "var(--mantine-color-text)",
          },
          ".cm-editor": {
            backgroundColor: "var(--mantine-color-body)",
          },
          ".cm-content": {
            caretColor: "var(--mantine-color-text)",
          },
          ".cm-lineNumbers .cm-gutterElement": {
            color: "var(--mantine-color-dimmed)",
          },
          ".cm-gutters": {
            backgroundColor: "var(--mantine-color-body)",
            borderRight: "1px solid var(--mantine-color-default-border)",
          },
          ".cm-placeholder": {
            color: "var(--mantine-color-dimmed)",
          },
          ".cm-selectionBackground, .cm-content ::selection": {
            backgroundColor: "var(--mantine-color-default-hover)",
          },
          ".cm-activeLine, .cm-activeLineGutter": {
            backgroundColor: "var(--mantine-color-default-hover)",
          },
          ".cm-scroller": {
            fontFamily: "var(--mantine-font-family-monospace)",
          },
        },
        { dark: colorScheme === "dark" }
      ),
    [colorScheme]
  );

  const extensions = useMemo(() => {
    const base = [EditorView.lineWrapping, themeExtension];
    if (showLineNumbers) {
      base.unshift(lineNumbers());
    }
    if (completionSource) {
      base.push(
        autocompletion({
          override: [completionSource],
          activateOnTyping: true,
        })
      );
    }
    return base;
  }, [showLineNumbers, themeExtension, completionSource]);

  return (
    <CodeMirror
      value={value}
      height={resolvedHeight}
      theme={colorScheme === "dark" ? "dark" : "light"}
      basicSetup={{
        lineNumbers: false,
        foldGutter: false,
        highlightActiveLine: false,
        highlightSelectionMatches: false,
      }}
      extensions={extensions}
      editable={!readOnly}
      placeholder={placeholder}
      style={{
        fontFamily: "var(--mantine-font-family-monospace)",
        border: "1px solid var(--mantine-color-default-border)",
        borderRadius: "var(--mantine-radius-sm)",
        backgroundColor: "var(--mantine-color-body)",
        width: "100%",
      }}
      onChange={onChange}
    />
  );
}
