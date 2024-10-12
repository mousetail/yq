const { argv } = require('node:process');
const { writeFile } = require('node:fs/promises');
const { execFile } = require('node:child_process');
const { default: test } = require('node:test');

const [, , execute, code, judge] = argv;

class TestCase {
    constructor(name, pass, result_display, error) {
        this.name = name;
        this.pass = pass;
        this.result_display = result_display;
        this.error = error;
    }
}

class FinalVerdict {
    constructor(pass) {
        this.pass = pass;
    }
}

const eqIgnoreTrailingWhitespace = (a, b) => {
    const [a_stripped, b_stripped] = [a, b].map(
        (text) => text.replace(/\s*(?=\n|$)/ug, '')
    )
    return a_stripped == b_stripped
}

const run_program = (file, args, env, input) => {
    return new Promise((resolve, reject) => {
        const process = execFile(file, args, {
            env: env,
        }, (error, stdout, stderr) => {
            resolve({
                stdout,
                stderr,
                exitStatus: error?.status ?? 0
            })
        });

        process.stdin.write(input, () => {
            process.stdin.end();
        });

    });
}

(async () => {
    const judge_function = eval(judge);

    const on_run_callback = async (program, input) => {
        writeFile('/tmp/code', program);

        return await run_program(
            execute,
            ['/tmp/code'],
            {
                "LD_LIBRARY_PATH": "/lang/lib"
            }, input ?? ''
        );
    };

    for await (const testCase of judge_function(code, on_run_callback)) {
        console.log(JSON.stringify(testCase));
    }
})();
