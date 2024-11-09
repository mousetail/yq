import { argv, stdin } from 'node:process';
import { writeFile } from 'node:fs/promises';
import { execFile } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { Code, FinalVerdict, RunCodeResult, TestCase } from './runner-lib.ts';

type Lang = {
    name: string,
    compileCommand: string[],
    runCommand: string[],
    env: [string, string][],
    installEnv: [string, string][],
    plugin: string,
    latestVersion: string
};

type Input = {
    code: string,
    lang: Lang,
    judge: string
}

const { code, lang, judge }: Input = await new Response(Deno.stdin.readable).json();

const run = async (args: string[], env: [string, string][], input: string): Promise<RunCodeResult> => {
    const command = new Deno.Command(
        args[0],
        {
            args: args.slice(1),
            stdin: 'piped',
            stdout: 'piped',
            stderr: 'piped',
            env: Object.fromEntries(env)
        }
    )

    const process = command.spawn();
    const writer = process.stdin.getWriter();
    await writer.write(new TextEncoder().encode(input));
    await writer.close();
    const { code, stdout, stderr } = await process.output();
    const textDecoder = new TextDecoder();
    return {
        exitStatus: code,
        stdout: textDecoder.decode(stdout),
        stderr: textDecoder.decode(stderr)
    }
}

const compile_and_run_program = (() => {
    const compiled_programs = {};

    const replaceTokens = ar => ar.map((e) => {
        return e.replace(/\$\{LANG_LOCATION\}/ug, '/lang')
            .replace(/\$\{FILE_LOCATION\}/ug, '/tmp/code');
    })

    return async (lang: Lang, code: string, input: string) => {
        let [combined_stdout, combined_stderr] = ["", ""];
        if (!Object.prototype.hasOwnProperty.call(compiled_programs, code) && lang.compileCommand.length > 0) {
            const { stdout, stderr, exitStatus } = await run(
                replaceTokens(lang.compileCommand),
                lang.env,
                ""
            )
            compiled_programs[code] = true;
            combined_stdout += stdout;
            combined_stderr += stderr;
        }

        const { stdout, stderr, exitStatus } = await run(
            replaceTokens(lang.runCommand),
            lang.env,
            input
        );

        return {
            stdout: combined_stdout + stdout,
            stderr: combined_stderr + stderr,
            exitStatus
        }
    }
})();

(async () => {
    const judge_function = (
        await import('data:text/typescript,' + encodeURIComponent(
            readFileSync('/scripts/runner-lib.ts') +
            '\nexport default ' +
            judge
        ))
    ).default as ((code: Code) => AsyncGenerator<TestCase, FinalVerdict, undefined>);

    const on_run_callback = async (program: string, input?: string | undefined): Promise<RunCodeResult> => {
        writeFile('/tmp/code', program);
        return await compile_and_run_program(
            lang,
            program,
            input ?? ''
        );
    };

    const generator = judge_function(new Code(code, on_run_callback));

    let value: IteratorResult<TestCase, FinalVerdict>;
    while (!(value = await generator.next()).done) {
        console.log(JSON.stringify(value.value));
    }
    console.log(JSON.stringify(value.value));
})();
