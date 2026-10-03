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
const FAILURE_MESSAGE = "SMTP server unreachable";

const STATES = {
  idle: { status: "idle" },
  previewPending: { status: "previewing", input: INPUT, isPreviewReady: false },
  previewReady: { status: "previewing", input: INPUT, isPreviewReady: true },
  sending: { status: "sending", input: INPUT },
  sent: { status: "sent", input: INPUT },
  previewError: { status: "error", message: FAILURE_MESSAGE, previewedInput: null },
  sendError: { status: "error", message: FAILURE_MESSAGE, previewedInput: INPUT },
} satisfies Record<string, ScreenState>;

const EVENTS = {
  previewRequested: { type: "previewRequested", input: OTHER_INPUT },
  previewReceived: { type: "previewReceived" },
  sendRequested: { type: "sendRequested" },
  sendSucceeded: { type: "sendSucceeded" },
  operationFailed: { type: "operationFailed", message: FAILURE_MESSAGE },
  formEdited: { type: "formEdited" },
} satisfies Record<string, ScreenEvent>;

type StateName = keyof typeof STATES;
type EventName = keyof typeof EVENTS;

describe("INITIAL_SCREEN_STATE", () => {
  it("starts idle", () => {
    // Arrange & Act
    const state = INITIAL_SCREEN_STATE;

    // Assert
    expect(state).toEqual({ status: "idle" });
  });
});

describe("nextScreenState", () => {
  it.each<[StateName, EventName, ScreenState]>([
    [
      "idle",
      "previewRequested",
      { status: "previewing", input: OTHER_INPUT, isPreviewReady: false },
    ],
    ["idle", "formEdited", STATES.idle],
    ["previewPending", "previewReceived", STATES.previewReady],
    [
      "previewPending",
      "previewRequested",
      { status: "previewing", input: OTHER_INPUT, isPreviewReady: false },
    ],
    ["previewPending", "operationFailed", STATES.previewError],
    ["previewPending", "formEdited", STATES.idle],
    ["previewReady", "sendRequested", STATES.sending],
    [
      "previewReady",
      "previewRequested",
      { status: "previewing", input: OTHER_INPUT, isPreviewReady: false },
    ],
    ["previewReady", "formEdited", STATES.idle],
    ["sending", "sendSucceeded", STATES.sent],
    ["sending", "operationFailed", STATES.sendError],
    [
      "sent",
      "previewRequested",
      { status: "previewing", input: OTHER_INPUT, isPreviewReady: false },
    ],
    ["sent", "formEdited", STATES.idle],
    [
      "previewError",
      "previewRequested",
      { status: "previewing", input: OTHER_INPUT, isPreviewReady: false },
    ],
    ["previewError", "formEdited", STATES.idle],
    ["sendError", "sendRequested", STATES.sending],
    [
      "sendError",
      "previewRequested",
      { status: "previewing", input: OTHER_INPUT, isPreviewReady: false },
    ],
    ["sendError", "formEdited", STATES.idle],
  ])("moves %s on %s", (stateName, eventName, expectedState) => {
    // Arrange
    const state = STATES[stateName];
    const event = EVENTS[eventName];

    // Act
    const nextState = nextScreenState(state, event);

    // Assert
    expect(nextState).toEqual(expectedState);
  });

  it.each<[StateName, EventName]>([
    ["idle", "previewReceived"],
    ["idle", "sendRequested"],
    ["idle", "sendSucceeded"],
    ["idle", "operationFailed"],
    ["previewPending", "sendRequested"],
    ["previewPending", "sendSucceeded"],
    ["previewReady", "previewReceived"],
    ["previewReady", "sendSucceeded"],
    ["previewReady", "operationFailed"],
    ["sending", "previewRequested"],
    ["sending", "previewReceived"],
    ["sending", "sendRequested"],
    ["sending", "formEdited"],
    ["sent", "previewReceived"],
    ["sent", "sendRequested"],
    ["sent", "sendSucceeded"],
    ["sent", "operationFailed"],
    ["previewError", "previewReceived"],
    ["previewError", "sendRequested"],
    ["previewError", "sendSucceeded"],
    ["previewError", "operationFailed"],
    ["sendError", "previewReceived"],
    ["sendError", "sendSucceeded"],
    ["sendError", "operationFailed"],
  ])("ignores %s on %s by returning the same state", (stateName, eventName) => {
    // Arrange
    const state = STATES[stateName];
    const event = EVENTS[eventName];

    // Act
    const nextState = nextScreenState(state, event);

    // Assert
    expect(nextState).toBe(state);
  });

  it("allows sending only after a successful preview of a valid input", () => {
    // Arrange
    const events: ScreenEvent[] = [
      { type: "sendRequested" },
      { type: "previewRequested", input: INPUT },
      { type: "sendRequested" },
      { type: "previewReceived" },
      { type: "sendRequested" },
    ];

    // Act
    const finalState = events.reduce(nextScreenState, INITIAL_SCREEN_STATE);

    // Assert
    expect(finalState).toEqual(STATES.sending);
  });

  it("drops a preview that arrives after the form was edited", () => {
    // Arrange
    const events: ScreenEvent[] = [
      { type: "previewRequested", input: INPUT },
      { type: "formEdited" },
      { type: "previewReceived" },
      { type: "sendRequested" },
    ];

    // Act
    const finalState = events.reduce(nextScreenState, INITIAL_SCREEN_STATE);

    // Assert
    expect(finalState).toEqual(STATES.idle);
  });
});
