.PHONY: first-time-setup
first-time-setup:
	touch .env.local
	mkdir -p target/debug
	npm install
	make ts-build-runner
	docker compose up --detach
	cargo install sqlx-cli --no-default-features --features rustls,postgres
	sqlx migrate run
	cargo build
	docker compose down

.PHONY: run-main-server
run-main-server:
	docker compose up --detach
	cargo run --bin main-server

.PHONY: clean
clean:
	cargo clean
	docker compose down
	rm -rf ./target ./scripts/build ./static/target ./node_modules

.PHONY: ts-build-runner
ts-build-runner:
	npx tsc scripts/runner-lib.ts --target es2022 --moduleResolution bundler --declaration --outDir scripts/build

.PHONY: restart-runner
restart-runner:
	cargo build --package lang-runner
	docker container kill --signal USR1 yet-to-be-named-golfing-site-yq-runner-1
