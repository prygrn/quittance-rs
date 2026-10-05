# Point d'entrée unique des commandes de développement, partagé par le hook git et la CI.
.PHONY: setup system-deps format fmt-check lint quality build-debug test-unit test build-release test-integration test-e2e ci msrv-version msrv-check

setup:
	npm ci
	git config core.hooksPath .githooks

# Bibliothèques système requises pour compiler l'app Tauri (Debian, Ubuntu).
system-deps:
	sudo apt-get update
	sudo apt-get install -y libwebkit2gtk-4.1-dev libxdo-dev libayatana-appindicator3-dev librsvg2-dev

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
	.agents/scripts/compile-agents --check

build-debug:
	cargo build --workspace

test-unit:
	cargo test --workspace --lib --bins
	cargo test --workspace --doc
	npx vitest run --passWithNoTests

test: test-unit

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

# Affiche le rust-version déclaré dans Cargo.toml ; échoue s'il est introuvable.
msrv-version:
	@v=$$(sed -n 's/^rust-version = "\(.*\)"/\1/p' Cargo.toml); \
	if [ -z "$$v" ]; then echo "msrv-version: rust-version introuvable dans Cargo.toml" >&2; exit 1; fi; \
	echo "$$v"

# Vérifie la compilation avec le rust-version déclaré dans Cargo.toml (requiert rustup).
msrv-check:
	@v=$$($(MAKE) -s msrv-version) || exit 1; \
	cargo +$$v check --workspace --all-targets --locked

ci: quality build-debug test-unit build-release test-integration test-e2e
