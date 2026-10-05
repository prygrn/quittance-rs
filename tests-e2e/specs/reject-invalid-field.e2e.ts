import assert from "node:assert/strict";

import { assertMailboxStaysEmpty } from "../support/assert-mailbox-stays-empty";
import { fillReceiptForm } from "../support/fill-receipt-form";
import { MAILPIT_CLIENT } from "../support/mailpit-client";
import { openReceiptScreen } from "../support/open-receipt-screen";
import { SAMPLE_RECEIPT } from "../support/sample-receipt";
import { TIMEOUTS_MS } from "../support/timeouts-ms";

describe("rejecting an invalid field", () => {
  beforeEach(async () => {
    await MAILPIT_CLIENT.deleteAllMessages();
  });

  it("flags the invalid tenant email and sends no email", async () => {
    await openReceiptScreen();
    await fillReceiptForm({ ...SAMPLE_RECEIPT.entries, tenantEmail: "jeanne.martin@" });

    await $("#preview-button").click();

    await $("#tenant-email-error").waitForDisplayed({ timeout: TIMEOUTS_MS.fieldError });
    assert.notEqual(
      (await $("#tenant-email-error .field-error-text").getText()).trim(),
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
    await assertMailboxStaysEmpty();
  });
});
