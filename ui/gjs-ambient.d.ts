export interface GtkWindowLike {
  set_child(child: unknown): void;
  present(): void;
  connect(signal: string, callback: (...args: any[]) => unknown): number;
}

export interface GtkWidgetLike {
    add_css_class(name: string): void;
    add_controller(controller: unknown): void;
    set_sensitive(value: boolean): void;
    connect(signal: string, callback: (...args: any[]) => unknown): number;
  }

export interface GtkStringObjectLike {
    get_string(): string;
  }

export interface GtkApplication {
    connect(signal: string, callback: (...args: any[]) => unknown): number;
    quit(): void;
    run(args: string[]): number;
  }

export interface GtkApplicationStatic {
    new (props?: Record<string, unknown>): GtkApplication;
  }

export interface GtkApplicationWindow extends GtkWindowLike {}
export interface GtkApplicationWindowStatic {
    new (props?: Record<string, unknown>): GtkApplicationWindow;
  }

export interface GtkBox extends GtkWidgetLike {
    append(child: unknown): void;
    remove(child: unknown): void;
  }
export interface GtkBoxStatic {
    new (props?: Record<string, unknown>): GtkBox;
  }

export interface GtkButton extends GtkWidgetLike {}
export interface GtkButtonStatic {
    new (props?: Record<string, unknown>): GtkButton;
  }

export interface GtkGestureClick {
    connect(signal: string, callback: (...args: any[]) => unknown): number;
  }
export interface GtkGestureClickStatic {
    new (): GtkGestureClick;
  }

export interface GtkCssProvider {
    prefers_color_scheme: number;
  }
export interface GtkCssProviderStatic {
    new (): GtkCssProvider;
  }

export interface GtkSettings {
    gtk_application_prefer_dark_theme: boolean;
    gtk_interface_color_scheme: number;
    connect(signal: string, callback: (...args: any[]) => unknown): number;
  }
export interface GtkSettingsStatic {
    get_default(): GtkSettings | null;
  }

export interface GtkStyleContextStatic {
    add_provider_for_display(display: GdkDisplay, provider: GtkCssProvider, priority: number): void;
  }

export interface GtkDropDown extends GtkWidgetLike {
    set_selected(index: number): void;
    get_selected_item(): GtkStringObjectLike | null;
  }
export interface GtkDropDownStatic {
    new (props?: Record<string, unknown>): GtkDropDown;
    new_from_strings(values: string[]): GtkDropDown;
  }

export interface GtkEntry extends GtkWidgetLike {
    get_text(): string;
    set_text(value: string): void;
  }
export interface GtkEntryStatic {
    new (props?: Record<string, unknown>): GtkEntry;
  }

export interface GtkLabel extends GtkWidgetLike {
    set_label(value: string): void;
  }
export interface GtkLabelStatic {
    new (props?: Record<string, unknown>): GtkLabel;
  }

export interface GtkListBox extends GtkWidgetLike {
    append(child: GtkListBoxRow): void;
    get_first_child(): GtkListBoxRow | null;
    remove(child: GtkListBoxRow): void;
  }
export interface GtkListBoxStatic {
    new (props?: Record<string, unknown>): GtkListBox;
  }

export interface GtkListBoxRow extends GtkWidgetLike {
    set_child(child: unknown): void;
    get_next_sibling(): GtkListBoxRow | null;
  }
export interface GtkListBoxRowStatic {
    new (props?: Record<string, unknown>): GtkListBoxRow;
  }

export interface GtkScrolledWindow extends GtkWidgetLike {
    set_child(child: unknown): void;
  }
export interface GtkScrolledWindowStatic {
    new (props?: Record<string, unknown>): GtkScrolledWindow;
  }

export interface GtkTextIter {}

export interface GtkTextBuffer {
    get_bounds(): [GtkTextIter, GtkTextIter];
    get_text(start: GtkTextIter, end: GtkTextIter, includeHiddenChars: boolean): string;
    set_text(text: string, length: number): void;
  }

