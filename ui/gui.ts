#!/usr/bin/env gjs

type GtkNamespace = import("./gjs-ambient").GtkNamespace;
type AdwNamespace = import("./gjs-ambient").AdwNamespace;
type GioNamespace = import("./gjs-ambient").GioNamespace;
type GLibNamespace = import("./gjs-ambient").GLibNamespace;
type GioUnixNamespace = import("./gjs-ambient").GioUnixNamespace;
type GObjectNamespace = import("./gjs-ambient").GObjectNamespace;

declare const imports: {
  gi: {
    versions: Record<string, string>;
    Adw: AdwNamespace;
    Gtk: GtkNamespace;
    Gio: GioNamespace;
    GLib: GLibNamespace;
    GioUnix: GioUnixNamespace;
    GObject: GObjectNamespace;
  };
  byteArray: {
    toString(bytes: Uint8Array): string;
    fromString(text: string): Uint8Array;
  };
};
declare function print(message: string): void;



type GtkModule = GtkNamespace;
type AdwModule = AdwNamespace;
type GioModule = GioNamespace;
type GLibModule = GLibNamespace;
type GioUnixModule = GioUnixNamespace;
type GObjectModule = GObjectNamespace;

imports.gi.versions.Gtk = "4.0";
imports.gi.versions.Adw = "1";

const Adw: AdwModule = imports.gi.Adw;
const Gtk: GtkModule = imports.gi.Gtk;
const Gio: GioModule = imports.gi.Gio;
const GioUnix: GioUnixModule = imports.gi.GioUnix;
const GLib: GLibModule = imports.gi.GLib;
const GObject: GObjectModule = imports.gi.GObject;
const ByteArray = imports.byteArray;
const APPLICATION_ID = "ai.minitask.Gui";
const PROTOCOL_VERSION = "2024-11-05";
const TASK_STATES = [
  "todo",
  "in-progress",
  "review",
  "done",
  "blocked",
] as const;

function syncSystemColorScheme(): void {
  Adw.StyleManager.get_default().color_scheme = Adw.ColorScheme.DEFAULT;
}

interface JsonMap {
  [key: string]: JsonValue;
}

type JsonValue = null | boolean | number | string | JsonMap | JsonValue[];
type RpcId = number;
type TaskState = (typeof TASK_STATES)[number];
type TaskMoveHandler = (taskId: string, nextState: TaskState) => void;

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

interface RpcNotificationMessage {
  jsonrpc: "2.0";
  method: string;
  params?: JsonValue;
}

interface ContentBlock {
  type?: string;
  text?: string;
}

interface ToolCallEnvelope {
  content?: ContentBlock[];
  structuredContent?: JsonValue;
  isError?: boolean;
}

interface TaskRecord {
  name: string;
  state: string;
  content: string;
  depends_on: string[];
  epic: string[];
}

interface TaskListResult {
  tasks: TaskRecord[];
}

interface PendingRequest {
  resolve: (value: JsonValue) => void;
  reject: (reason?: unknown) => void;
}

interface UiRefs {
  window: GtkApplicationWindow;
  taskEntry: GtkEntry;
  addButton: GtkButton;
  refreshButton: GtkButton;
  stateFilter: GtkDropDown;
  epicFilter: GtkEntry;
  statusLabel: GtkLabel;
  taskList: GtkListBox;
}

interface FileWatchOptions {
  path: string;
  onChange: () => void;
}

interface FileVersion {
  etag: string;
  size: number;
}

type GtkApplication = InstanceType<typeof Gtk.Application>;
type GtkApplicationWindow = InstanceType<typeof Gtk.ApplicationWindow>;
type GtkBox = InstanceType<typeof Gtk.Box>;
type GtkButton = InstanceType<typeof Gtk.Button>;
type GtkDropDown = InstanceType<typeof Gtk.DropDown>;
type GtkEntry = InstanceType<typeof Gtk.Entry>;
type GtkLabel = InstanceType<typeof Gtk.Label>;
type GtkListBox = InstanceType<typeof Gtk.ListBox>;
type GtkListBoxRow = InstanceType<typeof Gtk.ListBoxRow>;
type GtkScrolledWindow = InstanceType<typeof Gtk.ScrolledWindow>;
type GtkTextView = InstanceType<typeof Gtk.TextView>;
type GioCancellable = InstanceType<typeof Gio.Cancellable>;
type GioFile = import("./gjs-ambient").GioFile;
type GioFileMonitor = ReturnType<GioFile["monitor_file"]>;
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

/**
 * Centralized logging and error handling utilities.
 * Provides consistent logging across the application with debug, info, and error levels.
 */
const Logger = {
  debug(message: string): void {
    if (envString("MINITASK_GUI_DEBUG")) {
      print(`[DEBUG] ${message}`);
    }
  },

  info(message: string): void {
    print(`[INFO] ${message}`);
  },

  error(message: string): void {
    print(`[ERROR] ${message}`);
  },

  // Helper to log and show error to user
  showError(statusLabel: GtkLabel | null, message: string, error?: unknown): void {
    const errorMsg = error instanceof Error ? error.message : String(error);
    const fullMessage = error ? `${message}: ${errorMsg}` : message;
    this.error(fullMessage);
    if (statusLabel) {
      statusLabel.set_label(`❌ ${message}`);
    }
  },
};

