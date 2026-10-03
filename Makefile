# Point d'entrée unique des commandes de développement, partagé par le hook git et la CI.
.PHONY: setup format fmt-check lint quality build-debug test-unit build-release test-integration test-e2e ci

setup:
	npm ci
	git config core.hooksPath .githooks
	ln -sfn ../../rules/project .agents/rules/project

format:
	cargo fmt --all
	npx prettier --write .
	npx eslint --fix .

fmt-check:
	cargo fmt --all --check
	npx prettier --check .

lint:
	cargo clippy --workspace --all-targets -- -D warnings
	npx eslint .
	npx tsc --noEmit

quality: fmt-check lint

build-debug:
	cargo build --workspace

test-unit:
	cargo test --workspace --lib --bins
	npx vitest run --passWithNoTests

build-release:
	cargo build --workspace --release

# cargo échoue sur --test '*' tant qu'aucun dossier tests/ n'existe.
test-integration:
	@if find crates/*/tests src-tauri/tests -maxdepth 1 -name '*.rs' 2>/dev/null | grep -q .; then \
		cargo test --release --workspace --test '*'; \
	else \
		echo "test-integration: aucun test d'intégration pour l'instant"; \
	fi

# Branché en F8 (tauri-driver + WebdriverIO).
test-e2e:
	@echo "test-e2e: aucun test e2e pour l'instant"

ci: quality build-debug test-unit build-release test-integration test-e2e
