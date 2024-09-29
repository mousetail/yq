const { argv } = require('node:process');
const { writeFile } = require('node:fs/promises');
const { execFile } = require('node:child_process');

const [, , execute, code, judge] = argv;

class TestCase {
    __construct(pass, expectedOutput, actualOutput) {
        this.pass = pass
        this.expectedOutput = expectedOutput;
        this.actualOutput = actualOutput;
    }
}

class FinalVerdit {
    __construct(pass) {
        this.pass = pass;
    }
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