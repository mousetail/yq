const { argv, stdin } = require('node:process');
const { writeFile } = require('node:fs/promises');
const { execFile } = require('node:child_process');
const { default: test } = require('node:test');
const { readFileSync } = require('node:fs');

const { code, lang, judge } = JSON.parse(readFileSync(0));

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

const run = (args, env, input) => {
    return new Promise((resolve, reject) => {
        const process = execFile(args[0], args.slice(1), {
            env: Object.fromEntries(env),
            stdio: "pipe"
        }, (error, stdout, stderr) => {
            const status = error ? error.code : 0;
            if (status === undefined) {
                reject(error);
            }

            resolve({
                stdout,
                stderr,
                exitStatus: status
            })
        });

        process.stdin.addListener('error', (err) => {
            console.warn(JSON.stringify(err))
        });

        try {
            process.stdin.write(input, (err) => {
                try {
                    process.stdin.end();
                } catch {
                    console.warn("Failed to close stdin");
                }
            });
        } catch {
            console.warn("Failed to write to stdin");
        }

    });
}

const compile_and_run_program = (() => {
    const compiled_programs = {};

    const replaceTokens = ar => ar.map((e) => {
        return e.replace(/\$\{LANG_LOCATION\}/ug, '/lang')
            .replace(/\$\{FILE_LOCATION\}/ug, '/tmp/code');
    })

    return async (lang, code, input) => {
        let [combined_stdout, combined_stderr] = ["", ""];
        if (!Object.prototype.hasOwnProperty(compiled_programs, code) && lang.compile_command.length > 0) {
            const { stdout, stderr, status } = await run(
                replaceTokens(lang.compile_command),
                lang.env,
                ""
            )
            compiled_programs[code] = true;
            combined_stdout += stdout;
            combined_stderr += stderr;
        }

        const { stdout, stderr, status } = await run(
            replaceTokens(lang.run_command),
            lang.env,
            input
        );

        return {
            stdout: combined_stdout + stdout,
            stderr: combined_stderr + stderr,
            status
        }
    }
})();

(async () => {
    const judge_function = eval(judge);

    const on_run_callback = async (program, input) => {
        writeFile('/tmp/code', program);

        return await compile_and_run_program(
            lang,
            {
                "LD_LIBRARY_PATH": "/lang/lib"
            },
            input ?? ''
        );
    };

    for await (const testCase of judge_function(code, on_run_callback)) {
        console.log(JSON.stringify(testCase));
    }
})();
