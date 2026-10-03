import { describe, expect, it } from "vitest";

import type { ReceiptInput } from "./api";
import {
  INITIAL_SCREEN_STATE,
  nextScreenState,
  type ScreenEvent,
  type ScreenState,
} from "./screen-state";
import { sampleReceiptInput } from "./test-fixtures";

const INPUT: ReceiptInput = sampleReceiptInput();
const OTHER_INPUT: ReceiptInput = { ...sampleReceiptInput(), rentCents: 70_000 };
const PREVIEW_HTML = "<html><body>Quittance de loyer</body></html>";
const OTHER_PREVIEW_HTML = "<html><body>Autre quittance</body></html>";

const STATES = {
  idle: { status: "idle" },
  previewPending: { status: "previewing", input: INPUT, previewHtml: null },
  previewReady: { status: "previewing", input: INPUT, previewHtml: PREVIEW_HTML },
  sending: { status: "sending", input: INPUT, previewHtml: PREVIEW_HTML },
  sent: { status: "sent", input: INPUT, previewHtml: PREVIEW_HTML },
  previewError: { status: "error", code: "template", sendRetry: null },
  sendError: {
    status: "error",
    code: "mail",
    sendRetry: { input: INPUT, previewHtml: PREVIEW_HTML },
  },
} satisfies Record<string, ScreenState>;

// Les événements de réponse portent INPUT, la saisie des états ; les variantes
// « stale » répondent pour une autre saisie.
const EVENTS = {
  previewRequested: { type: "previewRequested", input: OTHER_INPUT },
  previewReceived: { type: "previewReceived", input: INPUT, html: PREVIEW_HTML },
  stalePreviewReceived: { type: "previewReceived", input: OTHER_INPUT, html: OTHER_PREVIEW_HTML },
  previewFailed: { type: "previewFailed", input: INPUT, code: "template" },
  stalePreviewFailed: { type: "previewFailed", input: OTHER_INPUT, code: "template" },
  sendRequested: { type: "sendRequested" },
  sendSucceeded: { type: "sendSucceeded" },
  sendFailed: { type: "sendFailed", code: "mail" },
  formEdited: { type: "formEdited" },
} satisfies Record<string, ScreenEvent>;

type StateName = keyof typeof STATES;
type EventName = keyof typeof EVENTS;

const PENDING_OTHER_PREVIEW: ScreenState = {
  status: "previewing",
  input: OTHER_INPUT,
  previewHtml: null,
};

const MOVES: [StateName, EventName, ScreenState][] = [
  ["idle", "previewRequested", PENDING_OTHER_PREVIEW],
  ["previewPending", "previewRequested", PENDING_OTHER_PREVIEW],
  ["previewPending", "previewReceived", STATES.previewReady],
  ["previewPending", "previewFailed", STATES.previewError],
  ["previewPending", "formEdited", STATES.idle],
  ["previewReady", "previewRequested", PENDING_OTHER_PREVIEW],
  ["previewReady", "sendRequested", STATES.sending],
  ["previewReady", "formEdited", STATES.idle],
  ["sending", "sendSucceeded", STATES.sent],
  ["sending", "sendFailed", STATES.sendError],
  ["sent", "previewRequested", PENDING_OTHER_PREVIEW],
  ["sent", "formEdited", STATES.idle],
  ["previewError", "previewRequested", PENDING_OTHER_PREVIEW],
  ["previewError", "formEdited", STATES.idle],
  ["sendError", "previewRequested", PENDING_OTHER_PREVIEW],
  ["sendError", "sendRequested", STATES.sending],
  ["sendError", "formEdited", STATES.idle],
];

// Tout couple (état, événement) absent de MOVES est une transition ignorée.
const IGNORED: [StateName, EventName][] = (Object.keys(STATES) as StateName[]).flatMap(
  (stateName) =>
    (Object.keys(EVENTS) as EventName[])
      .filter((eventName) => !MOVES.some(([from, on]) => from === stateName && on === eventName))
      .map((eventName): [StateName, EventName] => [stateName, eventName]),
);

describe("INITIAL_SCREEN_STATE", () => {
  it("starts idle", () => {
    // Arrange & Act
    const state = INITIAL_SCREEN_STATE;

    // Assert
    expect(state).toEqual({ status: "idle" });
  });
});