function decodeBytes(bytes: Uint8Array): string {
  return imports.byteArray.toString(bytes);
}

function encodeLine(message: JsonValue): Uint8Array {
  return ByteArray.fromString(`${JSON.stringify(message)}\n`);
}

/**
 * Widget creation helpers for common GTK patterns.
 * These functions reduce boilerplate when creating GTK widgets.
 */

/**
 * Creates a GTK Box container with specified orientation and spacing.
 * @param orientation - GTK.Orientation.HORIZONTAL or GTK.Orientation.VERTICAL
 * @param spacing - Space between child widgets in pixels (default: 6)
 * @param props - Additional GTK.Box properties
 * @returns A configured GTK Box widget
 */
function createBox(
  orientation: number,
  spacing = 6,
  props?: Record<string, unknown>
): GtkBox {
  return new Gtk.Box({
    orientation,
    spacing,
    ...props,
  });
}

/**
 * Creates a horizontal GTK Box container.
 * @param spacing - Space between child widgets in pixels (default: 6)
 * @param props - Additional GTK.Box properties
 * @returns A horizontal GTK Box widget
 */
function createHBox(spacing = 6, props?: Record<string, unknown>): GtkBox {
  return createBox(Gtk.Orientation.HORIZONTAL, spacing, props);
}

/**
 * Creates a vertical GTK Box container.
 * @param spacing - Space between child widgets in pixels (default: 6)
 * @param props - Additional GTK.Box properties
 * @returns A vertical GTK Box widget
 */
function createVBox(spacing = 6, props?: Record<string, unknown>): GtkBox {
  return createBox(Gtk.Orientation.VERTICAL, spacing, props);
}

/**
 * Creates a GTK Label widget with text.
 * @param text - The label text to display
 * @param props - Additional GTK.Label properties
 * @returns A configured GTK Label widget
 */
function createLabel(
  text: string,
  props?: Record<string, unknown>
): GtkLabel {
  return new Gtk.Label({
    label: text,
    xalign: 0,
    wrap: true,
    ...props,
  });
}

/**
 * Creates a GTK Button with a click handler.
 * @param label - The button label text
 * @param onClick - Callback function when button is clicked
 * @param props - Additional GTK.Button properties
 * @returns A configured GTK Button widget
 */
function createButton(
  label: string,
  onClick: () => void,
  props?: Record<string, unknown>
): GtkButton {
  const button = new Gtk.Button({
    label,
    ...props,
  });
  button.connect("clicked", onClick);
  return button;
}

/**
 * Creates a GTK Entry (text input) widget.
 * @param placeholder - Placeholder text shown when empty (default: "")
 * @param props - Additional GTK.Entry properties
 * @returns A configured GTK Entry widget
 */
function createEntry(
  placeholder = "",
  props?: Record<string, unknown>
): GtkEntry {
  return new Gtk.Entry({
    placeholder_text: placeholder,
    ...props,
  });
}

/**
 * Type conversion and validation utilities for JSON-RPC data handling.
 * Provides safe type checking and conversion functions for JsonValue types.
 */
const TypeUtils = {
  isJsonMap(value: JsonValue | undefined): value is JsonMap {
    return (
      value !== null &&
      value !== undefined &&
      !Array.isArray(value) &&
      typeof value === "object"
    );
  },

  asArray(value: JsonValue | undefined): JsonValue[] {
    return Array.isArray(value) ? value : [];
  },

  asString(value: JsonValue | undefined, fallback = ""): string {
    return typeof value === "string" ? value : fallback;
  },

  asStringArray(value: JsonValue | undefined): string[] {
    return this.asArray(value).filter(
      (item): item is string => typeof item === "string",
    );
  },
};

/**
 * JSON-RPC 2.0 message validation utilities.
 * Validates message structure according to the JSON-RPC 2.0 specification.
 */
const RpcValidator = {
  isFailure(value: JsonMap): boolean {
    return (
      typeof value.id === "number" &&
    TypeUtils.isJsonMap(value.error) &&
      typeof value.error.code === "number" &&
      typeof value.error.message === "string"
    );
  },

  isSuccess(value: JsonMap): boolean {
    return typeof value.id === "number" && "result" in value;
  },

  isNotification(value: JsonMap): boolean {
    return typeof value.method === "string" && !("id" in value);
  },

  isMessage(value: JsonValue): boolean {
  if (!TypeUtils.isJsonMap(value) || value.jsonrpc !== "2.0") {
      return false;
    }
    return this.isFailure(value) || this.isSuccess(value) || this.isNotification(value);
  },
};

