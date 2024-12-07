import { basicSetup, EditorView, minimalSetup } from "codemirror";
import { javascript } from "@codemirror/lang-javascript";
import { autocompletion } from "@codemirror/autocomplete";
import { WorkerShape } from "@valtown/codemirror-ts/worker";
import * as Comlink from "comlink";
import { StateEffect } from "@codemirror/state";
import "./style.css";
import { renderResultDisplay } from "./test_case";

function editorFromTextArea(
  textarea: HTMLTextAreaElement,
  extensions: typeof minimalSetup
): EditorView {
  let view = new EditorView({ doc: textarea.value, extensions });
  textarea.parentNode?.insertBefore(view.dom, textarea);
  textarea.style.display = "none";

  return view;
}

let typeScriptEnvironment: WorkerShape | undefined = undefined;

async function initTypescriptForCodebox(): Promise<typeof minimalSetup> {
  const {
    tsFacetWorker,
    tsSyncWorker,
    tsLinterWorker,
    tsAutocompleteWorker,
    tsHoverWorker,
  } = await import("@valtown/codemirror-ts");

  if (typeScriptEnvironment === undefined) {
    const innerWorker = new Worker(
      new URL("./typescript_worker.ts", import.meta.url),
      {
        type: "module",
      }
    );
    const worker = Comlink.wrap(innerWorker) as WorkerShape;
    await worker.initialize();
    typeScriptEnvironment = worker;
  }
  const path = "/src/index.ts";

  return [
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
    tsHoverWorker(),
  ];
}

const setupEditorControls = (
  editorControls: HTMLElement,
  mainTextArea: EditorView
) => {
  editorControls.classList.remove("hidden");
  const byteCountElement = editorControls.querySelector("#byte-counter")!;
  const resetButton = editorControls.querySelector("#restore-solution-button")!;
  const textEncoder = new TextEncoder();
  const originalText = mainTextArea.state.doc.toString();
  const lengthInBytes = (s: string): number => textEncoder.encode(s).length;

  byteCountElement.textContent = lengthInBytes(originalText).toString();

  mainTextArea.dispatch({
    effects: StateEffect.appendConfig.of([
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          byteCountElement.textContent = lengthInBytes(
            mainTextArea.state.doc.toString()
          ).toString();
        }
      }),
    ]),
  });

  resetButton.addEventListener("click", () => {
    mainTextArea.dispatch({
      changes: {
        from: 0,
        to: mainTextArea.state.doc.length,
        insert: originalText,
      },
    });
  });
};

window.addEventListener("load", async () => {
  let mainTextArea: EditorView;

  for (const textarea of document.querySelectorAll<HTMLTextAreaElement>(
    "textarea.codemirror"
  )) {
    let plugins: typeof basicSetup = [basicSetup, EditorView.lineWrapping];
    console.log("Replacing textarea with codemirror");
    let view = editorFromTextArea(textarea, plugins);

    if (textarea.classList.contains("lang-typescript")) {
      initTypescriptForCodebox().then((plugin) => {
        view.dispatch({
          effects: StateEffect.appendConfig.of(plugin),
        });
      });
    }
    if (textarea.id === "main-code") {
      mainTextArea = view;
    }
  }

  let editorControls = document.getElementById("editor-controls");
  if (editorControls !== null) {
    console.log("editor controls exists");
    setupEditorControls(editorControls, mainTextArea!);

    setupJsSubmitOnForm(mainTextArea!);
  }
});

/// Only works from the solutions page
async function submitNewSolution(mainTextArea: EditorView) {
  const content = mainTextArea.state.doc.toString();

  const response = await fetch(window.location.href, {
    method: "POST",
    headers: {
      accept: "application/json",
      "content-type": "application/json",
    },
    body: JSON.stringify({
      code: content,
    }),
  });

  // todo: Also render the leaderboard
  const { tests } = await response.json();
  const testsContainer = document.querySelector(
    "div.result-display-wrapper"
  ) as HTMLDivElement;
  renderResultDisplay(tests, testsContainer);
}

function setupJsSubmitOnForm(mainTextArea: EditorView) {
  const form = document.querySelector("form");

  form.addEventListener("submit", (ev) => {
    ev.preventDefault();

    submitNewSolution(mainTextArea);
  });
}
