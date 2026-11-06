import { describe, expect, it } from "vitest";

import { type WebDriverReadyClock, waitForWebDriverReady } from "./wait-for-web-driver-ready";

const STATUS_URL = "http://127.0.0.1:4444/status";
const TIMEOUT_MS = 1_000;
const POLL_INTERVAL_MS = 100;

// Réponse de WebKitWebDriver relayée par tauri-driver 2.1.0 une fois le driver natif lancé.
const READY_BODY = JSON.stringify({ value: { ready: true, message: "No sessions" } });

type StatusAttempt = () => Promise<Response>;

/** Horloge factice : chaque attente avance le temps sans délai réel. */
function fakeClock(): WebDriverReadyClock & { readonly sleeps: number[] } {
  let nowMs = 0;
  const sleeps: number[] = [];
  return {
    sleeps,
    now: (): number => nowMs,
    sleep: async (durationMs: number): Promise<void> => {
      sleeps.push(durationMs);
      nowMs += durationMs;
    },
  };
}

/** fetch factice : rejoue les tentatives dans l'ordre, puis répète la dernière. */
function fakeFetch(attempts: readonly StatusAttempt[]): {
  readonly fetch: (url: string) => Promise<Response>;
  readonly requestedUrls: string[];
} {
  const requestedUrls: string[] = [];
  return {
    requestedUrls,
    fetch: (url: string): Promise<Response> => {
      requestedUrls.push(url);
      const attempt = attempts[Math.min(requestedUrls.length, attempts.length) - 1];
      if (attempt === undefined) {
        throw new Error("fakeFetch needs at least one attempt");
      }
      return attempt();
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
});
