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

function describeError(err: unknown): string {
  if (!(err instanceof Error)) {
    return String(err);
  }
  return err.cause === undefined ? err.message : `${err.message} (${String(err.cause)})`;
}

async function describeStatusFailure(attempt: {
  readonly options: WaitForWebDriverReadyOptions;
  readonly signal: AbortSignal;
}): Promise<string | null> {
  try {
    const response = await attempt.options.fetch({
      url: attempt.options.statusUrl,
      signal: attempt.signal,
    });
    return response.ok ? null : `HTTP ${response.status}`;
  } catch (err: unknown) {
    return describeError(err);
  }
}

async function pollStatus(poll: {
  readonly options: WaitForWebDriverReadyOptions;
  readonly deadlineMs: number;
  readonly lastFailure: string;
}): Promise<void> {
  const { options, deadlineMs } = poll;
  const remainingMs = deadlineMs - options.clock.now();
  if (remainingMs <= 0) {
    throw new Error(
      `[E2E_WEBDRIVER_NOT_READY] ${options.statusUrl} did not answer within ${options.timeoutMs} ms, last failure: ${poll.lastFailure}`,
    );
  }
  // Une connexion acceptée sans réponse ne doit pas prolonger l'attente au-delà du délai.
  const failure = await describeStatusFailure({
    options,
    signal: options.clock.timeoutSignal(remainingMs),
  });
  if (failure === null) {
    return;
  }
  const waitMs = Math.min(options.pollIntervalMs, deadlineMs - options.clock.now());
  await options.clock.sleep(Math.max(0, waitMs));
  return pollStatus({ options, deadlineMs, lastFailure: failure });
}

/**
 * Attend qu'un serveur WebDriver réponde à `GET /status`, en réessayant jusqu'au délai
 * maximum.
 */
export async function waitForWebDriverReady(options: WaitForWebDriverReadyOptions): Promise<void> {
  return pollStatus({
    options,
    deadlineMs: options.clock.now() + options.timeoutMs,
    lastFailure: "none, the timeout leaves no time for an attempt",
  });
}
