import { type ChildProcess, spawn } from "node:child_process";
import { fileURLToPath } from "node:url";

import { appEnvironment } from "./support/app-environment";
import { TIMEOUTS_MS } from "./support/timeouts-ms";

// Binaire produit par `tauri build --no-bundle` (cible `make test-e2e`), front embarqué.
const APPLICATION_PATH = fileURLToPath(new URL("../target/release/quittance-app", import.meta.url));
const TAURI_DRIVER_PORT = 4444;

let tauriDriver: ChildProcess | null = null;
let isTauriDriverStopping = false;

function stopTauriDriver(): void {
  isTauriDriverStopping = true;
  tauriDriver?.kill();
  tauriDriver = null;
}

/**
 * Lance tauri-driver, qui démarre l'app pour chaque session WebDriver. L'app hérite de
 * son environnement, d'où la configuration de test injectée ici.
 */
function startTauriDriver(): void {
  isTauriDriverStopping = false;
  const driver = spawn("tauri-driver", ["--port", String(TAURI_DRIVER_PORT)], {
    env: { ...process.env, ...appEnvironment() },
    stdio: ["ignore", "inherit", "inherit"],
  });
  driver.on("error", (err: Error) => {
    console.error(
      "[E2E_TAURI_DRIVER_FAILED] tauri-driver could not start (cargo install --locked tauri-driver)",
      err,
    );
    process.exit(1);
  });
  driver.on("exit", (code: number | null) => {
    if (!isTauriDriverStopping) {
      console.error("[E2E_TAURI_DRIVER_EXITED] tauri-driver stopped during the session", { code });
      process.exit(1);
    }
  });
  tauriDriver = driver;
}

process.on("exit", stopTauriDriver);

/** Configuration WebdriverIO : un scénario par fichier, une instance de l'app par scénario. */
export const config: WebdriverIO.Config = {
  runner: "local",
  hostname: "127.0.0.1",
  port: TAURI_DRIVER_PORT,
  specs: ["./specs/**/*.e2e.ts"],
  maxInstances: 1,
  capabilities: [
    {
      "tauri:options": { application: APPLICATION_PATH },
    },
  ],
  // L'affichage virtuel est fourni par xvfb-run (Makefile), pas par WebdriverIO.
  autoXvfb: false,
  logLevel: "warn",
  reporters: ["spec"],
  framework: "mocha",
  mochaOpts: {
    ui: "bdd",
    timeout: TIMEOUTS_MS.scenario,
  },
  beforeSession: startTauriDriver,
  afterSession: stopTauriDriver,
};
