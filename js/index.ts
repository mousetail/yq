import { EditorView, minimalSetup } from 'codemirror';

function editorFromTextArea(textarea: HTMLTextAreaElement, extensions: typeof minimalSetup) {
    let view = new EditorView({ doc: textarea.value, extensions })
    textarea.parentNode?.insertBefore(view.dom, textarea)
    textarea.style.display = "none"
    if (textarea.form) {
        textarea.form.addEventListener("submit", () => {
            textarea.value = view.state.doc.toString()
        });
    }
    return view
}

document.addEventListener('DOMContentLoaded', () => {
    for (const textarea of document.querySelectorAll<HTMLTextAreaElement>('textarea.codemirror')) {
        editorFromTextArea(textarea, minimalSetup);
    }
})


