import assert from "node:assert/strict";

import { fillReceiptForm } from "../support/fill-receipt-form";
import { MAILPIT_CLIENT } from "../support/mailpit-client";
import { openReceiptScreen } from "../support/open-receipt-screen";
import { SAMPLE_RECEIPT } from "../support/sample-receipt";
import { TEST_LANDLORD } from "../support/test-landlord";

const PREVIEW_TIMEOUT_MS = 30_000;
// Chromium imprime le PDF avant l'envoi SMTP.
const SEND_TIMEOUT_MS = 60_000;
const DELIVERY_TIMEOUT_MS = 10_000;
const PDF_SIGNATURE = "%PDF-";

describe("sending a receipt", () => {
  beforeEach(async () => {
    await MAILPIT_CLIENT.deleteAllMessages();
  });

  it("previews the entered receipt, then mails it as a PDF to the tenant with the landlord in Bcc", async () => {
    const { entries } = SAMPLE_RECEIPT;
    await openReceiptScreen();
    await fillReceiptForm(entries);

    await $("#preview-button").click();

    const previewFrame = $("#preview-frame");
    await $("#preview-page").waitForDisplayed({ timeout: PREVIEW_TIMEOUT_MS });
    const previewHtml = await previewFrame.getAttribute("srcdoc");
    assert.ok(previewHtml !== null, "the preview frame should hold the receipt HTML");
    [
      entries.tenantName,
      entries.tenantAddress,
      entries.tenantEmail,
      entries.propertyAddress,
      entries.periodStart,
      entries.periodEnd,
      entries.paymentDate,
      SAMPLE_RECEIPT.expectedRent,
      SAMPLE_RECEIPT.expectedCharges,
      SAMPLE_RECEIPT.expectedTotal,
      TEST_LANDLORD.name,
    ].forEach((expected) => {
      assert.ok(previewHtml.includes(expected), `the preview should show ${expected}`);
    });

    const sendButton = $("#send-button");
    await sendButton.waitForEnabled({ timeout: PREVIEW_TIMEOUT_MS });
    await sendButton.click();
    await $("#send-success").waitForDisplayed({ timeout: SEND_TIMEOUT_MS });

    await browser.waitUntil(async () => (await MAILPIT_CLIENT.listMessageIds()).length > 0, {
      timeout: DELIVERY_TIMEOUT_MS,
      timeoutMsg: "[E2E_MAIL_NOT_RECEIVED] Mailpit received no message after the send",
    });
    const messageIds = await MAILPIT_CLIENT.listMessageIds();
    assert.equal(messageIds.length, 1, "exactly one email should be sent");
    const [messageId] = messageIds;
    assert.ok(messageId !== undefined);
    const message = await MAILPIT_CLIENT.getMessage(messageId);
    assert.deepEqual(message.toAddresses, [entries.tenantEmail]);
    assert.deepEqual(message.bccAddresses, [TEST_LANDLORD.email]);
    assert.equal(message.fromAddress, TEST_LANDLORD.email);
    assert.ok(
      message.subject.includes("octobre 2026"),
      `subject names the period: ${message.subject}`,
    );

    assert.equal(message.attachments.length, 1, "the receipt should be the only attachment");
    const [attachment] = message.attachments;
    assert.ok(attachment !== undefined);
    assert.equal(attachment.fileName, SAMPLE_RECEIPT.expectedAttachmentName);
    assert.equal(attachment.contentType, "application/pdf");
    assert.ok(attachment.size > 0, "the PDF attachment should not be empty");
    const pdf = await MAILPIT_CLIENT.getAttachmentContent({
      messageId,
      partId: attachment.partId,
    });
    assert.equal(
      new TextDecoder().decode(pdf.subarray(0, PDF_SIGNATURE.length)),
      PDF_SIGNATURE,
      "the attachment should be a PDF document",
    );
  });
});
