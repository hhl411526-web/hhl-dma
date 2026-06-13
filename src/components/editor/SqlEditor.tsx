import { useRef, useCallback } from "react";
import Editor, { OnMount } from "@monaco-editor/react";
import type { editor, languages, Position } from "monaco-editor";

interface SqlEditorProps {
  value: string;
  onChange: (value: string) => void;
  onExecute: () => void;
  dialect?: string;
}

export default function SqlEditor({ value, onChange, onExecute }: SqlEditorProps) {
  const editorRef = useRef<editor.IStandaloneCodeEditor | null>(null);

  const handleMount: OnMount = useCallback((editor, monaco) => {
    editorRef.current = editor;
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter, () => {
      onExecute();
    });
    monaco.languages.registerCompletionItemProvider("sql", {
      provideCompletionItems: (model: editor.ITextModel, position: Position): languages.CompletionList => {
        const word = model.getWordUntilPosition(position);
        const range = {
          startLineNumber: position.lineNumber, endLineNumber: position.lineNumber,
          startColumn: word.startColumn, endColumn: word.endColumn,
        };
        const suggestions = [
          { label: "SELECT", kind: monaco.languages.CompletionItemKind.Keyword, insertText: "SELECT ", range },
          { label: "FROM", kind: monaco.languages.CompletionItemKind.Keyword, insertText: "FROM ", range },
          { label: "WHERE", kind: monaco.languages.CompletionItemKind.Keyword, insertText: "WHERE ", range },
          { label: "INSERT", kind: monaco.languages.CompletionItemKind.Keyword, insertText: "INSERT INTO ", range },
          { label: "UPDATE", kind: monaco.languages.CompletionItemKind.Keyword, insertText: "UPDATE ", range },
          { label: "DELETE", kind: monaco.languages.CompletionItemKind.Keyword, insertText: "DELETE FROM ", range },
          { label: "CREATE TABLE", kind: monaco.languages.CompletionItemKind.Snippet, insertText: "CREATE TABLE ${1:table_name} (\n\t$0\n);", insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet, range },
        ];
        return { suggestions };
      },
    });
  }, [onExecute]);

  return (
    <Editor
      height="100%"
      language="sql"
      value={value}
      onChange={(v) => onChange(v || "")}
      onMount={handleMount}
      theme="vs"
      options={{
        minimap: { enabled: false },
        fontSize: 14,
        lineNumbers: "on",
        wordWrap: "on",
        scrollBeyondLastLine: false,
        automaticLayout: true,
        tabSize: 2,
        padding: { top: 8 },
      }}
    />
  );
}
