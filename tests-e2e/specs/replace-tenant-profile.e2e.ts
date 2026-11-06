import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

import {
  claimTenantProfileDirectory,
  type TenantProfileDirectoryClaim,
} from "../support/claim-tenant-profile-directory";
import { fillReceiptForm } from "../support/fill-receipt-form";
import { openReceiptScreen } from "../support/open-receipt-screen";
import { SAMPLE_RECEIPT } from "../support/sample-receipt";
import { SAMPLE_TENANT_PROFILE } from "../support/sample-tenant-profile";
import { TIMEOUTS_MS } from "../support/timeouts-ms";

describe("replacing a tenant profile", () => {
  let tenantProfileDirectory: TenantProfileDirectoryClaim | null = null;

  beforeEach(async () => {
    tenantProfileDirectory = await claimTenantProfileDirectory(SAMPLE_TENANT_PROFILE.file);
  });

  afterEach(async () => {
    await tenantProfileDirectory?.release();
    tenantProfileDirectory = null;
  });

  it("asks before overwriting the profile of the same tenant, then rewrites its file once confirmed", async () => {
    assert.ok(tenantProfileDirectory !== null);
    await openReceiptScreen();
    const saveButton = $("#save-tenant-profile-button");
    const savedBanner = $("#tenant-profile-saved");
    await fillReceiptForm(SAMPLE_RECEIPT.entries);
    await saveButton.click();
    await savedBanner.waitForDisplayed({ timeout: TIMEOUTS_MS.tenantProfile });

    await fillReceiptForm({ rentAmount: SAMPLE_TENANT_PROFILE.replacementRentAmount });
    await saveButton.click();

    await $("#tenant-profile-replace").waitForDisplayed({ timeout: TIMEOUTS_MS.tenantProfile });
    assert.equal(
      (await $("#tenant-profile-replace .banner-name").getText()).trim(),
      SAMPLE_TENANT_PROFILE.label,
    );
    assert.equal(
      await readFile(tenantProfileDirectory.filePath, "utf8"),
      SAMPLE_TENANT_PROFILE.fileContents,
      "the profile file should stay untouched until the replacement is confirmed",
    );

    await $("#replace-tenant-profile-button").click();

    await savedBanner.waitForDisplayed({ timeout: TIMEOUTS_MS.tenantProfile });
    assert.equal(
      await readFile(tenantProfileDirectory.filePath, "utf8"),
      SAMPLE_TENANT_PROFILE.replacedFileContents,
    );
  });
});
