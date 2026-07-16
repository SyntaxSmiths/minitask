// Base interfaces for common patterns
export interface Connectable {
  connect(signal: string, callback: (...args: any[]) => unknown): number;
}

export interface Constructable<T> {
  new (props?: Record<string, unknown>): T;
}

export interface SimpleConstructable<T> {
  new (): T;
}

// GTK Widget hierarchy
export interface GtkWindowLike extends Connectable {
  set_child(child: unknown): void;
  present(): void;
  close(): void;
}

export interface GtkWidgetLike extends Connectable {
  add_css_class(name: string): void;
  add_controller(controller: unknown): void;
  set_sensitive(value: boolean): void;
}

export interface GtkStringObjectLike {
  get_string(): string;
}

export interface GtkApplication extends Connectable {
  active_window: GtkApplicationWindow | null;
  quit(): void;
  run(args: string[]): number;
  set_menubar(menubar: GioMenu | null): void;
  add_action(action: GioSimpleAction): void;
  get_windows(): GtkApplicationWindow[];
  activate(): void;
}

export interface GtkApplicationWindow extends GtkWindowLike {
  set_titlebar(titlebar: GtkHeaderBar | null): void;
}

export interface GtkBox extends GtkWidgetLike {
  append(child: unknown): void;
  remove(child: unknown): void;
}

export interface GtkButton extends GtkWidgetLike {}

export interface GtkGestureClick extends Connectable {}

export interface GtkEventControllerFocus extends Connectable {}

export interface GtkCssProvider {
  prefers_color_scheme: number;
}

export interface GtkSettings extends Connectable {
  gtk_application_prefer_dark_theme: boolean;
  gtk_interface_color_scheme: number;
}

export interface GtkStyleContextStatic {
  add_provider_for_display(
    display: GdkDisplay,
    provider: GtkCssProvider,
    priority: number,
  ): void;
}

export interface GtkDropDown extends GtkWidgetLike {
  set_selected(index: number): void;
  get_selected_item(): GtkStringObjectLike | null;
}

export interface GtkDropDownStatic extends Constructable<GtkDropDown> {
  new_from_strings(values: string[]): GtkDropDown;
}

export interface GtkEntry extends GtkWidgetLike {
  get_text(): string;
  set_text(value: string): void;
}

export interface GtkLabel extends GtkWidgetLike {
  set_label(value: string): void;
  get_allocated_height(): any;
}

export interface GtkListBox extends GtkWidgetLike {
  append(child: GtkListBoxRow): void;
  get_first_child(): GtkListBoxRow | null;
  remove(child: GtkListBoxRow): void;
}

export interface GtkListBoxRow extends GtkWidgetLike {
  set_child(child: unknown): void;
  get_next_sibling(): GtkListBoxRow | null;
}

export interface GtkScrolledWindow extends GtkWidgetLike {
  set_child(child: unknown): void;
}

export interface GtkTextIter {}

export interface GtkTextBuffer {
  get_bounds(): [GtkTextIter, GtkTextIter];
  get_text(
    start: GtkTextIter,
    end: GtkTextIter,
    includeHiddenChars: boolean,
  ): string;
  set_text(text: string, length: number): void;
}

export interface GtkTextView extends GtkWidgetLike {
  get_buffer(): GtkTextBuffer;
}

export interface GtkFileChooserDialog extends GtkWindowLike {
  add_button(label: string, response_id: number): void;
  add_filter(filter: GtkFileFilter): void;
  get_file(): GioFile | null;
}

export interface GtkFileFilter {
  set_name(name: string): void;
  add_pattern(pattern: string): void;
}

export interface GtkMessageDialog extends GtkWindowLike {}

export interface GtkAboutDialog extends GtkWindowLike {}

export interface GtkHeaderBar extends GtkWidgetLike {
  pack_end(child: GtkWidgetLike): void;
  pack_start(child: GtkWidgetLike): void;
}

export interface GtkMenuButton extends GtkWidgetLike {
  set_menu_model(model: GioMenu | null): void;
  set_icon_name(icon_name: string): void;
}

export interface GtkPopoverMenuBar extends GtkWidgetLike {}

export interface GtkPopoverMenuBarStatic {
  new_from_model(model: GioMenu | null): GtkPopoverMenuBar;
}