describe("nextScreenState", () => {
  it.each(MOVES)("moves %s on %s", (stateName, eventName, expectedState) => {
    // Arrange
    const state = STATES[stateName];
    const event = EVENTS[eventName];

    // Act
    const nextState = nextScreenState(state, event);

    // Assert
    expect(nextState).toEqual(expectedState);
  });

  it.each(IGNORED)("ignores %s on %s by returning the same state", (stateName, eventName) => {
    // Arrange
    const state = STATES[stateName];
    const event = EVENTS[eventName];

    // Act
    const nextState = nextScreenState(state, event);

    // Assert
    expect(nextState).toBe(state);
  });

  it("accepts a preview whose input is a structurally equal copy of the requested one", () => {
    // Arrange
    const event: ScreenEvent = { type: "previewReceived", input: { ...INPUT }, html: PREVIEW_HTML };

    // Act
    const nextState = nextScreenState(STATES.previewPending, event);

    // Assert
    expect(nextState).toEqual(STATES.previewReady);
  });

  it("allows sending only after a successful preview of a valid input", () => {
    // Arrange
    const events: ScreenEvent[] = [
      { type: "sendRequested" },
      { type: "previewRequested", input: INPUT },
      { type: "sendRequested" },
      { type: "previewReceived", input: INPUT, html: PREVIEW_HTML },
      { type: "sendRequested" },
    ];

    // Act
    const finalState = events.reduce(nextScreenState, INITIAL_SCREEN_STATE);

    // Assert
    expect(finalState).toEqual(STATES.sending);
  });

  it("ignores the preview of A arriving after the form was edited", () => {
    // Arrange
    const events: ScreenEvent[] = [
      { type: "previewRequested", input: INPUT },
      { type: "formEdited" },
      { type: "previewReceived", input: INPUT, html: PREVIEW_HTML },
      { type: "sendRequested" },
    ];

    // Act
    const finalState = events.reduce(nextScreenState, INITIAL_SCREEN_STATE);

    // Assert
    expect(finalState).toEqual(STATES.idle);
  });

  it("ignores the preview of A arriving after B was requested and keeps sending impossible", () => {
    // Arrange
    const events: ScreenEvent[] = [
      { type: "previewRequested", input: INPUT },
      { type: "formEdited" },
      { type: "previewRequested", input: OTHER_INPUT },
      { type: "previewReceived", input: INPUT, html: PREVIEW_HTML },
      { type: "sendRequested" },
    ];

    // Act
    const finalState = events.reduce(nextScreenState, INITIAL_SCREEN_STATE);

    // Assert
    expect(finalState).toEqual(PENDING_OTHER_PREVIEW);
  });

  it("sends B once its own preview arrives after the stale preview of A", () => {
    // Arrange
    const events: ScreenEvent[] = [
      { type: "previewRequested", input: INPUT },
      { type: "previewRequested", input: OTHER_INPUT },
      { type: "previewReceived", input: INPUT, html: PREVIEW_HTML },
      { type: "previewReceived", input: OTHER_INPUT, html: OTHER_PREVIEW_HTML },
      { type: "sendRequested" },
    ];

    // Act
    const finalState = events.reduce(nextScreenState, INITIAL_SCREEN_STATE);

    // Assert
    expect(finalState).toEqual({
      status: "sending",
      input: OTHER_INPUT,
      previewHtml: OTHER_PREVIEW_HTML,
    });
  });

  it("keeps sending when the form is edited because F7c disables the form while sending", () => {
    // Arrange
    const events: ScreenEvent[] = [{ type: "formEdited" }, { type: "sendSucceeded" }];

    // Act
    const finalState = events.reduce(nextScreenState, STATES.sending);

    // Assert
    expect(finalState).toEqual(STATES.sent);
  });

  it("retries a failed send with the previewed input without a new preview", () => {
    // Arrange
    const events: ScreenEvent[] = [{ type: "sendFailed", code: "mail" }, { type: "sendRequested" }];

    // Act
    const finalState = events.reduce(nextScreenState, STATES.sending);

    // Assert
    expect(finalState).toEqual(STATES.sending);
  });
});
