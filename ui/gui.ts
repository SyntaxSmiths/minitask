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
declare function logError(error: unknown, message?: string): void;

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
const TASK_STATES = ["todo", "in-progress", "review", "done", "blocked"] as const;

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

function decodeBytes(bytes: Uint8Array): string {
  return imports.byteArray.toString(bytes);
}

function encodeLine(message: JsonValue): Uint8Array {
  return ByteArray.fromString(`${JSON.stringify(message)}\n`);
}

function isJsonMap(value: JsonValue | undefined): value is JsonMap {
  return value !== null && value !== undefined && !Array.isArray(value) && typeof value === "object";
}

function asArray(value: JsonValue | undefined): JsonValue[] {
  return Array.isArray(value) ? value : [];
}

function asString(value: JsonValue | undefined, fallback = ""): string {
  return typeof value === "string" ? value : fallback;
}

function asStringArray(value: JsonValue | undefined): string[] {
  return asArray(value).filter((item): item is string => typeof item === "string");
}

function isRpcFailure(value: JsonMap): boolean {
  return typeof value.id === "number" &&
    isJsonMap(value.error) &&
    typeof value.error.code === "number" &&
    typeof value.error.message === "string";
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

function parseToolPayload(result: ToolCallEnvelope): JsonValue {
  if (result.structuredContent !== undefined) {
    return result.structuredContent;
  }

  const text = (result.content ?? [])
    .filter((item) => item.type === "text" && typeof item.text === "string")
    .map((item) => item.text ?? "")
    .join("\n")
    .trim();

  if (!text) {
    return null;
  }

  try {
    return JSON.parse(text) as JsonValue;
  } catch {
    return text;
  }
}

function toTaskRecord(value: JsonValue): TaskRecord {
  const object = isJsonMap(value) ? value : {};
  return {
    name: asString(object.name),
    state: asString(object.state),
    content: asString(object.content),
    depends_on: asStringArray(object.depends_on),
    epic: asStringArray(object.epic),
  };
}

function toTaskListResult(value: JsonValue): TaskListResult {
  if (Array.isArray(value)) {
    return { tasks: value.map(toTaskRecord) };
  }

  const object = isJsonMap(value) ? value : {};
  const tasks = Array.isArray(object.tasks) ? object.tasks.map(toTaskRecord) : [];
  return { tasks };
}


class JsonLineChannel {
  private readonly input: GioInputStream;
  private readonly output: GioOutputStream;
  private readonly cancellable: GioCancellable;
  private buffer = "";
  private closed = false;
  private started = false;
  private readonly onMessage: (message: RpcMessage) => void;

  constructor(input: GioInputStream, output: GioOutputStream, onMessage: (message: RpcMessage) => void) {
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
        logError(error, "closing minitask output");
      }
    }
  }

  private readStdout(): void {
    const pump = () => {
      if (this.closed) {
        return;
      }

      this.input.read_bytes_async(4096, GLib.PRIORITY_DEFAULT, this.cancellable, (_stream: unknown, result: unknown) => {
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
            logError(error, "minitask stdout");
          }
        }
      });
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
      if (!isJsonMap(parsed) || !isRpcMessage(parsed)) {
        logError(new Error(`Ignoring unexpected MCP message: ${line}`), "minitask protocol");
        return null;
      }

      if (isRpcNotification(parsed)) {
        return null;
      }

      return parsed as unknown as RpcMessage;
    } catch (error) {
      logError(error, `Failed to parse MCP message: ${line}`);
      return null;
    }
  }

  private isExpectedCloseError(error: unknown): boolean {
    if (!(error instanceof Error)) {
      return false;
    }

    return error.message.includes("Operation was cancelled")
      || error.message.includes("Stream is already closed");
  }
}

class McpConnection {
  private readonly channel: JsonLineChannel;
  private readonly pending = new Map<RpcId, PendingRequest>();
  private nextId = 1;
  private ready = false;

