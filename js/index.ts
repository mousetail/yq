import { basicSetup, EditorView, minimalSetup } from 'codemirror';
import {
    createDefaultMapFromCDN,
    createSystem,
    createVirtualTypeScriptEnvironment,
    VirtualTypeScriptEnvironment,
} from "@typescript/vfs";
import ts from "typescript";
import { tsSync, tsFacet, tsAutocomplete, tsLinter, tsHover, tsFacetWorker, tsSyncWorker, tsLinterWorker, tsAutocompleteWorker, tsHoverWorker } from "@valtown/codemirror-ts";
import { javascript } from '@codemirror/lang-javascript';
import { autocompletion } from "@codemirror/autocomplete";
import { WorkerShape } from '@valtown/codemirror-ts/worker';
import * as Comlink from "comlink";

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

let typeScriptEnvironment: WorkerShape | undefined = undefined;

async function initTypescriptForCodebox(): Promise<typeof minimalSetup> {
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

window.addEventListener('load', async () => {
    for (const textarea of document.querySelectorAll<HTMLTextAreaElement>('textarea.codemirror')) {

        let plugins = basicSetup;
        if (textarea.classList.contains('lang-typescript')) {
            plugins = await initTypescriptForCodebox()
        }
        console.log("Replacing textarea with codemirror");
        editorFromTextArea(textarea, plugins);
    }
});