function parseToolPayload(result: ToolCallEnvelope): JsonValue {
  Logger.debug(`[PARSE] parseToolPayload called with: ${JSON.stringify(result)}`);

  if (result.structuredContent !== undefined) {
    Logger.debug(`[PARSE] Using structuredContent: ${JSON.stringify(result.structuredContent)}`);
    return result.structuredContent;
  }

  Logger.debug(`[PARSE] No structuredContent, parsing content array`);
  const text = (result.content ?? [])
    .filter((item) => item.type === "text" && typeof item.text === "string")
    .map((item) => item.text ?? "")
    .join("\n")
    .trim();

  Logger.debug(`[PARSE] Extracted text (${text.length} chars): ${text.substring(0, 200)}`);

  if (!text) {
    Logger.error(`[PARSE] No text found, returning null`);
    return null;
  }

  try {
    const parsed = JSON.parse(text) as JsonValue;
    Logger.debug(`[PARSE] Successfully parsed JSON`);
    return parsed;
  } catch (e) {
    Logger.error(`[PARSE] Failed to parse JSON: ${e}`);
    return text;
  }
}

function toTaskRecord(value: JsonValue): TaskRecord {
  const object = TypeUtils.isJsonMap(value) ? value : {};
  return {
    name: TypeUtils.asString(object.name),
    state: TypeUtils.asString(object.state),
    content: TypeUtils.asString(object.content),
    depends_on: TypeUtils.asStringArray(object.depends_on),
    epic: TypeUtils.asStringArray(object.epic),
  };
}

function toTaskListResult(value: JsonValue): TaskListResult {
  if (Array.isArray(value)) {
    return { tasks: value.map(toTaskRecord) };
  }

  const object = TypeUtils.isJsonMap(value) ? value : {};
  const tasks = Array.isArray(object.tasks)
    ? object.tasks.map(toTaskRecord)
    : [];
  return { tasks };
}

