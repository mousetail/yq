type PassState = 'Pass' | 'Fail' | 'Warning' | 'Error';
type ResultDisplay = { type: 'Diff', expected: 'String', actual: 'String' } | { type: 'Text', text: string };

class TestCase {
    name: string | undefined;
    pass: PassState;
    resultDisplay: ResultDisplay;

    constructor(name: string | undefined, pass: PassState, resultDisplay: ResultDisplay) {
        this.name = name;
        this.pass = pass;
        this.resultDisplay = resultDisplay;
    }
}

class FinalVerdict {
    pass: boolean

    constructor(pass: boolean) {
        this.pass = pass;
    }
}

type RunCodeResult = {
    stdout: string,
    stderr: string,
    exitStatus: number
}

class Code {
    code: string;
    onRunCallback: (input: string) => RunCodeResult

    constructor(code: string, onRunCallback: (input: string) => RunCodeResult) {
        this.code = code;
        this.onRunCallback = onRunCallback;
    }

    run(input: string): RunCodeResult {
        return this.onRunCallback(input)
    }
}

class Run {

}

const eqIgnoreTrailingWhitespace = (a: string, b: string): boolean => {
    const [a_stripped, b_stripped] = [a, b].map(
        (text) => text.replace(/\s*(?=\n|$)/ug, '')
    )
    return a_stripped == b_stripped
}
