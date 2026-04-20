import type { UnlistenFn } from "@tauri-apps/api/event";
import { vi } from "vitest";
import type {
  TauriCommandArgs,
  TauriCommandName,
  TauriCommandResult,
  TauriEventName,
  TauriEventPayload,
} from "./contracts";

type MaybePromise<TValue> = TValue | Promise<TValue>;
type CommandStub<TName extends TauriCommandName> =
  | TauriCommandResult<TName>
  | ((args: TauriCommandArgs<TName>) => MaybePromise<TauriCommandResult<TName>>);
type CommandStubMap = Partial<{ [TName in TauriCommandName]: CommandStub<TName> }>;
type EventCallback<TName extends TauriEventName> = (payload: TauriEventPayload<TName>) => void;

const commandStubs = new Map<TauriCommandName, (args: unknown) => Promise<unknown>>();
const eventListeners = new Map<TauriEventName, Set<(payload: unknown) => void>>();

export const invokeMock = vi.fn(async (commandName: string, args?: unknown) => {
  const commandStub = commandStubs.get(commandName as TauriCommandName);
  if (!commandStub) {
    throw new Error(`Missing IPC mock for ${commandName}`);
  }
  return commandStub(args);
});

export const listenMock = vi.fn(
  async (eventName: string, callback: (event: { payload: unknown }) => void) => {
    const listeners = ensureListeners(eventName as TauriEventName);
    const payloadCallback = (payload: unknown) => callback({ payload });
    listeners.add(payloadCallback);
    const unlisten: UnlistenFn = () => {
      listeners.delete(payloadCallback);
    };
    return unlisten;
  },
);

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));
vi.mock("@tauri-apps/api/event", () => ({ listen: listenMock }));

function ensureListeners<TName extends TauriEventName>(eventName: TName) {
  const current = eventListeners.get(eventName);
  if (current) {
    return current;
  }
  const created = new Set<(payload: unknown) => void>();
  eventListeners.set(eventName, created);
  return created;
}

function toCommandHandler(commandStub: CommandStub<TauriCommandName>) {
  if (typeof commandStub === "function") {
    return async (args: unknown) =>
      (commandStub as (value: unknown) => MaybePromise<unknown>)(args);
  }
  return async () => commandStub;
}

export function mockTauriCommand<TName extends TauriCommandName>(
  commandName: TName,
  commandStub: CommandStub<TName>,
) {
  commandStubs.set(commandName, toCommandHandler(commandStub));
}

export function mockTauriCommands(commandMap: CommandStubMap) {
  for (const commandName of Object.keys(commandMap) as TauriCommandName[]) {
    const commandStub = commandMap[commandName];
    if (commandStub === undefined) {
      continue;
    }
    commandStubs.set(
      commandName,
      toCommandHandler(commandStub as unknown as CommandStub<TauriCommandName>),
    );
  }
}

export async function emitTauriEvent<TName extends TauriEventName>(
  eventName: TName,
  payload: TauriEventPayload<TName>,
) {
  const listeners = eventListeners.get(eventName);
  if (!listeners) {
    return;
  }
  for (const listener of listeners) {
    listener(payload);
  }
  await Promise.resolve();
}

export function subscribeTauriEvent<TName extends TauriEventName>(
  eventName: TName,
  callback: EventCallback<TName>,
) {
  const listeners = ensureListeners(eventName);
  const payloadCallback = (payload: unknown) => callback(payload as TauriEventPayload<TName>);
  listeners.add(payloadCallback);
  return () => {
    listeners.delete(payloadCallback);
  };
}

export function getEventListenerCount<TName extends TauriEventName>(eventName: TName) {
  return eventListeners.get(eventName)?.size ?? 0;
}

export function resetTauriMocks() {
  commandStubs.clear();
  eventListeners.clear();
  invokeMock.mockClear();
  listenMock.mockClear();
}