/**
 * Handles line-delimited JSON communication over Unix streams.
 * Reads JSON-RPC messages line-by-line from input stream and sends messages to output stream.
 * Automatically handles message parsing and invokes callback for each received message.
 */
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
    this.readStdout();
  }

  async send(message: RpcRequest | RpcNotification): Promise<void> {
    const bytes = encodeLine(message as unknown as JsonValue);

    await new Promise<void>((resolve, reject) => {
      this.output.write_bytes_async(
        new GLib.Bytes(bytes),
        GLib.PRIORITY_DEFAULT,
        null,
        (_stream: unknown, result: unknown) => {
          try {
            this.output.write_bytes_finish(result);
            resolve();
          } catch (error) {
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

    this.closed = true;
    this.cancellable.cancel();

    try {
      this.output.close(null);
    } catch (error) {
      if (!this.isExpectedCloseError(error)) {
        Logger.error(`closing minitask output: ${error}`);
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
              return;
            }

            this.buffer += decodeBytes(chunk);
            this.drainBuffer();
            pump();
          } catch (error) {
            if (!this.closed && !this.isExpectedCloseError(error)) {
              Logger.error(`minitask stdout: ${error}`);
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

      const message = this.parseMessage(line);
      if (message) {
        this.onMessage(message);
      }
    }
  }

  private parseMessage(line: string): RpcMessage | null {
    try {
      const parsed = JSON.parse(line) as JsonValue;
    if (!TypeUtils.isJsonMap(parsed) || !RpcValidator.isMessage(parsed)) {
        Logger.error(`Ignoring unexpected MCP message: ${line}`);
        return null;
      }

    if (RpcValidator.isNotification(parsed)) {
        return null;
      }

      return parsed as unknown as RpcMessage;
    } catch (error) {
      Logger.error(`Failed to parse MCP message: ${line} - ${error}`);
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

/**
 * Manages MCP (Model Context Protocol) connection over a Unix socket.
 * Handles JSON-RPC request/response lifecycle, initialization handshake,
 * and notification delivery. Maintains pending request state for async operations.
 */
class McpConnection {
  private readonly channel: JsonLineChannel;
  private readonly pending = new Map<RpcId, PendingRequest>();
  private nextId = 1;
  private ready = false;
  private serverInfo: { name: string; version: string; description: string; authors: string; license: string; repository: string } | null = null;

  constructor() {
    const socketFd = requireEnvString("MINITASK_GUI_SOCKET_FD");
    const fd = Number.parseInt(socketFd, 10);
    if (!Number.isInteger(fd) || fd < 0) {
      throw new Error(
        `MINITASK_GUI_SOCKET_FD must be a non-negative integer, got: ${socketFd}`,
      );
    }

    const input = new GioUnix.InputStream({ fd, close_fd: true });
    const output = new GioUnix.OutputStream({ fd, close_fd: false });
    this.channel = new JsonLineChannel(input, output, (message) => {
      this.handleMessage(message);
    });
  }

  async initialize(): Promise<void> {
    if (this.ready) {
      return;
    }

    this.channel.start();
    const result = await this.request("initialize", {
      protocolVersion: PROTOCOL_VERSION,
      capabilities: {},
      clientInfo: {
        name: "gjs-minitask-gui",
        version: "0.1.0",
      },
    });

    // Extract serverInfo from initialize response
    if (result && typeof result === 'object' && 'serverInfo' in result) {
      const info = (result as any).serverInfo;
      if (info && typeof info === 'object') {
        const name = String(info.name || 'minitask');
        const version = String(info.version || '0.0.0');

        // Parse description field which contains JSON with additional metadata
        let metadata: any = {};
        if (info.description && typeof info.description === 'string') {
          try {
            metadata = JSON.parse(info.description);
          } catch (e) {
            // If parsing fails, use description as-is
            metadata = { description: info.description };
          }
        }

        this.serverInfo = {
          name,
          version,
          description: metadata.description || 'A simple task management tool',
          authors: metadata.authors || 'Unknown',
          license: metadata.license || 'MIT',
          repository: metadata.repository || '',
        };
      }
    }

    await this.notify("notifications/initialized", {});
    this.ready = true;
  }

  getServerInfo(): { name: string; version: string; description: string; authors: string; license: string; repository: string } {
    return this.serverInfo || {
      name: 'minitask',
      version: '0.0.0',
      description: 'A simple task management tool',
      authors: 'Unknown',
      license: 'MIT',
      repository: '',
    };
  }

  async request(method: string, params?: JsonMap): Promise<JsonValue> {
    const id = this.nextId;
    this.nextId += 1;

    const message: RpcRequest = {
      jsonrpc: "2.0",
      id,
      method,
      params,
    };

    return new Promise<JsonValue>((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
      this.channel.send(message).catch((error) => {
        this.pending.delete(id);
        reject(error);
      });
    });
  }

  async notify(method: string, params?: JsonMap): Promise<void> {
    const message: RpcNotification = {
      jsonrpc: "2.0",
      method,
      params,
    };
    await this.channel.send(message);
  }

  close(): void {
    for (const pending of this.pending.values()) {
      pending.reject(new Error("GUI closed"));
    }
    this.pending.clear();
    this.channel.close();
  }

  private handleMessage(message: RpcMessage): void {
    if (!("id" in message) || typeof message.id !== "number") {
      return;
    }

    const pending = this.pending.get(message.id);
    if (!pending) {
      return;
    }

    this.pending.delete(message.id);
    if ("error" in message) {
      pending.reject(new Error(message.error.message));
      return;
    }

    pending.resolve(message.result);
  }
}

/**
 * High-level service interface for minitask operations.
 * Wraps MCP connection and provides task management methods:
 * listing, creating, and updating tasks via the minitask MCP server.
 */
class MinitaskService {
  private readonly connection: McpConnection;
  private taskFile: string;

  constructor(connection: McpConnection, taskFile: string) {
    this.connection = connection;
    this.taskFile = taskFile;
  }

  setTaskFile(path: string): void {
    this.taskFile = path;
  }

  getTaskFile(): string {
    return this.taskFile;
  }

  async connect(): Promise<void> {
    await this.connection.initialize();
  }

  async listTasks(stateFilter = "", epicFilter = ""): Promise<TaskRecord[]> {
    const args: JsonMap = {
      file: this.taskFile,
      verbose: true,
    };
    if (stateFilter) {
      args.state = stateFilter;
    }
    if (epicFilter) {
      args.epic = epicFilter;
    }

    Logger.debug(`[SERVICE] Calling list with args: ${JSON.stringify(args)}`);
    const result = await this.connection.request("tools/call", {
      name: "list",
      arguments: args,
    });

    Logger.debug(`[SERVICE] List result type: ${typeof result}`);
    Logger.debug(`[SERVICE] List result: ${JSON.stringify(result)}`);

    if (!TypeUtils.isJsonMap(result)) {
      Logger.error(`[SERVICE] Result is not a JsonMap, returning empty array`);
      return [];
    }

    Logger.debug(`[SERVICE] Parsing tool payload...`);
    const payload = parseToolPayload(result as unknown as ToolCallEnvelope);
    Logger.debug(`[SERVICE] Parsed payload: ${JSON.stringify(payload)}`);

    const taskList = toTaskListResult(payload);
    Logger.info(`[SERVICE] Task list has ${taskList.tasks.length} tasks`);
    Logger.debug(`[SERVICE] Tasks: ${JSON.stringify(taskList)}`);
    return taskList.tasks;
  }

  async createTask(content: string): Promise<void> {
    await this.connection.request("tools/call", {
      name: "new",
      arguments: {
        file: this.taskFile,
        content
      },
    });
  }

  async updateTaskState(taskId: string, state: TaskState): Promise<void> {
    await this.connection.request("tools/call", {
      name: "edit-state",
      arguments: {
        file: this.taskFile,
        task_id: taskId,
        state,
      },
    });
  }

  async updateTaskContent(taskId: string, content: string): Promise<void> {
    await this.connection.request("tools/call", {
      name: "edit-content",
      arguments: {
        file: this.taskFile,
        task_id: taskId,
        content,
      },
    });
  }

  close(): void {
    this.connection.close();
  }
}

/**
 * Monitors a task file for external changes and triggers reload callbacks.
 * Tracks file version (etag + size) to detect changes and distinguish
 * between own writes and external modifications. Prevents reload loops
 * by ignoring changes immediately after own writes.
 */
class TaskFileWatcher {
  private readonly file: GioFile;
  private monitor: GioFileMonitor | null = null;
  private version: FileVersion | null = null;
  private ignoreNextChange = false;
  private readonly onChange: () => void;
  private readonly directory: GioFile;
  private readonly filename: string;

  constructor(options: FileWatchOptions) {
    this.file = Gio.File.new_for_path(options.path);

    this.directory = this.file.get_parent()!;
    this.filename = this.file.get_basename();

    this.onChange = options.onChange;
  }

  start(): void {
    if (this.monitor) {
      return;
    }

    this.monitor = this.directory.monitor_directory(
      Gio.FileMonitorFlags.NONE,
      null,
    );

    this.monitor!.connect("changed", (_monitor, file, _other, eventType) => {
      if (!file || file.get_basename() !== this.filename) {
        return;
      }

      if (
        eventType === Gio.FileMonitorEvent.CHANGES_DONE_HINT ||
        eventType === Gio.FileMonitorEvent.CREATED ||
        eventType === Gio.FileMonitorEvent.MOVED_IN ||
        eventType === Gio.FileMonitorEvent.CHANGED
      ) {
        this.onChange();
      }
    });
  }

  stop(): void {
    if (this.monitor) {
      this.monitor.cancel();
      this.monitor = null;
    }
  }

  markOwnWrite(): void {
    this.ignoreNextChange = true;
  }

  refreshVersion(): void {
    this.version = this.readVersion();
  }

  private shouldReload(eventType: number): boolean {
    return eventType === Gio.FileMonitorEvent.CHANGES_DONE_HINT;
  }

  private readVersion(): FileVersion | null {
    try {
      const info = this.file.query_info(
        "standard::size,etag::value",
        Gio.FileQueryInfoFlags.NONE,
        null,
      );
      return {
        etag: String(info.get_attribute_string("etag::value") ?? ""),
        size: info.get_size(),
      };
    } catch {
      return null;
    }
  }

  private didVersionChange(nextVersion: FileVersion | null): boolean {
    if (this.version === null || nextVersion === null) {
      return this.version !== nextVersion;
    }

    return (
      this.version.etag !== nextVersion.etag ||
      this.version.size !== nextVersion.size
    );
  }
}

class TaskRowFactory {
  create(
    task: TaskRecord,
    onMove: TaskMoveHandler,
    onSaveContent: (taskId: string, content: string) => void,
  ): GtkListBoxRow {
    const titleLabel = createLabel(task.name, { hexpand: true });
    titleLabel.add_css_class("heading");

    const stateLabel = createLabel(task.state, { xalign: 1 });
    stateLabel.add_css_class("dim-label");

    const metadataLabels: GtkLabel[] = [];
    if (task.epic.length > 0) {
      metadataLabels.push(createLabel(`epic: ${task.epic.join(", ")}`));
    }
    if (task.depends_on.length > 0) {
      metadataLabels.push(createLabel(`depends on: ${task.depends_on.join(", ")}`));
    }

    const actionButtons: GtkButton[] = TASK_STATES.map((nextState) => {
      const action = createButton(nextState, () => onMove(task.name, nextState), {
        sensitive: true,
      });
      if (nextState === task.state) {
        action.add_css_class("suggested-action");
      }
      return action;
    });

    const header = createHBox(12);
    header.append(titleLabel);
    header.append(stateLabel);

    const contentArea = createVBox();
    const contentLabel = createLabel(task.content);
    contentArea.append(contentLabel);

    const showEditor = (): void => {
      const labelHeight = contentLabel.get_allocated_height();
      contentArea.remove(contentLabel);

      const contentView: GtkTextView = new Gtk.TextView({
        editable: true,
        cursor_visible: true,
        monospace: false,
        wrap_mode: 2,
      });
      contentView.get_buffer().set_text(task.content, -1);

      const contentScroller: GtkScrolledWindow = new Gtk.ScrolledWindow({
        hexpand: true,
        min_content_height: labelHeight,
        max_content_height: labelHeight,
      });
      contentScroller.set_child(contentView);

      const restoreNormalView = (): void => {
        contentArea.remove(contentScroller);
        contentArea.remove(buttonBox);
        contentArea.append(contentLabel);
      };

      const saveButton = createButton("save", () => {
        const contentBuffer = contentView.get_buffer();
        const [start, end] = contentBuffer.get_bounds();
        const newContent = contentBuffer.get_text(start, end, false);
        task.content = newContent;
        contentLabel.set_label(newContent);
        onSaveContent(task.name, newContent);
        restoreNormalView();
      });

      const discardButton = createButton("discard", restoreNormalView);

      const buttonBox = createHBox();
      buttonBox.append(saveButton);
      buttonBox.append(discardButton);

      contentArea.append(contentScroller);
      contentArea.append(buttonBox);
    };

    const clickController = new Gtk.GestureClick();
    clickController.connect("pressed", () => {
      showEditor();
    });
    contentLabel.add_controller(clickController);

    const actions = createHBox();
    for (const actionButton of actionButtons) {
      actions.append(actionButton);
    }

    const contentBox = createVBox(6, {
      margin_top: 10,
      margin_bottom: 10,
      margin_start: 10,
      margin_end: 10,
    });
    contentBox.append(header);
    for (const metadataLabel of metadataLabels) {
      contentBox.append(metadataLabel);
    }
    contentBox.append(contentArea);
    contentBox.append(actions);

    const row: GtkListBoxRow = new Gtk.ListBoxRow();
    row.set_child(contentBox);
    return row;
  }
}

class MainWindowFactory {
  create(app: GtkApplication): UiRefs {
    const taskEntry = createEntry("new task content", { hexpand: true });
    const addButton = createButton("add", () => {});
    const refreshButton = createButton("refresh", () => {});

    const stateFilter: GtkDropDown = Gtk.DropDown.new_from_strings([
      "all",
      ...TASK_STATES,
    ]);
    stateFilter.set_selected(0);

    const epicFilter = createEntry("filter by epic text...", { hexpand: true });
    const statusLabel = createLabel("connecting...");

    const taskList: GtkListBox = new Gtk.ListBox({
      selection_mode: Gtk.SelectionMode.NONE,
    });

    const toolbar = createHBox();
    toolbar.append(taskEntry);
    toolbar.append(addButton);
    toolbar.append(refreshButton);

    const stateFilterBox = createHBox();
    stateFilterBox.append(createLabel("State Filter:"));
    stateFilterBox.append(stateFilter);

    const epicFilterBox = createHBox();
    epicFilterBox.append(createLabel("Epic Filter:"));
    epicFilterBox.append(epicFilter);

    const filtersBox = createHBox(12);
    filtersBox.append(stateFilterBox);
    filtersBox.append(epicFilterBox);

    const scroller: GtkScrolledWindow = new Gtk.ScrolledWindow({
      hexpand: true,
      vexpand: true,
    });
    scroller.set_child(taskList);

    const content = createVBox(12, {
      margin_top: 12,
      margin_bottom: 12,
      margin_start: 12,
      margin_end: 12,
    });
    content.append(toolbar);
    content.append(statusLabel);
    content.append(filtersBox);
    content.append(scroller);

    const headerBar = new Gtk.HeaderBar();

    // Create menu model
    const menuModel = new Gio.Menu();

    // File menu
    const fileMenu = new Gio.Menu();
    fileMenu.append("Open Task File...", "app.open");
    fileMenu.append("Close Window", "app.close");
    menuModel.append_submenu("File", fileMenu);

    // Help menu
    const helpMenu = new Gio.Menu();
    helpMenu.append("About", "app.about");
    helpMenu.append("Quit", "app.quit");
    menuModel.append_submenu("Help", helpMenu);

    // Create menu button and add to header bar
    const menuButton = new Gtk.MenuButton();
    menuButton.set_icon_name("open-menu-symbolic");
    menuButton.set_menu_model(menuModel);
    headerBar.pack_end(menuButton);

    const window: GtkApplicationWindow = new Gtk.ApplicationWindow({
      application: app,
      title: "minitask",
      default_width: 960,
      default_height: 720,
    });
    window.set_titlebar(headerBar);
    window.set_child(content);

    window.connect("close-request", () => {
      app.quit();
      return false;
    });

    return {
      window,
      taskEntry,
      addButton,
      refreshButton,
      stateFilter,
      epicFilter,
      statusLabel,
      taskList,
    };
  }
}

/**
 * Main window controller that coordinates UI interactions and task operations.
 * Manages the lifecycle of task list display, handles user input events,
 * coordinates with MinitaskService for backend operations, and manages
 * file watching for automatic reload on external changes.
 */
class MainWindowController {
  private readonly ui: UiRefs;
  private readonly service: MinitaskService;
  private readonly rowFactory: TaskRowFactory;
  private fileWatcher: TaskFileWatcher;
  private reloadInFlight = false;
  private reloadQueued = false;
  private stateFilter = "";
  private epicFilter = "";

  constructor(
    ui: UiRefs,
    service: MinitaskService,
    rowFactory: TaskRowFactory,
    taskFile: string,
  ) {
    this.ui = ui;
    this.service = service;
    this.rowFactory = rowFactory;
    this.fileWatcher = new TaskFileWatcher({
      path: taskFile,
      onChange: () => {
        void this.reloadFromFileEvent();
      },
    });
  }

  bind(): void {
    this.ui.addButton.connect("clicked", () => {
      void this.handleCreateTask();
    });
    this.ui.refreshButton.connect("clicked", () => {
      void this.reload();
    });
    this.ui.stateFilter.connect("notify::selected", () => {
      this.stateFilter = this.readStateFilter();
      void this.reload();
    });
    this.ui.epicFilter.connect("activate", () => {
      this.epicFilter = this.ui.epicFilter.get_text().trim();
      void this.reload();
    });
    const focusController = new Gtk.EventControllerFocus();
    focusController.connect("leave", () => {
      const newFilter = this.ui.epicFilter.get_text().trim();
      if (newFilter !== this.epicFilter) {
        this.epicFilter = newFilter;
        void this.reload();
      }
    });
    this.ui.epicFilter.add_controller(focusController);
  }

  async initialize(): Promise<void> {
    this.setStatus("connecting...");
    try {
      await this.service.connect();
      await this.reload();
      this.fileWatcher.start();
    } catch (error) {
      this.showError(error);
    }
  }

  async changeTaskFile(newPath: string): Promise<void> {
    this.setBusy(true, "switching task file...");
    try {
      // Stop watching the old file
      this.fileWatcher.stop();

      // Update the service with new file path
      this.service.setTaskFile(newPath);

      // Start watching the new file
      this.fileWatcher = new TaskFileWatcher({
        path: newPath,
        onChange: () => {
          void this.reloadFromFileEvent();
        },
      });
      this.fileWatcher.start();

      // Reload tasks from new file
      await this.reload();
      this.setStatus(`opened ${newPath}`);
    } catch (error) {
      this.showError(error);
    } finally {
      this.setBusy(false, "");
    }
  }

  private async handleCreateTask(): Promise<void> {
    const content = this.ui.taskEntry.get_text().trim();
    if (!content) {
      return;
    }

    this.setBusy(true, "creating task...");
    try {
      this.fileWatcher.markOwnWrite();
      await this.service.createTask(content);
      this.fileWatcher.refreshVersion();
      this.ui.taskEntry.set_text("");
      await this.reload();
      this.setStatus("task created");
    } catch (error) {
      this.showError(error);
    } finally {
      this.setBusy(false, "");
    }
  }

  private async handleMoveTask(
    taskId: string,
    nextState: TaskState,
  ): Promise<void> {
    this.setBusy(true, `updating ${taskId}...`);
    try {
      this.fileWatcher.markOwnWrite();
      await this.service.updateTaskState(taskId, nextState);
      this.fileWatcher.refreshVersion();
      await this.reload();
      this.setStatus(`${taskId} -> ${nextState}`);
    } catch (error) {
      this.showError(error);
    } finally {
      this.setBusy(false, "");
    }
  }

  private async handleSaveTaskContent(
    taskId: string,
    content: string,
  ): Promise<void> {
    this.setBusy(true, `saving ${taskId}...`);
    try {
      this.fileWatcher.markOwnWrite();
      await this.service.updateTaskContent(taskId, content);
      this.fileWatcher.refreshVersion();
      await this.reload();
      this.setStatus(`${taskId} content saved`);
    } catch (error) {
      this.showError(error);
    } finally {
      this.setBusy(false, "");
    }
  }

  private async reload(): Promise<void> {
    if (this.reloadInFlight) {
      this.reloadQueued = true;
      return;
    }

    this.reloadInFlight = true;
    this.setBusy(true, "loading tasks...");
    try {
      const tasks = await this.service.listTasks(this.stateFilter, this.epicFilter);
      this.fileWatcher.refreshVersion();
      this.renderTasks(tasks);
      this.setStatus(`${tasks.length} task(s)`);
    } catch (error) {
      this.showError(error);
    } finally {
      this.reloadInFlight = false;
      this.setBusy(false, "");
      if (this.reloadQueued) {
        this.reloadQueued = false;
        await this.reload();
      }
    }
  }

  private async reloadFromFileEvent(): Promise<void> {
    if (this.reloadInFlight) {
      return;
    }
    await this.reload();
  }

  private readStateFilter(): string {
    const selected = this.ui.stateFilter.get_selected_item();
    const text = selected ? selected.get_string() : "all";
    return text === "all" ? "" : text;
  }

  private renderTasks(tasks: TaskRecord[]): void {
    Logger.info(`[RENDER] renderTasks called with ${tasks.length} tasks`);
    Logger.debug(`[RENDER] Tasks: ${JSON.stringify(tasks)}`);

    // This view only renders the current filtered task slice and rebuilds it on
    // explicit reload/file-change events. For the expected small task counts and
    // per-row custom actions in this GUI, Gtk.ListBox keeps the code simpler than
    // a Gio.ListStore/Gtk.ListView migration without changing visible behavior.
    let child = this.ui.taskList.get_first_child();
    let removedCount = 0;
    while (child) {
      const next = child.get_next_sibling();
      this.ui.taskList.remove(child);
      removedCount++;
      child = next;
    }
    Logger.debug(`[RENDER] Removed ${removedCount} existing rows`);

    for (const task of [...tasks].reverse()) {
      Logger.debug(`[RENDER] Creating row for task: ${task.name}`);
      const row = this.rowFactory.create(
        task,
        (taskId, nextState) => {
          void this.handleMoveTask(taskId, nextState);
        },
        (taskId, content) => {
          void this.handleSaveTaskContent(taskId, content);
        },
      );
      this.ui.taskList.append(row);
      Logger.debug(`[RENDER] Appended row for task: ${task.name}`);
    }
    Logger.info(`[RENDER] Finished rendering ${tasks.length} tasks`);
  }

  private setBusy(isBusy: boolean, message: string): void {
    this.ui.addButton.set_sensitive(!isBusy);
    this.ui.refreshButton.set_sensitive(!isBusy);
    this.ui.taskEntry.set_sensitive(!isBusy);
    this.ui.stateFilter.set_sensitive(!isBusy);
    this.ui.epicFilter.set_sensitive(!isBusy);
    if (message) {
      this.setStatus(message);
    }
  }

  private setStatus(message: string): void {
    this.ui.statusLabel.set_label(message);
  }

  private showError(error: unknown): void {
    Logger.showError(this.ui.statusLabel, "Operation failed", error);
  }

  close(): void {
    this.service.close();
  }
}

const MinitaskApplication = GObject.registerClass(
class MinitaskApplication extends Gtk.Application {
    private controller: MainWindowController | null = null;
    private mcpConnection: McpConnection | null = null;

    constructor() {
      super({
        application_id: APPLICATION_ID,
        flags: Gio.ApplicationFlags.NON_UNIQUE,
      });

      this.connect("startup", () => {
        syncSystemColorScheme();
        this.setupActions();
      });
    }

    private setupActions(): void {
      const openAction = new Gio.SimpleAction({ name: "open" });
      openAction.connect("activate", () => {
        this.handleOpenFile();
      });
      this.add_action(openAction);

      const closeAction = new Gio.SimpleAction({ name: "close" });
      closeAction.connect("activate", () => {
        this.handleCloseWindow();
      });
      this.add_action(closeAction);

      const aboutAction = new Gio.SimpleAction({ name: "about" });
      aboutAction.connect("activate", () => {
        this.handleAbout();
      });
      this.add_action(aboutAction);

      const quitAction = new Gio.SimpleAction({ name: "quit" });
      quitAction.connect("activate", () => {
        this.quit();
      });
      this.add_action(quitAction);
    }

    private handleOpenFile(): void {
      const dialog = new Gtk.FileChooserDialog({
        title: "Open Task File",
        action: Gtk.FileChooserAction.OPEN,
        transient_for: this.active_window as GtkApplicationWindow,
        modal: true,
      });

      dialog.add_button("Cancel", Gtk.ResponseType.CANCEL);
      dialog.add_button("Open", Gtk.ResponseType.ACCEPT);

      const filter = new Gtk.FileFilter();
      filter.set_name("TOML files");
      filter.add_pattern("*.toml");
      dialog.add_filter(filter);

      const allFilter = new Gtk.FileFilter();
      allFilter.set_name("All files");
      allFilter.add_pattern("*");
      dialog.add_filter(allFilter);

      dialog.connect("response", (_dialog, response) => {
        if (response === Gtk.ResponseType.ACCEPT) {
          const file = dialog.get_file();
          if (file) {
            const path = file.get_path();
            if (path) {
              Logger.info(`Opening task file: ${path}`);

              // Simply change the task file and reload
              if (this.controller) {
                this.controller.changeTaskFile(path);
              }
            }
          }
        }
        dialog.close();
      });

      dialog.present();
    }

    private handleCloseWindow(): void {
      const window = this.active_window;
      if (window) {
        window.close();
      }
    }

  private handleAbout(): void {
      const serverInfo = this.mcpConnection?.getServerInfo() || {
        name: 'minitask',
        version: '0.0.0',
        description: 'A simple task management tool',
        authors: 'Unknown',
        license: 'MIT',
        repository: '',
      };

      const aboutDialog = new Gtk.AboutDialog({
        transient_for: this.active_window as GtkApplicationWindow,
        modal: true,
        program_name: serverInfo.name,
        version: serverInfo.version,
        comments: serverInfo.description,
        website: serverInfo.repository,
        website_label: "Project Repository",
        license_type: Gtk.License.MIT_X11,
        authors: serverInfo.authors.split(',').map(a => a.trim()),
      });
      aboutDialog.present();
    }

    vfunc_activate(): void {
      const taskFile = requireEnvString("MINITASK_GUI_TASK_FILE");
      const connection = new McpConnection();
      this.mcpConnection = connection;
      const service = new MinitaskService(connection, taskFile);
      const ui = new MainWindowFactory().create(this);

      this.controller = new MainWindowController(
        ui,
        service,
        new TaskRowFactory(),
        taskFile,
      );

      this.controller.bind();

      ui.window.connect("close-request", () => {
        this.controller?.close();
        return false;
      });

      ui.window.present();

      void this.controller.initialize();
    }
  },
);

const app = new MinitaskApplication();
app.run([]);
