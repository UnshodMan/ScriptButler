import { EditorView, keymap } from '@codemirror/view'
import { basicSetup } from 'codemirror'
import { oneDark } from '@codemirror/theme-one-dark'
import { languageExtension } from './language'

export interface EditorOptions {
  doc: string
  langId: string
  /** 已保存内容的基线，保存后外部更新其 value 即可重置脏状态比较 */
  baseline: { value: string }
  onSave: () => void
  onDirtyChange: (dirty: boolean) => void
}

export function createEditor(parent: HTMLElement, opts: EditorOptions): EditorView {
  const lang = languageExtension(opts.langId)
  return new EditorView({
    parent,
    doc: opts.doc,
    extensions: [
      basicSetup,
      oneDark,
      keymap.of([
        {
          key: 'Mod-s',
          run: () => {
            opts.onSave()
            return true
          },
        },
      ]),
      ...(lang ? [lang] : []),
      EditorView.updateListener.of((u) => {
        if (u.docChanged) opts.onDirtyChange(u.state.doc.toString() !== opts.baseline.value)
      }),
      EditorView.theme({
        '&': { height: '100%', fontSize: '13px' },
        '.cm-scroller': { overflow: 'auto' },
        '.cm-content': {
          fontFamily: 'ui-monospace, "JetBrains Mono", Menlo, Consolas, monospace',
        },
      }),
    ],
  })
}
