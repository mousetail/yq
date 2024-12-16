import { basicSetup, EditorView, minimalSetup } from "codemirror";
import { keymap } from "@codemirror/view";
import { indentWithTab } from "@codemirror/commands";
import { javascript } from "@codemirror/lang-javascript";
import { indentUnit } from "@codemirror/language";
import { autocompletion } from "@codemirror/autocomplete";
import { WorkerShape } from "@valtown/codemirror-ts/worker";
import * as Comlink from "comlink";
import { StateEffect } from "@codemirror/state";
import "./style.css";
import { renderResultDisplay, ResultDisplay } from "./test_case";

function editorFromTextArea(
  textarea: HTMLTextAreaElement,
  extensions: typeof minimalSetup,
  swapOnSubmit: boolean
): EditorView {
  let view = new EditorView({ doc: textarea.value, extensions });
  textarea.parentNode?.insertBefore(view.dom, textarea);
  if (swapOnSubmit) {
    textarea.style.display = "none";
  } else {
    textarea.parentElement.removeChild(textarea);
  }

  if (swapOnSubmit && textarea.form) {
    textarea.form.addEventListener("submit", () => {
      textarea.value = view.state.doc.toString();
    });
  }

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
  let originalText = mainTextArea.state.doc.toString();
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

  setupJsSubmitOnForm(mainTextArea!, (e) => {
    originalText = e;
  });
};

window.addEventListener("load", async () => {
  let mainTextArea: EditorView;

  let leaderboardForm: HTMLFormElement | undefined;
  if ((leaderboardForm = document.querySelector(".leaderboard-tabs-form"))) {
    setupLeaderboardForm(leaderboardForm);
  }

  for (const textarea of document.querySelectorAll<HTMLTextAreaElement>(
    "textarea.codemirror"
  )) {
    let plugins: typeof basicSetup = [
      basicSetup,
      keymap.of([indentWithTab]),
      indentUnit.of("\t"),
      EditorView.lineWrapping,
    ];
    console.log("Replacing textarea with codemirror");
    let view = editorFromTextArea(
      textarea,
      plugins,
      textarea.id !== "main-code"
    );

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
    setupEditorControls(editorControls, mainTextArea!);
  }
});

/// Only works from the solutions page
async function submitNewSolution(
  mainTextArea: EditorView,
  submitButton: HTMLButtonElement,
  setOriginalText: (e: string) => void
) {
  submitButton.disabled = true;
  try {
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

    const errorDiv = document.querySelector(".solution-submit-error");
    if (![200, 201, 400].includes(response.status)) {
      errorDiv.textContent = await response.text();
      errorDiv.classList.remove("hidden");
      return;
    }
    errorDiv.classList.add("hidden");

    // todo: Also render the leaderboard
    const { tests, leaderboard } = (await response.json()) as {
      tests: ResultDisplay;
      leaderboard: LeaderboardEntry[];
    };
    updateLeaderbaord(leaderboard);

    if (tests.passed && response.status === 201) {
      setOriginalText(content);
    }
    const testsContainer = document.querySelector(
      "div.result-display-wrapper"
    ) as HTMLDivElement;
    renderResultDisplay(tests, testsContainer);
  } finally {
    submitButton.disabled = false;
  }
}

function setupJsSubmitOnForm(
  mainTextArea: EditorView,
  setOriginalText: (e: string) => void
) {
  const form = document.querySelector("form.challenge-submission-form");
  const submitButton = form.querySelector(
    "button[type='submit']"
  ) as HTMLButtonElement;

  form.addEventListener("submit", (ev) => {
    ev.preventDefault();

    submitNewSolution(mainTextArea, submitButton, setOriginalText);
  });
}

function setupLeaderboardForm(form: HTMLFormElement) {
  form.addEventListener("submit", (ev) => {
    ev.preventDefault();
  });

  form.querySelectorAll("button").forEach((button) => {
    const languageName = window.location.pathname.split("/").pop();

    button.addEventListener("click", async () => {
      changeActiveLeaderboardTab(button.value);

      const response = await fetch(
        `../leaderboard/${languageName}?ranking=${encodeURIComponent(
          button.value
        )}`,
        {
          headers: {
            accept: "application/json",
            "content-type": "application/json",
          },
        }
      );

      if (!response.ok) {
        console.error(await response.json());
      }

      updateLeaderbaord(await response.json());
    });
  });
}

type LeaderboardEntry = {
  rank: number;
  author_avatar: string;
  author_name: string;
  author_id: number;
  score: number;
};

function updateLeaderbaord(ranking: LeaderboardEntry[]) {
  const leaderboard = document.querySelector(".leaderboard table tbody");

  leaderboard.replaceChildren(
    ...ranking.map((entry: LeaderboardEntry) => {
      const row = document.createElement("tr");

      const rankCell = document.createElement("td");
      rankCell.textContent = `#${entry.rank}`;
      row.appendChild(rankCell);

      const avatarCell = document.createElement("td");
      const pfp = document.createElement("img");
      pfp.src = entry.author_avatar;
      avatarCell.appendChild(pfp);
      row.appendChild(avatarCell);

      const authorNameCell = document.createElement("td");
      const link = document.createElement("a");
      link.href = `/user/${entry.author_id}`;
      link.textContent = entry.author_name;
      authorNameCell.appendChild(link);
      row.appendChild(authorNameCell);

      const scoreCell = document.createElement("td");
      scoreCell.textContent = `${entry.score}`;
      row.appendChild(scoreCell);

      return row;
    })
  );
}

function changeActiveLeaderboardTab(tab: string) {
  document
    .querySelector(`.leaderboard-tabs-form button[value=${tab}]`)
    .classList.add("active");
  document
    .querySelector(`.leaderboard-tabs-form button:not([value=${tab}])`)
    .classList.remove("active");
}
