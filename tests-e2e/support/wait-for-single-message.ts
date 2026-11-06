import assert from "node:assert/strict";

import { MAILPIT_CLIENT, type MailpitMessage } from "./mailpit-client";
import { TIMEOUTS_MS } from "./timeouts-ms";

/** Attend le message d'un envoi confirmé, vérifie qu'il est le seul reçu, puis le relit. */
export async function waitForSingleMessage(): Promise<MailpitMessage> {
  await browser.waitUntil(async () => (await MAILPIT_CLIENT.listMessageIds()).length > 0, {
    timeout: TIMEOUTS_MS.delivery,
    timeoutMsg: "[E2E_MAIL_NOT_RECEIVED] Mailpit received no message after the send",
  });
  const messageIds = await MAILPIT_CLIENT.listMessageIds();
  assert.equal(messageIds.length, 1, "exactly one email should be sent");
  const [messageId] = messageIds;
  assert.ok(messageId !== undefined);
  return MAILPIT_CLIENT.getMessage(messageId);
}
