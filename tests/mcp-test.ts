#!/usr/bin/env gjs

type GioNamespace = import("./gjs-ambient").GioNamespace;
type GLibNamespace = import("./gjs-ambient").GLibNamespace;
type GioUnixNamespace = import("./gjs-ambient").GioUnixNamespace;

declare const imports: {
  gi: {
    versions: Record<string, string>;
    Gio: GioNamespace;
    GLib: GLibNamespace;
    GioUnix: GioUnixNamespace;
  };
  byteArray: {
    toString(bytes: Uint8Array): string;
    fromString(text: string): Uint8Array;
  };
};
declare function logError(error: unknown, message?: string): void;
declare function print(message: string): void;

type GioModule = GioNamespace;
type GLibModule = GLibNamespace;
type GioUnixModule = GioUnixNamespace;

imports.gi.versions.Gio = "2.0";

const Gio: GioModule = imports.gi.Gio;
const GLib: GLibModule = imports.gi.GLib;
const GioUnix: GioUnixModule = imports.gi.GioUnix;
const ByteArray = imports.byteArray;

const PROTOCOL_VERSION = "2024-11-05";

interface JsonMap {
  [key: string]: JsonValue;
}

type JsonValue = null | boolean | number | string | JsonMap | JsonValue[];
type RpcId = number;

interface RpcRequest {
  jsonrpc: "2.0";
  id: RpcId;
  method: string;
  params?: JsonMap;
}

interface RpcNotification {
  jsonrpc: "2.0";
  method: string;
  params?: JsonMap;
}

interface RpcSuccess {
  jsonrpc: "2.0";
  id: RpcId;
  result: JsonValue;
}

interface RpcErrorBody {
  code: number;
  message: string;
  data?: JsonValue;
}

interface RpcFailure {
  jsonrpc: "2.0";
  id: RpcId;
  error: RpcErrorBody;
}

type RpcMessage = RpcSuccess | RpcFailure;

interface PendingRequest {
  resolve: (value: JsonValue) => void;
  reject: (reason?: unknown) => void;
}

type GioCancellable = InstanceType<typeof Gio.Cancellable>;
type GioInputStream = InstanceType<typeof GioUnix.InputStream>;
type GioOutputStream = InstanceType<typeof GioUnix.OutputStream>;

function envString(name: string): string {
  const value = GLib.getenv(name);
  return value ?? "";
}

function requireEnvString(name: string): string {
  const value = envString(name);
  if (!value) {
    throw new Error(`${name} is required`);
  }
  return value;
}

function decodeBytes(bytes: Uint8Array): string {
  return imports.byteArray.toString(bytes);
}

function encodeLine(message: JsonValue): Uint8Array {
  return ByteArray.fromString(`${JSON.stringify(message)}\n`);
}

function isJsonMap(value: JsonValue | undefined): value is JsonMap {
  return (
    value !== null &&
    value !== undefined &&
    !Array.isArray(value) &&
    typeof value === "object"
  );
}

function isRpcFailure(value: JsonMap): boolean {
  return (
    typeof value.id === "number" &&
    isJsonMap(value.error) &&
    typeof value.error.code === "number" &&
    typeof value.error.message === "string"
  );
}

function isRpcSuccess(value: JsonMap): boolean {
  return typeof value.id === "number" && "result" in value;
}

function isRpcNotification(value: JsonMap): boolean {
  return typeof value.method === "string" && !("id" in value);
}

function isRpcMessage(value: JsonValue): boolean {
  if (!isJsonMap(value) || value.jsonrpc !== "2.0") {
    return false;
  }

  return isRpcFailure(value) || isRpcSuccess(value) || isRpcNotification(value);
}

class JsonLineChannel {
  private readonly input: GioInputStream;
  private readonly output: GioOutputStream;
  private readonly cancellable: GioCancellable;
  private buffer = "";
  private closed = false;
  private started = false;
  private readonly onMessage: (message: RpcMessage) => void;

  constructor(
    input: GioInputStream,
    output: GioOutputStream,
    onMessage: (message: RpcMessage) => void,
  ) {
    this.input = input;
    this.output = output;
    this.cancellable = new Gio.Cancellable();
    this.onMessage = onMessage;
  }

  start(): void {
    if (this.started) {
      return;
    }

    this.started = true;
    print("[CHANNEL] Starting to read from input stream");
    this.readStdout();
  }