export interface GtkNamespace {
  AboutDialog: Constructable<GtkAboutDialog>;
  Application: Constructable<GtkApplication>;
  ApplicationWindow: Constructable<GtkApplicationWindow>;
  Box: Constructable<GtkBox>;
  Button: Constructable<GtkButton>;
  CssProvider: SimpleConstructable<GtkCssProvider>;
  DropDown: GtkDropDownStatic;
  Entry: Constructable<GtkEntry>;
  EventControllerFocus: SimpleConstructable<GtkEventControllerFocus>;
  FileChooserDialog: Constructable<GtkFileChooserDialog>;
  FileFilter: SimpleConstructable<GtkFileFilter>;
  GestureClick: SimpleConstructable<GtkGestureClick>;
  HeaderBar: Constructable<GtkHeaderBar>;
  Label: Constructable<GtkLabel>;
  ListBox: Constructable<GtkListBox>;
  ListBoxRow: Constructable<GtkListBoxRow>;
  MenuButton: Constructable<GtkMenuButton>;
  MessageDialog: Constructable<GtkMessageDialog>;
  PopoverMenuBar: GtkPopoverMenuBarStatic;
  ScrolledWindow: Constructable<GtkScrolledWindow>;
  Settings: {
    get_default(): GtkSettings | null;
  };
  StyleContext: GtkStyleContextStatic;
  TextView: Constructable<GtkTextView>;
  ButtonsType: {
    OK: number;
  };
  FileChooserAction: {
    OPEN: number;
  };
  License: {
    MIT_X11: number;
  };
  Orientation: {
    HORIZONTAL: number;
    VERTICAL: number;
  };
  InterfaceColorScheme: {
    DEFAULT: number;
    LIGHT: number;
    DARK: number;
    UNSUPPORTED: number;
  };
  ResponseType: {
    ACCEPT: number;
    CANCEL: number;
  };
  STYLE_PROVIDER_PRIORITY_APPLICATION: number;
  SelectionMode: {
    NONE: number;
  };
}

export interface GdkDisplay {}

export interface GdkNamespace {
  Display: {
    get_default(): GdkDisplay | null;
  };
}

export interface AdwStyleManager {
  color_scheme: number;
}

export interface AdwNamespace {
  StyleManager: {
    get_default(): AdwStyleManager;
  };
  ColorScheme: {
    DEFAULT: number;
    FORCE_LIGHT: number;
    PREFER_LIGHT: number;
    PREFER_DARK: number;
    FORCE_DARK: number;
  };
}

export interface GioCancellable {
  cancel(): void;
}

export interface GioBytesLike {
  toArray(): Uint8Array;
}

export interface GioFileInfoLike {
  get_attribute_string(name: string): string | null;
  get_size(): number;
}

export interface GioFileMonitor extends Connectable {
  cancel(): void;
}

export interface GioFile {
  monitor_file(
    flags: number,
    cancellable: GioCancellable | null,
  ): GioFileMonitor;
  query_info(
    attributes: string,
    flags: number,
    cancellable: GioCancellable | null,
  ): GioFileInfoLike;
  get_parent(): any;
  get_basename(): any;
  get_path(): string | null;
  monitor_directory(_first: any, _second: any): any;
}

export interface GioMenu {
  append(label: string, detailed_action: string): void;
  append_submenu(label: string, submenu: GioMenu): void;
  append_section(label: string | null, section: GioMenu): void;
}

export interface GioSimpleAction extends Connectable {}

export interface GioNamespace {
  ApplicationFlags: {
    NON_UNIQUE: number;
  };
  Cancellable: SimpleConstructable<GioCancellable>;
  File: {
    new_for_path(path: string): GioFile;
  };
  Menu: SimpleConstructable<GioMenu>;
  SimpleAction: Constructable<GioSimpleAction>;
  FileMonitorFlags: {
    NONE: number;
  };
  FileMonitorEvent: {
    CHANGED: number;
    CHANGES_DONE_HINT: number;
    DELETED: number;
    CREATED: number;
    ATTRIBUTE_CHANGED: number;
    PRE_UNMOUNT: number;
    UNMOUNTED: number;
    MOVED: number;
    RENAMED: number;
    MOVED_IN: number;
    MOVED_OUT: number;
  };
  FileQueryInfoFlags: {
    NONE: number;
  };
}

export interface GioAsyncOutputStream {
  write_bytes_async(
    bytes: unknown,
    priority: number,
    cancellable: GioCancellable | null,
    callback: (stream: unknown, result: unknown) => void,
  ): void;
  write_bytes_finish(result: unknown): unknown;
  close(cancellable: GioCancellable | null): void;
}

export interface GioAsyncInputStream {
  read_bytes_async(
    size: number,
    priority: number,
    cancellable: GioCancellable | null,
    callback: (stream: unknown, result: unknown) => void,
  ): void;
  read_bytes_finish(result: unknown): GioBytesLike;
}

export interface GioUnixNamespace {
  InputStream: Constructable<GioAsyncInputStream>;
  OutputStream: Constructable<GioAsyncOutputStream>;
}

export interface GLibNamespace {
  getenv(name: string): string | null;
  setenv(name: string, value: string, overwrite: boolean): boolean;
  PRIORITY_DEFAULT: number;
  Bytes: {
    new (data: Uint8Array): unknown;
  };
  timeout_add(
    priority: number,
    interval: number,
    callback: () => boolean,
  ): number;
}

export interface GObjectNamespace {
  registerClass<T>(klass: T): T;
}

export {};