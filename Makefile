# Point d'entrée unique des commandes de développement, partagé par le hook git et la CI.
.PHONY: setup system-deps e2e-deps format fmt-check lint quality build-debug test-unit test build-release test-integration build-e2e-app test-e2e ci msrv-version msrv-check

setup:
	npm ci
	git config core.hooksPath .githooks

# Bibliothèques système requises pour compiler l'app Tauri (Debian, Ubuntu).
system-deps:
	sudo apt-get update
	sudo apt-get install -y libwebkit2gtk-4.1-dev libxdo-dev libayatana-appindicator3-dev librsvg2-dev

TAURI_DRIVER_VERSION := 2.1.0

# Outils des tests e2e (Debian, Ubuntu) : WebDriver de WebKitGTK, affichage virtuel, tauri-driver.
e2e-deps:
	sudo apt-get update
	sudo apt-get install -y webkit2gtk-driver xvfb
	cargo install --locked tauri-driver --version $(TAURI_DRIVER_VERSION)

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
	npx tsc --noEmit -p tests-e2e

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

# Binaire release lancé par les tests e2e : contrairement à `cargo build`, la CLI Tauri
# embarque le front (vite build) au lieu de pointer vers le serveur de développement.
build-e2e-app:
	npx tauri build --no-bundle

# Requiert `make e2e-deps`, CHROME_PATH et un Mailpit local (SMTP 1025, API 8025), vidé
# avant chaque scénario.
test-e2e: build-e2e-app
	xvfb-run --auto-servernum npx wdio run tests-e2e/wdio.conf.ts

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