  constructor() {
    const socketFd = requireEnvString("MINITASK_GUI_SOCKET_FD");
    const fd = Number.parseInt(socketFd, 10);
    if (!Number.isInteger(fd) || fd < 0) {
      throw new Error(`MINITASK_GUI_SOCKET_FD must be a non-negative integer, got: ${socketFd}`);
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
    await this.request("initialize", {
      protocolVersion: PROTOCOL_VERSION,
      capabilities: {},
      clientInfo: {
        name: "gjs-minitask-gui",
        version: "0.1.0",
      },
    });
    await this.notify("notifications/initialized", {});
    this.ready = true;
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

class MinitaskService {
  private readonly connection: McpConnection;

  constructor(connection: McpConnection) {
    this.connection = connection;
  }

  async connect(): Promise<void> {
    await this.connection.initialize();
  }

  async listTasks(stateFilter = ""): Promise<TaskRecord[]> {
    const args: JsonMap = {
      verbose: true,
    };
    if (stateFilter) {
      args.state = stateFilter;
    }

    const result = await this.connection.request("tools/call", {
      name: "list",
      arguments: args,
    });

    if (!isJsonMap(result)) {
      return [];
    }

    const payload = parseToolPayload(result as unknown as ToolCallEnvelope);
    return toTaskListResult(payload).tasks;
  }

  async createTask(content: string): Promise<void> {
    await this.connection.request("tools/call", {
      name: "new",
      arguments: { content },
    });
  }

  async updateTaskState(taskId: string, state: TaskState): Promise<void> {
    await this.connection.request("tools/call", {
      name: "edit-state",
      arguments: {
        task_id: taskId,
        state,
      },
    });
  }

  async updateTaskContent(taskId: string, content: string): Promise<void> {
    await this.connection.request("tools/call", {
      name: "edit-content",
      arguments: {
        task_id: taskId,
        content,
      },
    });
  }

  close(): void {
    this.connection.close();
  }
}

class TaskFileWatcher {
  private readonly file: GioFile;
  private monitor: GioFileMonitor | null = null;
  private version: FileVersion | null = null;
  private ignoreNextChange = false;
  private readonly onChange: () => void;

  constructor(options: FileWatchOptions) {
    this.file = Gio.File.new_for_path(options.path);
    this.onChange = options.onChange;
    this.version = this.readVersion();
  }

  start(): void {
    if (this.monitor) {
      return;
    }

    this.monitor = this.file.monitor_file(Gio.FileMonitorFlags.NONE, null);
    this.monitor.connect("changed", (_monitor: unknown, _file: unknown, _otherFile: unknown, eventType: number) => {
      if (!this.shouldReload(eventType)) {
        return;
      }

      const nextVersion = this.readVersion();
      if (!this.didVersionChange(nextVersion)) {
        return;
      }

      this.version = nextVersion;
      if (this.ignoreNextChange) {
        this.ignoreNextChange = false;
        return;
      }
      this.onChange();
    });
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

    return this.version.etag !== nextVersion.etag ||
      this.version.size !== nextVersion.size;
  }
}

class TaskRowFactory {
  create(
    task: TaskRecord,
    onMove: TaskMoveHandler,
    onSaveContent: (taskId: string, content: string) => void,
  ): GtkListBoxRow {
    const titleLabel: GtkLabel = new Gtk.Label({
      label: task.name,
      xalign: 0,
      hexpand: true,
    });
    titleLabel.add_css_class("heading");

    const stateLabel: GtkLabel = new Gtk.Label({
      label: task.state,
      xalign: 1,
    });
    stateLabel.add_css_class("dim-label");

    const metadataLabels: GtkLabel[] = [];
    if (task.epic.length > 0) {
      metadataLabels.push(new Gtk.Label({ label: `epic: ${task.epic.join(", ")}`, xalign: 0, wrap: true }));
    }
    if (task.depends_on.length > 0) {
      metadataLabels.push(new Gtk.Label({ label: `depends on: ${task.depends_on.join(", ")}`, xalign: 0, wrap: true }));
    }

    const actionButtons: GtkButton[] = TASK_STATES.map((nextState) => {
      const action: GtkButton = new Gtk.Button({
        label: nextState,
        sensitive: true,
      });
      if (nextState === task.state) {
        action.add_css_class("suggested-action");
      }
      action.connect("clicked", () => {
        onMove(task.name, nextState);
      });
      return action;
    });

    const header: GtkBox = new Gtk.Box({
      orientation: Gtk.Orientation.HORIZONTAL,
      spacing: 12,
    });
    header.append(titleLabel);
    header.append(stateLabel);

    const contentArea: GtkBox = new Gtk.Box({
      orientation: Gtk.Orientation.VERTICAL,
      spacing: 6,
    });
    const contentLabel: GtkLabel = new Gtk.Label({
      label: task.content,
      xalign: 0,
      wrap: true,
    });
    contentArea.append(contentLabel);

    const showEditor = (): void => {
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
        min_content_height: 96,
        max_content_height: 320,
      });
      contentScroller.set_child(contentView);

      const saveButton: GtkButton = new Gtk.Button({
        label: "save",
      });
      saveButton.connect("clicked", () => {
        const contentBuffer = contentView.get_buffer();
        const [start, end] = contentBuffer.get_bounds();
        onSaveContent(task.name, contentBuffer.get_text(start, end, false));
      });

      contentArea.append(contentScroller);
      contentArea.append(saveButton);
    };

    const clickController = new Gtk.GestureClick();
    clickController.connect("pressed", () => {
      showEditor();
    });
    contentLabel.add_controller(clickController);

    const actions: GtkBox = new Gtk.Box({
      orientation: Gtk.Orientation.HORIZONTAL,
      spacing: 6,
    });
    for (const actionButton of actionButtons) {
      actions.append(actionButton);
    }

    const contentBox: GtkBox = new Gtk.Box({
      orientation: Gtk.Orientation.VERTICAL,
      spacing: 6,
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
    const taskEntry: GtkEntry = new Gtk.Entry({
      hexpand: true,
      placeholder_text: "new task content",
    });
    const addButton: GtkButton = new Gtk.Button({ label: "add" });
    const refreshButton: GtkButton = new Gtk.Button({ label: "refresh" });
    const stateFilter: GtkDropDown = Gtk.DropDown.new_from_strings(["all", ...TASK_STATES]);
    stateFilter.set_selected(0);
    const statusLabel: GtkLabel = new Gtk.Label({
      label: "connecting...",
      xalign: 0,
    });
    const taskList: GtkListBox = new Gtk.ListBox({
      selection_mode: Gtk.SelectionMode.NONE,
    });

    const toolbar: GtkBox = new Gtk.Box({
      orientation: Gtk.Orientation.HORIZONTAL,
      spacing: 6,
    });
    toolbar.append(taskEntry);
    toolbar.append(stateFilter);
    toolbar.append(addButton);
    toolbar.append(refreshButton);

    const scroller: GtkScrolledWindow = new Gtk.ScrolledWindow({
      hexpand: true,
      vexpand: true,
    });
    scroller.set_child(taskList);

    const content: GtkBox = new Gtk.Box({
      orientation: Gtk.Orientation.VERTICAL,
      spacing: 12,
      margin_top: 12,
      margin_bottom: 12,
      margin_start: 12,
      margin_end: 12,
    });
    content.append(toolbar);
    content.append(statusLabel);
    content.append(scroller);

    const window: GtkApplicationWindow = new Gtk.ApplicationWindow({
      application: app,
      title: "minitask",
      default_width: 960,
      default_height: 720,
    });
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
      statusLabel,
      taskList,
    };
  }
}

class MainWindowController {
  private readonly ui: UiRefs;
  private readonly service: MinitaskService;
  private readonly rowFactory: TaskRowFactory;
  private readonly fileWatcher: TaskFileWatcher;
  private reloadInFlight = false;
  private reloadQueued = false;
  private stateFilter = "";

  constructor(ui: UiRefs, service: MinitaskService, rowFactory: TaskRowFactory, taskFile: string) {
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

  private async handleMoveTask(taskId: string, nextState: TaskState): Promise<void> {
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

  private async handleSaveTaskContent(taskId: string, content: string): Promise<void> {
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
      const tasks = await this.service.listTasks(this.stateFilter);
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
    // This view only renders the current filtered task slice and rebuilds it on
    // explicit reload/file-change events. For the expected small task counts and
    // per-row custom actions in this GUI, Gtk.ListBox keeps the code simpler than
    // a Gio.ListStore/Gtk.ListView migration without changing visible behavior.
    let child = this.ui.taskList.get_first_child();
    while (child) {
      const next = child.get_next_sibling();
      this.ui.taskList.remove(child);
      child = next;
    }

    for (const task of [...tasks].reverse()) {
      this.ui.taskList.append(this.rowFactory.create(
        task,
        (taskId, nextState) => {
          void this.handleMoveTask(taskId, nextState);
        },
        (taskId, content) => {
          void this.handleSaveTaskContent(taskId, content);
        },
      ));
    }
  }

  private setBusy(isBusy: boolean, message: string): void {
    this.ui.addButton.set_sensitive(!isBusy);
    this.ui.refreshButton.set_sensitive(!isBusy);
    this.ui.taskEntry.set_sensitive(!isBusy);
    this.ui.stateFilter.set_sensitive(!isBusy);
    if (message) {
      this.setStatus(message);
    }
  }

  private setStatus(message: string): void {
    this.ui.statusLabel.set_label(message);
  }

  private showError(error: unknown): void {
    const message = error instanceof Error ? error.message : String(error);
    this.setStatus(`error: ${message}`);
  }

  close(): void {
    this.service.close();
  }
}

const MinitaskApplication = GObject.registerClass(
  class MinitaskApplication extends Gtk.Application {
    constructor() {
      super({
        application_id: APPLICATION_ID,
        flags: Gio.ApplicationFlags.NON_UNIQUE,
      });
      this.connect("startup", () => {
        syncSystemColorScheme();
      });
    }

    vfunc_activate(): void {
      const taskFile = requireEnvString("MINITASK_GUI_TASK_FILE");
      const connection = new McpConnection();
      const service = new MinitaskService(connection);
      const ui = new MainWindowFactory().create(this);
      const controller = new MainWindowController(ui, service, new TaskRowFactory(), taskFile);

      controller.bind();
      ui.window.connect("close-request", () => {
        controller.close();
        return false;
      });
      ui.window.present();
      void controller.initialize();
    }
  },
);

const app = new MinitaskApplication();
app.run([]);
