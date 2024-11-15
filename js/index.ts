import { basicSetup, EditorView, minimalSetup } from 'codemirror';
import { javascript } from '@codemirror/lang-javascript';
import { autocompletion } from "@codemirror/autocomplete";
import { WorkerShape } from '@valtown/codemirror-ts/worker';
import * as Comlink from "comlink";
import { StateEffect } from "@codemirror/state"
import "./style.css";

function editorFromTextArea(textarea: HTMLTextAreaElement, extensions: typeof minimalSetup): EditorView {
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

let typeScriptEnvironment: WorkerShape | undefined = undefined;

async function initTypescriptForCodebox(): Promise<typeof minimalSetup> {
    const { tsFacetWorker, tsSyncWorker, tsLinterWorker, tsAutocompleteWorker, tsHoverWorker } = await import("@valtown/codemirror-ts");

    if (typeScriptEnvironment === undefined) {
        const innerWorker = new Worker(new URL("./typescript_worker.ts", import.meta.url), {
            type: "module",
        });
        const worker = Comlink.wrap(innerWorker) as WorkerShape;
        await worker.initialize();
        typeScriptEnvironment = worker;
    }
    const path = '/src/index.ts'

    return [
        basicSetup,
        javascript({
            typescript: true,
            jsx: true,
        }),
        tsFacetWorker.of({ worker: typeScriptEnvironment, path }),
        tsSyncWorker(),
        tsLinterWorker(),
        autocompletion({
            override: [tsAutocompleteWorker()],
        }),
        tsHoverWorker()
    ]
}

const setupEditorControls = (editorControls: HTMLElement, mainTextArea: EditorView) => {
    editorControls.classList.remove('hidden');
    const byteCountElement = editorControls.querySelector('#byte-counter')!;
    const resetButton = editorControls.querySelector('#restore-solution-button')!;
    const textEncoder = new TextEncoder();
    const originalText = mainTextArea.state.doc.toString();

    byteCountElement.textContent = '' + textEncoder.encode(originalText).length;

    mainTextArea.dispatch(
        {
            effects: StateEffect.appendConfig.of([
                EditorView.updateListener.of((update) => {
                    if (update.docChanged) {
                        byteCountElement.textContent = '' +
                            [...mainTextArea.state.doc.iterLines()].reduce(
                                (a, b) => a + textEncoder.encode(b).length + 1,
                                0
                            );
                    }
                })
            ])
        }
    )

    resetButton.addEventListener('click', () => {
        mainTextArea.dispatch(
            {
                changes: {
                    from: 0,
                    to: mainTextArea.state.doc.length,
                    insert: originalText
                }
            }
        )
    })
}

window.addEventListener('load', async () => {
    let mainTextArea: EditorView;

    for (const textarea of document.querySelectorAll<HTMLTextAreaElement>('textarea.codemirror')) {

        let plugins = basicSetup;
        if (textarea.classList.contains('lang-typescript')) {
            plugins = await initTypescriptForCodebox()
        }
        console.log("Replacing textarea with codemirror");
        let view = editorFromTextArea(textarea, plugins);
        if (textarea.id === 'main-code') {
            mainTextArea = view;
        }
    };


    let editorControls = document.getElementById('editor-controls');
    if (editorControls !== null) {
        console.log("editor controls exists");
        setupEditorControls(editorControls, mainTextArea!);
    }
});

