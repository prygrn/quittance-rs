/** Horloge de l'attente, injectée pour tester sans délai réel. */
export interface WebDriverReadyClock {
  readonly now: () => number;
  readonly sleep: (durationMs: number) => Promise<void>;
}

/** Paramètres de waitForWebDriverReady. */
export interface WaitForWebDriverReadyOptions {
  /** Endpoint W3C `GET /status` du serveur WebDriver. */
  readonly statusUrl: string;
  readonly timeoutMs: number;
  readonly pollIntervalMs: number;
  readonly fetch: (url: string) => Promise<Response>;
  readonly clock: WebDriverReadyClock;
}

/**
 * Attend qu'un serveur WebDriver réponde à `GET /status`, en réessayant jusqu'au délai
 * maximum.
 */
export async function waitForWebDriverReady(options: WaitForWebDriverReadyOptions): Promise<void> {
  throw new Error(`[E2E_NOT_IMPLEMENTED] waitForWebDriverReady(${options.statusUrl})`);
}