export interface GtkTextView extends GtkWidgetLike {
    get_buffer(): GtkTextBuffer;
  }
export interface GtkTextViewStatic {
    new (props?: Record<string, unknown>): GtkTextView;
  }

export interface GtkNamespace {
    Application: GtkApplicationStatic;
    ApplicationWindow: GtkApplicationWindowStatic;
    Box: GtkBoxStatic;
    Button: GtkButtonStatic;
    CssProvider: GtkCssProviderStatic;
    DropDown: GtkDropDownStatic;
    Entry: GtkEntryStatic;
    GestureClick: GtkGestureClickStatic;
    Label: GtkLabelStatic;
    ListBox: GtkListBoxStatic;
    ListBoxRow: GtkListBoxRowStatic;
    ScrolledWindow: GtkScrolledWindowStatic;
    Settings: GtkSettingsStatic;
    StyleContext: GtkStyleContextStatic;
    TextView: GtkTextViewStatic;
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
    STYLE_PROVIDER_PRIORITY_APPLICATION: number;
    SelectionMode: {
      NONE: number;
    };
  }

export interface GdkDisplay {}
export interface GdkDisplayStatic {
    get_default(): GdkDisplay | null;
  }

export interface GdkNamespace {
    Display: GdkDisplayStatic;
  }

export interface AdwStyleManager {
    color_scheme: number;
  }

export interface AdwStyleManagerStatic {
    get_default(): AdwStyleManager;
  }

export interface AdwNamespace {
    StyleManager: AdwStyleManagerStatic;
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
export interface GioCancellableStatic {
    new (): GioCancellable;
  }

export interface GioBytesLike {
    toArray(): Uint8Array;
  }

export interface GioFileInfoLike {
    get_attribute_string(name: string): string | null;
    get_size(): number;
  }

export interface GioFileMonitor {
  connect(signal: string, callback: (...args: any[]) => unknown): number;
}

export interface GioFile {
    monitor_file(flags: number, cancellable: GioCancellable | null): GioFileMonitor;
    query_info(attributes: string, flags: number, cancellable: GioCancellable | null): GioFileInfoLike;
  }

export interface GioFileStatic {
    new_for_path(path: string): GioFile;
  }

export interface GioNamespace {
    ApplicationFlags: {
      NON_UNIQUE: number;
    };
    Cancellable: GioCancellableStatic;
    File: GioFileStatic;
    FileMonitorFlags: {
      NONE: number;
    };
    FileMonitorEvent: {
      CHANGES_DONE_HINT: number;
    };
    FileQueryInfoFlags: {
      NONE: number;
    };
  }

export interface GioAsyncOutputStream {
    write_bytes_async(bytes: unknown, priority: number, cancellable: GioCancellable | null, callback: (stream: unknown, result: unknown) => void): void;
    write_bytes_finish(result: unknown): unknown;
    close(cancellable: GioCancellable | null): void;
  }

export interface GioAsyncInputStream {
    read_bytes_async(size: number, priority: number, cancellable: GioCancellable | null, callback: (stream: unknown, result: unknown) => void): void;
    read_bytes_finish(result: unknown): GioBytesLike;
  }

export interface GioInputStreamStatic {
    new (props?: Record<string, unknown>): GioAsyncInputStream;
  }

export interface GioOutputStreamStatic {
    new (props?: Record<string, unknown>): GioAsyncOutputStream;
  }

export interface GioUnixNamespace {
    InputStream: GioInputStreamStatic;
    OutputStream: GioOutputStreamStatic;
  }

export interface GLibBytesStatic {
    new (data: Uint8Array): unknown;
  }

export interface GLibNamespace {
    getenv(name: string): string | null;
    PRIORITY_DEFAULT: number;
    Bytes: GLibBytesStatic;
  }

export interface GObjectNamespace {
    registerClass<T>(klass: T): T;
  }

export {};