  async send(message: RpcRequest | RpcNotification): Promise<void> {
    const json = JSON.stringify(message);
    print(`[CHANNEL] Sending: ${json}`);
    const bytes = encodeLine(message as unknown as JsonValue);

    await new Promise<void>((resolve, reject) => {
      this.output.write_bytes_async(
        new GLib.Bytes(bytes),
        GLib.PRIORITY_DEFAULT,
        null,
        (_stream: unknown, result: unknown) => {
          try {
            this.output.write_bytes_finish(result);
            print("[CHANNEL] Message sent successfully");
            resolve();
          } catch (error) {
            print(`[CHANNEL] Error sending message: ${error}`);
            reject(error);
          }
        },
      );
    });
  }

  close(): void {
    if (this.closed) {
      return;
    }

    print("[CHANNEL] Closing channel");
    this.closed = true;
    this.cancellable.cancel();

    try {
      this.output.close(null);
    } catch (error) {
      if (!this.isExpectedCloseError(error)) {
        logError(error, "closing output");
      }
    }
  }

  private readStdout(): void {
    const pump = () => {
      if (this.closed) {
        return;
      }

      this.input.read_bytes_async(
        4096,
        GLib.PRIORITY_DEFAULT,
        this.cancellable,
        (_stream: unknown, result: unknown) => {
          try {
            const bytes = this.input.read_bytes_finish(result);
            const chunk = bytes.toArray();

            if (chunk.length === 0) {
              print("[CHANNEL] Received 0 bytes, stream ended");
              return;
            }

            const text = decodeBytes(chunk);
            print(`[CHANNEL] Received ${chunk.length} bytes: ${text.substring(0, 200)}...`);
            this.buffer += text;
            this.drainBuffer();
            pump();
          } catch (error) {
            if (!this.closed && !this.isExpectedCloseError(error)) {
              print(`[CHANNEL] Error reading: ${error}`);
              logError(error, "reading input");
            }
          }
        },
      );
    };

    pump();
  }

  private drainBuffer(): void {
    while (true) {
      const newlineIndex = this.buffer.indexOf("\n");
      if (newlineIndex === -1) {
        return;
      }

      const line = this.buffer.slice(0, newlineIndex).trim();
      this.buffer = this.buffer.slice(newlineIndex + 1);
      if (!line) {
        continue;
      }

      print(`[CHANNEL] Processing line: ${line}`);
      const message = this.parseMessage(line);
      if (message) {
        print(`[CHANNEL] Parsed message, calling handler`);
        this.onMessage(message);
      }
    }
  }

  private parseMessage(line: string): RpcMessage | null {
    try {
      const parsed = JSON.parse(line) as JsonValue;
      print(`[CHANNEL] Parsed JSON: ${JSON.stringify(parsed)}`);
      
      if (!isJsonMap(parsed) || !isRpcMessage(parsed)) {
        print(`[CHANNEL] Not a valid RPC message: ${line}`);
        return null;
      }

      if (isRpcNotification(parsed)) {
        print(`[CHANNEL] Ignoring notification: ${parsed.method}`);
        return null;
      }

      print(`[CHANNEL] Valid RPC message received`);
      return parsed as unknown as RpcMessage;
    } catch (error) {
      print(`[CHANNEL] Failed to parse: ${error}`);
      logError(error, `Failed to parse MCP message: ${line}`);
      return null;
    }
  }

  private isExpectedCloseError(error: unknown): boolean {
    if (!(error instanceof Error)) {
      return false;
    }

    return (
      error.message.includes("Operation was cancelled") ||
      error.message.includes("Stream is already closed")
    );
  }
}

class McpConnection {
  private readonly channel: JsonLineChannel;
  private readonly pending = new Map<RpcId, PendingRequest>();
  private nextId = 1;
  private ready = false;

  constructor() {
    print("[MCP] Initializing MCP connection");
    const socketFd = requireEnvString("MINITASK_GUI_SOCKET_FD");
    print(`[MCP] Socket FD: ${socketFd}`);
    const fd = Number.parseInt(socketFd, 10);
    if (!Number.isInteger(fd) || fd < 0) {
      throw new Error(
        `MINITASK_GUI_SOCKET_FD must be a non-negative integer, got: ${socketFd}`,
      );
    }

    const input = new GioUnix.InputStream({ fd, close_fd: true });
    const output = new GioUnix.OutputStream({ fd, close_fd: false });
    print("[MCP] Created input/output streams");
    
    this.channel = new JsonLineChannel(input, output, (message) => {
      print("[MCP] Received message from channel");
      this.handleMessage(message);
    });
  }

