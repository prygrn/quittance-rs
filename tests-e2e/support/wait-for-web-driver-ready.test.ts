import { once } from "node:events";

import { describe, expect, it } from "vitest";

import {
  type WebDriverReadyClock,
  type WebDriverStatusRequest,
  waitForWebDriverReady,
} from "./wait-for-web-driver-ready";

const STATUS_URL: string = "http://127.0.0.1:4444/status";
const TIMEOUT_MS: number = 1_000;
const POLL_INTERVAL_MS: number = 100;

// Réponse de WebKitWebDriver relayée par tauri-driver 2.1.0 une fois le driver natif lancé.
const READY_BODY: string = JSON.stringify({ value: { ready: true, message: "No sessions" } });

type StatusAttempt = (request: WebDriverStatusRequest) => Promise<Response>;

type FakeClock = WebDriverReadyClock & {
  readonly sleeps: number[];
  readonly signalDurations: number[];
  /** Avance le temps jusqu'à l'échéance du dernier signal créé, puis l'annule. */
  readonly expireLastSignal: () => void;
};

/** Horloge factice : chaque attente avance le temps sans délai réel. */
function fakeClock(): FakeClock {
  let nowMs = 0;
  const sleeps: number[] = [];
  const signalDurations: number[] = [];
  let lastSignal: { readonly deadlineMs: number; readonly controller: AbortController } | null =
    null;
  return {
    sleeps,
    signalDurations,
    now: (): number => nowMs,
    sleep: async (durationMs: number): Promise<void> => {
      sleeps.push(durationMs);
      nowMs += durationMs;
    },
    timeoutSignal: (durationMs: number): AbortSignal => {
      signalDurations.push(durationMs);
      lastSignal = { deadlineMs: nowMs + durationMs, controller: new AbortController() };
      return lastSignal.controller.signal;
    },
    expireLastSignal: (): void => {
      if (lastSignal === null) {
        throw new Error("expireLastSignal: no signal was created by timeoutSignal");
      }
      nowMs = lastSignal.deadlineMs;
      lastSignal.controller.abort(new DOMException("The operation timed out.", "TimeoutError"));
    },
  };
}

/** fetch factice : rejoue les tentatives dans l'ordre, puis répète la dernière. */
function fakeFetch(attempts: readonly StatusAttempt[]): {
  readonly fetch: StatusAttempt;
  readonly requestedUrls: string[];
} {
  const requestedUrls: string[] = [];
  return {
    requestedUrls,
    fetch: (request: WebDriverStatusRequest): Promise<Response> => {
      requestedUrls.push(request.url);
      const attempt = attempts[Math.min(requestedUrls.length, attempts.length) - 1];
      if (attempt === undefined) {
        throw new Error("fakeFetch needs at least one attempt");
      }
      return attempt(request);
    },
  };
}

const ready: StatusAttempt = async () => new Response(READY_BODY, { status: 200 });
// tauri-driver pas encore à l'écoute.
const connectionRefused: StatusAttempt = async () => {
  throw new TypeError("fetch failed", { cause: new Error("connect ECONNREFUSED 127.0.0.1:4444") });
};
// tauri-driver à l'écoute, WebKitWebDriver pas encore : la connexion est fermée sans réponse.
const socketClosed: StatusAttempt = async () => {
  throw new TypeError("fetch failed", { cause: new Error("other side closed") });
};
const serverError: StatusAttempt = async () => new Response("", { status: 500 });
const failedWithoutCause: StatusAttempt = async () => {
  throw new TypeError("fetch failed");
};

/** Connexion acceptée qui ne répond jamais : seule l'annulation de son signal la termine. */
function neverAnswers(clock: FakeClock): StatusAttempt {
  return async (request: WebDriverStatusRequest) => {
    const aborted = once(request.signal, "abort");
    clock.expireLastSignal();
    await aborted;
    throw request.signal.reason;
  };
}

