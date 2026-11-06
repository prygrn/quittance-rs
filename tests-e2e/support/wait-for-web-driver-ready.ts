/** Horloge de l'attente, injectée pour tester sans délai réel. */
export interface WebDriverReadyClock {
  readonly now: () => number;
  readonly sleep: (durationMs: number) => Promise<void>;
  /** Signal annulé après durationMs, comme `AbortSignal.timeout`. */
  readonly timeoutSignal: (durationMs: number) => AbortSignal;
}

/** Requête `GET /status`, annulée par son signal si le serveur ne répond pas à temps. */
export interface WebDriverStatusRequest {
  readonly url: string;
  readonly signal: AbortSignal;
}

/** Paramètres de waitForWebDriverReady. */
export interface WaitForWebDriverReadyOptions {
  /** Endpoint W3C `GET /status` du serveur WebDriver. */
  readonly statusUrl: string;
  readonly timeoutMs: number;
  readonly pollIntervalMs: number;
  readonly fetch: (request: WebDriverStatusRequest) => Promise<Response>;
  readonly clock: WebDriverReadyClock;
}

async function describeStatusFailure(
  options: WaitForWebDriverReadyOptions,
): Promise<string | null> {
  try {
    const response = await options.fetch({
      url: options.statusUrl,
      signal: new AbortController().signal,
    });
    return response.ok ? null : `HTTP ${response.status}`;
  } catch (err: unknown) {
    return err instanceof Error ? `${err.message} (${String(err.cause)})` : String(err);
  }
}

async function pollStatus(poll: {
  readonly options: WaitForWebDriverReadyOptions;
  readonly deadlineMs: number;
}): Promise<void> {
  const { options, deadlineMs } = poll;
  const failure = await describeStatusFailure(options);
  if (failure === null) {
    return;
  }
  const remainingMs = deadlineMs - options.clock.now();
  if (remainingMs <= 0) {
    throw new Error(
      `[E2E_WEBDRIVER_NOT_READY] ${options.statusUrl} did not answer within ${options.timeoutMs} ms, last failure: ${failure}`,
    );
  }
  await options.clock.sleep(Math.min(options.pollIntervalMs, remainingMs));
  return pollStatus(poll);
}

/**
 * Attend qu'un serveur WebDriver réponde à `GET /status`, en réessayant jusqu'au délai
 * maximum.
 */
export async function waitForWebDriverReady(options: WaitForWebDriverReadyOptions): Promise<void> {
  return pollStatus({ options, deadlineMs: options.clock.now() + options.timeoutMs });
}