  async initialize(): Promise<void> {
    if (this.ready) {
      print("[MCP] Already initialized");
      return;
    }

    print("[MCP] Starting channel");
    this.channel.start();
    
    print("[MCP] Sending initialize request");
    const initResult = await this.request("initialize", {
      protocolVersion: PROTOCOL_VERSION,
      capabilities: {},
      clientInfo: {
        name: "gjs-minitask-test",
        version: "0.1.0",
      },
    });
    print(`[MCP] Initialize result: ${JSON.stringify(initResult)}`);
    
    print("[MCP] Sending initialized notification");
    await this.notify("notifications/initialized", {});
    
    this.ready = true;
    print("[MCP] Connection ready");
  }

  async request(method: string, params?: JsonMap): Promise<JsonValue> {
    const id = this.nextId;
    this.nextId += 1;

    print(`[MCP] Creating request ${id}: ${method}`);
    const message: RpcRequest = {
      jsonrpc: "2.0",
      id,
      method,
      params,
    };

    return new Promise<JsonValue>((resolve, reject) => {
      print(`[MCP] Registering pending request ${id}`);
      this.pending.set(id, { resolve, reject });
      this.channel.send(message).catch((error) => {
        print(`[MCP] Error sending request ${id}: ${error}`);
        this.pending.delete(id);
        reject(error);
      });
    });
  }

  async notify(method: string, params?: JsonMap): Promise<void> {
    print(`[MCP] Sending notification: ${method}`);
    const message: RpcNotification = {
      jsonrpc: "2.0",
      method,
      params,
    };
    await this.channel.send(message);
  }

  close(): void {
    print("[MCP] Closing connection");
    for (const pending of this.pending.values()) {
      pending.reject(new Error("Connection closed"));
    }
    this.pending.clear();
    this.channel.close();
  }

  private handleMessage(message: RpcMessage): void {
    print(`[MCP] Handling message: ${JSON.stringify(message)}`);
    
    if (!("id" in message) || typeof message.id !== "number") {
      print("[MCP] Message has no valid id");
      return;
    }

    const pending = this.pending.get(message.id);
    if (!pending) {
      print(`[MCP] No pending request for id ${message.id}`);
      return;
    }

    print(`[MCP] Found pending request ${message.id}`);
    this.pending.delete(message.id);
    
    if ("error" in message) {
      print(`[MCP] Request ${message.id} failed: ${message.error.message}`);
      pending.reject(new Error(message.error.message));
      return;
    }

    print(`[MCP] Request ${message.id} succeeded`);
    pending.resolve(message.result);
  }
}

async function main(): Promise<void> {
  try {
    print("=== MCP Test Program ===");
    print("");
    
    const taskFile = requireEnvString("MINITASK_GUI_TASK_FILE");
    print(`Task file: ${taskFile}`);
    print("");
    
    const connection = new McpConnection();
    
    print("Initializing connection...");
    await connection.initialize();
    print("Connection initialized!");
    print("");
    
    // Test 1: List tasks
    print("TEST 1: Listing tasks");
    const listResult = await connection.request("tools/call", {
      name: "list",
      arguments: {
        file: taskFile,
        verbose: true,
      },
    });
    print(`List result: ${JSON.stringify(listResult, null, 2)}`);
    print("");
    
    // Test 2: Create a task
    print("TEST 2: Creating a task");
    const createResult = await connection.request("tools/call", {
      name: "new",
      arguments: {
        file: taskFile,
        content: "Test task from MCP test program",
      },
    });
    print(`Create result: ${JSON.stringify(createResult, null, 2)}`);
    print("");
    
    // Test 3: List tasks again
    print("TEST 3: Listing tasks again");
    const listResult2 = await connection.request("tools/call", {
      name: "list",
      arguments: {
        file: taskFile,
        verbose: true,
      },
    });
    print(`List result 2: ${JSON.stringify(listResult2, null, 2)}`);
    print("");
    
    connection.close();
    print("=== Tests completed ===");
  } catch (error) {
    print(`ERROR: ${error}`);
    if (error instanceof Error) {
      print(`Stack: ${error.stack}`);
    }
  }
}

void main();
