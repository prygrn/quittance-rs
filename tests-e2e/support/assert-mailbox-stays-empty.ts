import assert from "node:assert/strict";

import { MAILPIT_CLIENT } from "./mailpit-client";
import { TIMEOUTS_MS } from "./timeouts-ms";

const POLL_INTERVAL_MS = 250;

/**
 * Sonde Mailpit pendant `TIMEOUTS_MS.silentMailbox` et échoue dès qu'un message arrive :
 * un envoi parti en tâche de fond n'est pas encore visible au moment du clic.
 */
export async function assertMailboxStaysEmpty(): Promise<void> {
  const deadline = Date.now() + TIMEOUTS_MS.silentMailbox;
  const poll = async (): Promise<void> => {
    assert.deepEqual(await MAILPIT_CLIENT.listMessageIds(), [], "no email should be sent");
    if (Date.now() < deadline) {
      await browser.pause(POLL_INTERVAL_MS);
      await poll();
    }
  };
  await poll();
}
