.PHONY: ts-build-runner
ts-build-runner:
	npx tsc scripts/runner-lib.ts --target es2022 --moduleResolution bundler --declaration --outDir scripts/build

.PHONY: restart-runner
restart-runner:
	cargo build --package lang-runner
	docker container kill --signal USR1 yet-to-be-named-golfing-site-yq-runner-1
