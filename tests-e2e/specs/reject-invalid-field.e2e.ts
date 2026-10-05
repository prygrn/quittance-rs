import assert from "node:assert/strict";

import { fillReceiptForm } from "../support/fill-receipt-form";
import { MAILPIT_CLIENT } from "../support/mailpit-client";
import { openReceiptScreen } from "../support/open-receipt-screen";
import { SAMPLE_RECEIPT } from "../support/sample-receipt";

const ERROR_TIMEOUT_MS = 5_000;

describe("rejecting an invalid field", () => {
  beforeEach(async () => {
    await MAILPIT_CLIENT.deleteAllMessages();
  });

  it("flags the invalid tenant email and sends no email", async () => {
    await openReceiptScreen();
    await fillReceiptForm({ ...SAMPLE_RECEIPT.entries, tenantEmail: "jeanne.martin@" });

    await $("#preview-button").click();

    const fieldError = $("#tenant-email-error");
    await fieldError.waitForDisplayed({ timeout: ERROR_TIMEOUT_MS });
    assert.notEqual(
      (await fieldError.getText()).trim(),
      "",
      "the field error should explain the problem",
    );
    assert.equal(await $("#tenant-email").getAttribute("aria-invalid"), "true");
    assert.equal(
      await $("#error-summary").isDisplayed(),
      true,
      "the error summary should be shown",
    );
    assert.equal(await $("#preview-page").isDisplayed(), false, "no preview should be rendered");
    assert.equal(await $("#send-button").isEnabled(), false, "sending should stay unavailable");
    assert.deepEqual(await MAILPIT_CLIENT.listMessageIds(), [], "no email should be sent");
  });
});
