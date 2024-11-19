export type PassState = 'Pass' | 'Fail' | 'Warning' | 'Info';
export type ResultDisplay = { 'Diff': { expected: string, output: string } }
    | { 'Text': string }
    | { 'Run': { input?: string | undefined, output: string, error: string } };

export class TestCase {
    name: string | undefined;
    pass: PassState;
    resultDisplay: ResultDisplay;

    constructor(name: string | undefined, pass: PassState, resultDisplay: ResultDisplay) {
        this.name = name;
        this.pass = pass;
        this.resultDisplay = resultDisplay;
    }

    public setName(name: string): this {
        this.name = name;
        return this;
    }

    public replaceFailState(state: PassState): this {
        if (this.pass === "Fail") {
            this.pass = state;
        }
        return this
    }
}

export class FinalVerdict {
    pass: boolean

    constructor(pass: boolean) {
        this.pass = pass;
    }
}

export type RunCodeResult = {
    stdout: string,
    stderr: string,
    exitStatus: number
}

export interface RunCompiledCodeResult extends RunCodeResult {
    compilationResult: RunCodeResult | null
}

export class StringResult {
    protected code: Code
    public text: string

    public constructor(code: Code, text: string) {
        this.code = code;
        this.text = text;
    }

    public assertEquals(value: string): TestCase {
        const valid = eqIgnoreTrailingWhitespace(this.text, value);
        const testCase = new TestCase(
            undefined,
            valid ? "Pass" : "Fail",
            {
                "Diff": {
                    expected: value,
                    output: this.text
                }
            }
        );
        this.code.testCases.push(testCase);
        return testCase;
    }

    public assert(cb: (k: string) => TestCase): TestCase {
        const vestCase = cb(this.text);
        this.code.testCases.push(vestCase);
        return vestCase
    }
}

export class RunResult extends StringResult {
    private stderr: string;

    public constructor(code: Code, result: RunCodeResult) {
        super(code, result.stdout);
        this.stderr = result.stderr;
    }

    public error() {
        return new StringResult(this.code, this.stderr);
    }
}

export class Code {
    public code: string;
    private onRunCallback: (code: string, input: string | undefined) => Promise<RunCompiledCodeResult>;
    public testCases: TestCase[];

    private runs: number = 0;

    constructor(code: string, onRunCallback: (code: string, input: string | undefined) => Promise<RunCompiledCodeResult>) {
        this.code = code;
        this.onRunCallback = onRunCallback;
        this.testCases = [];
    }

    async run(input?: string | undefined): Promise<RunResult> {
        return this.runCode(this.code, input);
    }

    async runCode(code: string, input?: string | undefined) {
        const result = await this.onRunCallback(code, input);
        let {stderr, stdout, compilationResult} = result;
        const compilationFailed = compilationResult && compilationResult.exitStatus !== 0;
        if (compilationFailed) {
            stdout = compilationResult.stdout;
            stderr = compilationResult.stderr;
        }
        const runDisplay: TestCase = new TestCase(
            `Run #${++this.runs}`,
            compilationFailed ? 'Warning' : 'Info',
            {
                "Run": {
                    "input": input,
                    "output": stdout,
                    "error": stderr,
                }
            }
        )
        console.log(JSON.stringify(runDisplay));
        return new RunResult(this, result);
    }

    registerTestCase(testCase: TestCase): TestCase {
        this.testCases.push(testCase);
        return testCase;
    }

    noFailures(): FinalVerdict {
        if (this.testCases.every((i) => i.pass !== 'Fail')) {
            return new FinalVerdict(true)
        } else {
            return new FinalVerdict(false)
        }
    }
}

export const eqIgnoreTrailingWhitespace = (a: string, b: string): boolean => {
    const [a_stripped, b_stripped] = [a, b].map(
        (text) => text.replace(/\s*(?=\n|$)/ug, '')
    )
    return a_stripped == b_stripped
}
