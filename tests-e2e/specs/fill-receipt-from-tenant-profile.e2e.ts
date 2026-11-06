import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

import {
  claimTenantProfileDirectory,
  type TenantProfileDirectoryClaim,
} from "../support/claim-tenant-profile-directory";
import { fillReceiptForm } from "../support/fill-receipt-form";
import { MAILPIT_CLIENT } from "../support/mailpit-client";
import { openReceiptScreen } from "../support/open-receipt-screen";
import { readReceiptForm } from "../support/read-receipt-form";
import { SAMPLE_RECEIPT } from "../support/sample-receipt";
import { SAMPLE_TENANT_PROFILE } from "../support/sample-tenant-profile";
import { TIMEOUTS_MS } from "../support/timeouts-ms";
import { waitForSingleMessage } from "../support/wait-for-single-message";

describe("filling a receipt from a tenant profile", () => {
  let tenantProfileDirectory: TenantProfileDirectoryClaim | null = null;

  beforeEach(async () => {
    await MAILPIT_CLIENT.deleteAllMessages();
    tenantProfileDirectory = await claimTenantProfileDirectory(SAMPLE_TENANT_PROFILE.file);
  });

  afterEach(async () => {
    await tenantProfileDirectory?.release();
    tenantProfileDirectory = null;
  });

  it("saves the entered tenant as a profile file, prefills the reloaded form from it, then mails the receipt to its tenant", async () => {
    assert.ok(tenantProfileDirectory !== null);
    const { entries } = SAMPLE_RECEIPT;
    await openReceiptScreen();
    await fillReceiptForm(entries);

    await $("#save-tenant-profile-button").click();

    await $("#tenant-profile-saved").waitForDisplayed({ timeout: TIMEOUTS_MS.tenantProfile });
    assert.equal(
      await readFile(tenantProfileDirectory.filePath, "utf8"),
      SAMPLE_TENANT_PROFILE.fileContents,
    );

    await browser.refresh();
    await openReceiptScreen();
    await $(`#tenant-profile option[value="${SAMPLE_TENANT_PROFILE.id}"]`).waitForExist({
      timeout: TIMEOUTS_MS.tenantProfile,
      timeoutMsg: "[E2E_TENANT_PROFILE_NOT_LISTED] the saved tenant profile is not offered",
    });
    const reloadedForm = await readReceiptForm();
    assert.ok(
      Object.values(reloadedForm).every((value) => value === ""),
      `the reloaded form should start empty: ${JSON.stringify(reloadedForm)}`,
    );
    const { periodStart, periodEnd, paymentDate } = entries;
    await fillReceiptForm({ periodStart, periodEnd, paymentDate });

    await $("#tenant-profile").selectByAttribute("value", SAMPLE_TENANT_PROFILE.id);

    await browser.waitUntil(async () => (await $("#tenant-name").getValue()) !== "", {
      timeout: TIMEOUTS_MS.tenantProfile,
      timeoutMsg: "[E2E_TENANT_PROFILE_NOT_LOADED] choosing the tenant profile filled no field",
    });
    assert.deepEqual(await readReceiptForm(), {
      ...SAMPLE_TENANT_PROFILE.prefilledEntries,
      periodStart,
      periodEnd,
      paymentDate,
    });
    assert.equal(await $("#template").getValue(), SAMPLE_TENANT_PROFILE.templateId);

    await $("#preview-button").click();
    await $("#preview-page").waitForDisplayed({ timeout: TIMEOUTS_MS.preview });
    const sendButton = $("#send-button");
    await sendButton.waitForEnabled({ timeout: TIMEOUTS_MS.preview });
    await sendButton.click();
    await $("#send-success").waitForDisplayed({ timeout: TIMEOUTS_MS.send });

    const message = await waitForSingleMessage();
    assert.deepEqual(message.toAddresses, [SAMPLE_TENANT_PROFILE.prefilledEntries.tenantEmail]);
  });
});