describe("waitForWebDriverReady", () => {
  it("resolves at the first attempt when the driver already answers its status", async () => {
    const clock = fakeClock();
    const driver = fakeFetch([ready]);

    await waitForWebDriverReady({
      statusUrl: STATUS_URL,
      timeoutMs: TIMEOUT_MS,
      pollIntervalMs: POLL_INTERVAL_MS,
      fetch: driver.fetch,
      clock,
    });

    expect(driver.requestedUrls).toEqual([STATUS_URL]);
    expect(clock.sleeps).toEqual([]);
  });

  it("polls the status until the driver answers after several failed attempts", async () => {
    const clock = fakeClock();
    const driver = fakeFetch([connectionRefused, socketClosed, serverError, ready]);

    await waitForWebDriverReady({
      statusUrl: STATUS_URL,
      timeoutMs: TIMEOUT_MS,
      pollIntervalMs: POLL_INTERVAL_MS,
      fetch: driver.fetch,
      clock,
    });

    expect(driver.requestedUrls).toEqual([STATUS_URL, STATUS_URL, STATUS_URL, STATUS_URL]);
    expect(clock.sleeps).toEqual([POLL_INTERVAL_MS, POLL_INTERVAL_MS, POLL_INTERVAL_MS]);
  });

  it("rejects with a coded error once the timeout elapses without an answer", async () => {
    const clock = fakeClock();
    const driver = fakeFetch([connectionRefused]);

    await expect(
      waitForWebDriverReady({
        statusUrl: STATUS_URL,
        timeoutMs: TIMEOUT_MS,
        pollIntervalMs: POLL_INTERVAL_MS,
        fetch: driver.fetch,
        clock,
      }),
    ).rejects.toThrow("[E2E_WEBDRIVER_NOT_READY]");
    expect(clock.now()).toBe(TIMEOUT_MS);
  });

  it("shortens the last wait so that polling stops at the timeout", async () => {
    const clock = fakeClock();
    const driver = fakeFetch([connectionRefused]);

    await expect(
      waitForWebDriverReady({
        statusUrl: STATUS_URL,
        timeoutMs: 250,
        pollIntervalMs: 100,
        fetch: driver.fetch,
        clock,
      }),
    ).rejects.toThrow("[E2E_WEBDRIVER_NOT_READY]");
    expect(clock.sleeps).toEqual([100, 100, 50]);
  });

  it("aborts a status request that never answers once the timeout elapses", async () => {
    const clock = fakeClock();
    const driver = fakeFetch([neverAnswers(clock)]);

    await expect(
      waitForWebDriverReady({
        statusUrl: STATUS_URL,
        timeoutMs: TIMEOUT_MS,
        pollIntervalMs: POLL_INTERVAL_MS,
        fetch: driver.fetch,
        clock,
      }),
    ).rejects.toThrow("[E2E_WEBDRIVER_NOT_READY]");
    expect(clock.signalDurations).toEqual([TIMEOUT_MS]);
    expect(clock.now()).toBe(TIMEOUT_MS);
  });

  it("reports the cause of the last failure when it has one", async () => {
    const clock = fakeClock();
    const driver = fakeFetch([connectionRefused]);

    await expect(
      waitForWebDriverReady({
        statusUrl: STATUS_URL,
        timeoutMs: TIMEOUT_MS,
        pollIntervalMs: POLL_INTERVAL_MS,
        fetch: driver.fetch,
        clock,
      }),
    ).rejects.toThrow("ECONNREFUSED");
  });

  it("reports a failure without cause without mentioning a missing cause", async () => {
    const clock = fakeClock();
    const driver = fakeFetch([failedWithoutCause]);

    await expect(
      waitForWebDriverReady({
        statusUrl: STATUS_URL,
        timeoutMs: TIMEOUT_MS,
        pollIntervalMs: POLL_INTERVAL_MS,
        fetch: driver.fetch,
        clock,
      }),
    ).rejects.not.toThrow("undefined");
  });
});
