# Fedora Kernel Manager - Complete Code Breakdown

This document provides a line-by-line breakdown of every piece of code in the Fedora Kernel Manager application.

---

## Table of Contents

1. [Cargo.toml - Project Configuration](#cargotoml)
2. [build.rs - Build Script](#buildrs)
3. [src/main.rs - Application Entry Point](#srcmainrs)
4. [src/build_ui/mod.rs - UI Construction](#srcbuild_uimodrs)
5. [src/content/mod.rs - Main Content Logic](#srccontentmodrs)
6. [src/kernel_pkg/mod.rs - Kernel Package Management](#srckernel_pkgmodrs)
7. [src/kernel_package_row/mod.rs - Custom Widget](#srckernel_package_rowmodrs)
8. [src/kernel_package_row/imp.rs - Widget Implementation](#srckernel_package_rowimprs)
9. [src/sched_ext/mod.rs - Scheduler Extension Configuration](#srcsched_extmodrs)
10. [Data Files and Scripts](#data-files)

---

## Cargo.toml - Project Configuration {#cargotoml}

```toml
[package]
name = "fedora-kernel-manager"
version = "0.2.1"
edition = "2021"
```

**Line 1-2**: Package metadata section defining the project name and version.
- `name`: The crate name used for compilation and distribution
- `version`: Semantic version (0.2.1 = major.minor.patch)
- `edition = "2021"`: Uses Rust 2021 edition features

```toml
[dependencies]
adw = { version = "0.7.0", package = "libadwaita", features = ["v1_7"] }
```

**Line 9-10**: Dependencies section listing all external crates.
- `adw`: Libadwaita bindings for Rust, version 0.7.0, using the "libadwaita" package with v1_7 API features enabled. This provides modern GNOME UI components.

```toml
async-channel = "2.3.1"
```

**Line 11**: Async channel library for cross-thread communication without blocking.

```toml
duct = "0.13.7"
```

**Line 12**: Duct library for running child processes with piped I/O, used for executing shell commands.

```toml
gtk = { version = "0.9.2", package = "gtk4", features = ["v4_16"] }
```

**Line 13**: GTK4 bindings, version 0.9.2, with v4_16 API features for modern GTK functionality.

```toml
homedir = "0.2.1"
```

**Line 14**: Library to get the current user's home directory path.

```toml
os_pipe = "1.2.0"
```

**Line 15**: Cross-platform pipe creation for inter-process communication.

```toml
reqwest = { version = "0.11", features = ["blocking"] }
```

**Line 16**: HTTP client library with blocking features for synchronous web requests (used to download kernel branch databases).

```toml
serde_json = "1.0.117"
```

**Line 17**: JSON serialization/deserialization library.

```toml
version-compare = "0.2.0"
```

**Line 18**: Library for comparing version strings (e.g., "6.6" vs "6.5").

```toml
rust-i18n = "3.0.1"
```

**Line 19**: Internationalization library for multi-language support.

```toml
textwrap = "0.16.1"
```

**Line 20**: Text wrapping library for formatting long descriptions.

```toml
serde = { version = "1.0.219", features = ["derive"] }
```

**Line 21**: Serialization framework with derive macros for automatic trait implementation.

```toml
chrono = "0.4.41"
```

**Line 22**: Date and time library (though not heavily used in this codebase).

```toml
regex = "1.11.1"
```

**Line 23**: Regular expression library.

```toml
[build-dependencies]
glib-build-tools = "0.19.0"
```

**Line 25-26**: Build-time dependencies.
- `glib-build-tools`: Compiles GResource files (bundled data files) during build.

```toml
[profile.release]
opt-level = 'z'     # Optimize for size
lto = true          # Enable link-time optimization
codegen-units = 1   # Reduce number of codegen units to increase optimizations
panic = 'abort'     # Abort on panic
strip = true        # Strip symbols from binary*
```

**Line 28-33**: Release build profile optimizations.
- `opt-level = 'z'`: Optimize for smallest binary size
- `lto = true`: Link-time optimization for better performance
- `codegen-units = 1`: Single codegen unit for maximum optimization
- `panic = 'abort'`: Abort on panic instead of unwinding (smaller binary)
- `strip = true`: Remove debug symbols from final binary

---

## build.rs - Build Script {#buildrs}

```rust
fn main() {
    glib_build_tools::compile_resources(
        &["data"],
        "data/resources.gresource.xml",
        "data.gresource",
    );
}
```

**Line 1**: Entry point for the build script, executed before compilation.

**Line 2-6**: Calls `compile_resources` to:
- Search in the `data` directory
- Read `data/resources.gresource.xml` to find files to bundle
- Output `data.gresource` binary file containing all resources (icons, CSS)
- This binary is embedded in the final executable

**Purpose**: Bundles CSS files and SVG icons into the binary so they don't need to be installed separately.

---

## src/main.rs - Application Entry Point {#srcmainrs}

### Module Declarations

```rust
mod build_ui;
mod content;
mod kernel_package_row;
mod kernel_pkg;
mod sched_ext;
```

**Lines 1-5**: Declares all submodules that will be compiled and linked.

### Imports

```rust
use adw::prelude::*;
use gtk::{gdk, gio, glib, CssProvider};
use rust_i18n::Backend;
use std::collections::HashMap;
use std::env;
use std::fs;
use crate::gdk::Display;
```

**Line 7**: Imports Libadwaita prelude traits for widget methods.

**Line 8**: Imports GTK types:
- `gdk`: Graphics/display handling
- `gio`: GLib I/O and application framework
- `glib`: Core GLib types
- `CssProvider`: For loading CSS styles

**Line 9**: Imports `Backend` trait for custom i18n implementation.

**Line 10-12**: Standard library imports for collections, environment variables, and file system operations.

**Line 14**: Imports `Display` type for screen/display management.

### I18nBackend Structure

```rust
pub struct I18nBackend {
    trs: HashMap<String, HashMap<String, String>>,
}
```

**Lines 16-18**: Custom internationalization backend structure.
- `trs`: Nested HashMap storing translations
  - Outer key: locale (e.g., "en_US", "fr_FR")
  - Inner key: translation key (e.g., "application_name")
  - Value: Translated string

### I18nBackend::new()

```rust
impl I18nBackend {
    fn new() -> Self {
        let mut trs = HashMap::new();
        let locales_dir = fs::read_dir("/usr/lib/fedora-kernel-manager/locales")
            .expect("No translation files found");
```

**Line 19-23**: Constructor for I18nBackend.
- Creates empty HashMap for translations
- Reads directory containing locale JSON files
- Panics if directory doesn't exist (expected at install time)

```rust
        for locale_file in locales_dir {
            let locale_file_path = locale_file
                .expect("couldn't change dir entry to path")
                .path();
```

**Lines 24-27**: Iterates over each file in the locales directory.
- `locale_file`: Directory entry (Result type)
- Extracts the file path, panicking on error

```rust
            let locale = String::from(
                locale_file_path
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .trim_end_matches(".json"),
            );
```

**Lines 28-35**: Extracts locale name from filename.
- Gets filename (e.g., "en_US.json")
- Converts to string
- Removes ".json" extension to get "en_US"

```rust
            let locale_data = fs::read_to_string(locale_file_path)
                .expect(format!("invalid json for {}", locale).as_str());
            let locale_json =
                serde_json::from_str::<HashMap<String, String>>(&locale_data).unwrap();
            trs.insert(locale.to_string(), locale_json);
        }
```

**Lines 36-40**: Loads and parses translation file.
- Reads entire JSON file as string
- Parses JSON into HashMap<String, String> (key-value pairs)
- Inserts into outer HashMap with locale as key

```rust
        return Self { trs };
    }
}
```

**Lines 43-44**: Returns new I18nBackend instance with loaded translations.

### Backend Trait Implementation

```rust
impl Backend for I18nBackend {
    fn available_locales(&self) -> Vec<&str> {
        return self.trs.keys().map(|k| k.as_str()).collect();
    }
```

**Lines 47-50**: Implements `available_locales()` method.
- Returns vector of all locale names as string slices
- Used by i18n system to know which languages are available

```rust
    fn translate(&self, locale: &str, key: &str) -> Option<&str> {
        return self.trs.get(locale)?.get(key).map(|k| k.as_str());
    }
}
```

**Lines 52-54**: Implements `translate()` method.
- Looks up locale in outer HashMap
- Then looks up key in inner HashMap
- Returns `Option<&str>` - Some(translated string) or None if not found
- Uses `?` operator to short-circuit on missing locale

### I18n Macro Setup

```rust
#[macro_use]
extern crate rust_i18n;
i18n!(fallback = "en_US", backend = I18nBackend::new());
```

**Lines 57-59**: Sets up internationalization system.
- `#[macro_use]`: Enables macro imports from rust_i18n
- `i18n!`: Macro that configures the i18n system
  - `fallback = "en_US"`: Use English if translation missing
  - `backend = I18nBackend::new()`: Use custom backend that reads JSON files

### Constants

```rust
const APP_ID: &str = "com.github.cosmicfusion.fedora-kernel-manager";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_ICON: &str = "com.github.cosmicfusion.fedora-kernel-manager";
pub const APP_GITHUB: &str = "https://github.com/CosmicFusion/fedora-kernel-manager";
```

**Line 61**: Application ID following reverse domain notation (used by GNOME/Flatpak).

**Line 62**: Version constant from Cargo.toml at compile time via `env!` macro.

**Line 63**: Icon name for the application (matches desktop file).

**Line 64**: GitHub repository URL for about dialog.

### Data Structures

```rust
#[derive(Clone)]
struct RunningKernelInfo {
    kernel: String,
    version: String,
    sched: String,
}
```

**Lines 66-71**: Structure holding current kernel information.
- `kernel`: Full kernel release string (e.g., "6.6.5-200.fc39.x86_64")
- `version`: Kernel version only (e.g., "6.6.5")
- `sched`: Scheduler name (e.g., "CFS", "BORE", "sched_ext: scx_bpfland")
- `#[derive(Clone)]`: Allows copying the struct

```rust
#[allow(dead_code)]
#[derive(Clone)]
struct KernelBranch {
    name: String,
    db_url: String,
    db: String,
    init_script: String,
}
```

**Lines 73-80**: Structure for kernel branch configuration.
- `name`: Display name (e.g., "kernel-cachyos")
- `db_url`: URL to download kernel database JSON
- `db`: Cached database JSON content
- `init_script`: Command to run to initialize repository
- `#[allow(dead_code)]`: Suppresses warning (struct is used but Rust can't detect it)

```rust
struct KernelPackage {
    name: String,
    main_package: String,
    packages: String,
    min_x86_march: u32,
    package_version: String,
    description: String,
}
```

**Lines 82-89**: Structure for individual kernel package information.
- `name`: Display name
- `main_package`: Primary RPM package name
- `packages`: Space-separated list of all packages to install
- `min_x86_march`: Minimum x86 microarchitecture level (1-4)
- `package_version`: Version string from RPM
- `description`: Package description

### main() Function

```rust
fn main() -> glib::ExitCode {
    let current_locale = match env::var_os("LANG") {
        Some(v) => v
            .into_string()
            .unwrap()
            .chars()
            .take_while(|&ch| ch != '.')
            .collect::<String>(),
        None => panic!("$LANG is not set"),
    };
```

**Line 91**: Entry point returning GLib exit code.

**Lines 92-99**: Detects current locale from environment.
- Reads `LANG` environment variable (e.g., "en_US.UTF-8")
- Extracts locale part before dot (e.g., "en_US")
- Panics if `LANG` not set (should always be set on Linux)

```rust
    rust_i18n::set_locale(&current_locale);
```

**Line 101**: Sets the active locale for translations.

```rust
    let app = adw::Application::builder().application_id(APP_ID).build();
```

**Line 103**: Creates Libadwaita Application instance.
- Uses builder pattern
- Sets application ID for GNOME integration

```rust
    app.connect_startup(|app| {
        load_gresource();
        load_css();
        app.connect_activate(build_ui::build_ui);
    });
```

**Lines 105-109**: Connects startup signal handler.
- `load_gresource()`: Loads bundled resources (icons, CSS)
- `load_css()`: Applies CSS stylesheet
- `app.connect_activate()`: Connects activation handler (runs when app starts)

```rust
    // Run the application
    app.run()
}
```

**Lines 111-113**: Starts the GTK main loop and runs the application.

### load_gresource()

```rust
fn load_gresource() {
    gio::resources_register_include!("data.gresource").expect("Failed to register resources.");
}
```

**Lines 115-117**: Registers embedded GResource file.
- `data.gresource` was compiled by build.rs
- Makes resources available at runtime via resource paths
- Panics on failure (resources are required)

### load_css()

```rust
fn load_css() {
    // Load the CSS file and add it to the provider
    let provider = CssProvider::new();
    provider.load_from_resource("/com/github/cosmicfusion/fedora-kernel-manager/css/style.css");
```

**Lines 119-122**: Creates CSS provider and loads stylesheet.
- `CssProvider`: GTK object that holds CSS rules
- `load_from_resource()`: Loads CSS from embedded GResource

```rust
    // Add the provider to the default screen
    gtk::style_context_add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
```

**Lines 124-129**: Applies CSS to the display.
- Gets default display (screen)
- Adds CSS provider with APPLICATION priority (overrides theme defaults)
- All widgets will now use these styles

---

## src/build_ui/mod.rs - UI Construction {#srcbuild_uimodrs}

### Imports

```rust
use crate::APP_GITHUB;
use crate::APP_ICON;
use crate::APP_ID;
use crate::VERSION;
use crate::{content, KernelBranch};
```

**Lines 1-5**: Imports constants and types from main module.

```rust
use adw::prelude::*;
use adw::*;
use glib::{clone, MainContext};
use gtk::*;
use std::cell::RefCell;
use std::process::Command;
use std::rc::Rc;
use std::{thread, time};
```

**Lines 6-13**: Imports for UI and async operations.
- `clone!`: Macro for cloning variables into closures
- `MainContext`: GLib main loop context for async operations
- `RefCell`: Interior mutability for shared state
- `Rc`: Reference-counted smart pointer
- `Command`: For executing system commands
- `thread`, `time`: For threading and sleep

### build_ui() Function

```rust
pub fn build_ui(app: &adw::Application) {
    gtk::glib::set_prgname(Some(t!("application_name").to_string()));
    glib::set_application_name(&t!("application_name").to_string());
```

**Line 15**: Main UI construction function.

**Lines 16-17**: Sets application name for system integration.
- `set_prgname()`: Sets process name
- `set_application_name()`: Sets display name
- `t!()`: Translation macro (looks up "application_name" key)

```rust
    let theme_changed_action = gio::SimpleAction::new("theme_changed", None);
```

**Line 19**: Creates action for theme change events.
- Used to update UI when user changes accent color

```rust
    let internet_connected = Rc::new(RefCell::new(false));
    let selected_kernel_branch: Rc<RefCell<KernelBranch>> = Rc::new(RefCell::new(KernelBranch {
        name: "?".to_owned(),
        db_url: "?".to_owned(),
        db: "?".to_owned(),
        init_script: "?".to_owned(),
    }));
    let db_load_complete = Rc<RefCell::new(false));
```

**Lines 21-28**: Shared state variables.
- `internet_connected`: Tracks internet connectivity (wrapped in Rc<RefCell> for sharing)
- `selected_kernel_branch`: Currently selected branch (initialized with placeholder)
- `db_load_complete`: Flag indicating database has loaded

```rust
    let (internet_loop_sender, internet_loop_receiver) = async_channel::unbounded();
    let internet_loop_sender = internet_loop_sender.clone();
```

**Lines 30-31**: Creates async channel for internet status updates.
- Unbounded channel (no size limit)
- Clones sender for use in thread

```rust
    std::thread::spawn(move || loop {
        match Command::new("ping").arg("google.com").arg("-c 1").output() {
            Ok(t) if t.status.success() => internet_loop_sender
                .send_blocking(true)
                .expect("The channel needs to be open"),
            _ => internet_loop_sender
                .send_blocking(false)
                .expect("The channel needs to be open"),
        };
        thread::sleep(time::Duration::from_secs(5));
    });
```

**Lines 33-43**: Spawns background thread for internet monitoring.
- Runs in infinite loop
- Pings google.com with 1 packet
- Sends `true` if ping succeeds, `false` otherwise
- Sleeps 5 seconds between checks
- `send_blocking()`: Blocks until message sent

```rust
    let window_banner = adw::Banner::builder().revealed(false).build();
```

**Line 45**: Creates banner widget for showing warnings.
- Initially hidden (`revealed(false)`)

```rust
    let internet_connected_status = internet_connected.clone();
    let selected_kernel_branch2 = selected_kernel_branch.clone();
```

**Lines 47-49**: Clones shared state for use in closures.

```rust
    let internet_loop_context = MainContext::default();
    // The main loop executes the asynchronous block
    internet_loop_context.spawn_local(clone!(
        #[weak]
        window_banner,
        async move {
            while let Ok(state) = internet_loop_receiver.recv().await {
```

**Lines 51-57**: Sets up async receiver for internet status.
- `MainContext::default()`: Gets main GLib context
- `spawn_local()`: Spawns async task on main context
- `clone!()`: Clones variables into closure
- `#[weak]`: Weak reference (won't prevent widget destruction)
- `while let Ok(state)`: Receives messages from channel

```rust
                let banner_text = t!("banner_text_no_internet").to_string();
                if state == true {
                    *internet_connected_status.borrow_mut() = true;
                    if window_banner.title() == banner_text {
                        window_banner.set_revealed(false)
                    }
                } else {
                    *internet_connected_status.borrow_mut() = false;
                    if window_banner.title() != t!("banner_text_url_error").to_string() {
                        window_banner.set_title(&banner_text);
                        window_banner.set_revealed(true)
                    }
                }
            }
        }
    ));
```

**Lines 58-71**: Handles internet status updates.
- If connected: Updates flag, hides banner if showing "no internet"
- If disconnected: Updates flag, shows "no internet" banner (unless showing URL error)

```rust
    let window_headerbar = adw::HeaderBar::builder()
        .title_widget(
            &adw::WindowTitle::builder()
                .title(t!("application_name"))
                .build(),
        )
        .build();
```

**Lines 75-81**: Creates header bar with title.
- Modern GNOME-style header bar
- `WindowTitle`: Adaptive title widget

```rust
    let content_stack = gtk::Stack::builder()
        .transition_type(StackTransitionType::Crossfade)
        .build();
```

**Lines 83-85**: Creates stack widget for page navigation.
- `Stack`: Container that shows one child at a time
- `Crossfade`: Smooth fade transition between pages

```rust
    let window_toolbar = adw::ToolbarView::builder().content(&content_stack).build();
```

**Line 87**: Creates toolbar view (Libadwaita widget for modern layouts).

```rust
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .content(&window_toolbar)
        .width_request(600)
        .height_request(600)
        .resizable(false)
        .icon_name(APP_ICON)
        .startup_id(APP_ID)
        .build();
```

**Lines 89-97**: Creates main application window.
- Sets fixed size (600x600)
- Non-resizable
- Sets icon and startup ID for GNOME integration

```rust
    let (gsettings_change_sender, gsettings_change_receiver) = async_channel::unbounded();
    let gsettings_change_sender_clone0 = gsettings_change_sender.clone();
```

**Lines 99-100**: Creates channel for theme change notifications.

```rust
    thread::spawn(move || {
        let context = glib::MainContext::default();
        let main_loop = glib::MainLoop::new(Some(&context), false);
        let gsettings = gtk::gio::Settings::new("org.gnome.desktop.interface");
```

**Lines 102-105**: Spawns thread to monitor GSettings.
- Creates GLib main loop
- Opens GSettings schema for desktop interface settings

```rust
        gsettings.connect_changed(
            Some("accent-color"),
            clone!(
                #[strong]
                gsettings_change_sender_clone0,
                move |_, _| {
                    let gsettings_change_sender_clone0 = gsettings_change_sender_clone0.clone();
                    glib::timeout_add_seconds_local(5, move || {
                        gsettings_change_sender_clone0.send_blocking(()).unwrap();
                        glib::ControlFlow::Break
                    });
                }
            ),
        );
```

**Lines 106-118**: Connects to accent-color change signal.
- `connect_changed()`: Fires when "accent-color" setting changes
- `clone!()`: Clones sender into closure
- `timeout_add_seconds_local(5)`: Waits 5 seconds before sending (debounce)
- Sends empty message `()` to notify of change
- `ControlFlow::Break`: Timer fires once

```rust
        main_loop.run()
    });
```

**Line 120**: Runs main loop in thread (blocks until app closes).

```rust
    let gsettings_changed_context = MainContext::default();
    // The main loop executes the asynchronous block
    gsettings_changed_context.spawn_local(clone!(
        #[strong]
        theme_changed_action,
        async move {
            while let Ok(()) = gsettings_change_receiver.recv().await {
                theme_changed_action.activate(None);
            }
        }
    ));
```

**Lines 123-133**: Receives theme change notifications.
- Activates `theme_changed_action` when accent color changes
- This triggers UI updates (badge colors, etc.)

```rust
    content_stack.add_named(
        &content::content(
            &content_stack,
            &selected_kernel_branch2,
            &db_load_complete,
            &window,
            &window_banner,
            &theme_changed_action,
        ),
        Some("content_page"),
    );
```

**Lines 135-145**: Adds main content page to stack.
- Calls `content::content()` to build main UI
- Names it "content_page" for navigation

```rust
    window_toolbar.add_top_bar(&window_headerbar);
    window_toolbar.add_top_bar(&window_banner);
```

**Lines 147-148**: Adds header bar and banner to toolbar view.

```rust
    load_icon_theme(&window);
```

**Line 150**: Loads custom icons from resources.

```rust
    window.connect_close_request(move |window| {
        if let Some(application) = window.application() {
            application.remove_window(window);
        }
        glib::Propagation::Proceed
    });
```

**Lines 152-157**: Handles window close request.
- Removes window from application
- Returns `Proceed` to allow closing

```rust
    let credits_button = gtk::Button::builder()
        .icon_name("dialog-information-symbolic")
        .build();
```

**Lines 159-161**: Creates info button for about dialog.

```rust
    let credits_window = adw::AboutDialog::builder()
        .application_icon(APP_ICON)
        .application_name(t!("application_name"))
        .version(VERSION)
        .developer_name(t!("developer_name"))
        .license_type(License::Gpl20)
        .issue_url(APP_GITHUB.to_owned() + "/issues")
        .build();
```

**Lines 163-170**: Creates about dialog with app information.

```rust
    window_headerbar.pack_end(&credits_button);
    credits_button.connect_clicked(clone!(
        #[strong]
        window,
        move |_| credits_window.present(Some(&window))
    ));
```

**Lines 172-177**: Adds button to header and connects click handler.
- `pack_end()`: Adds to right side of header
- Shows about dialog when clicked

```rust
    window.present();
}
```

**Line 179**: Shows the window.

### load_icon_theme()

```rust
fn load_icon_theme(window: &adw::ApplicationWindow) {
    let icon_theme = gtk::IconTheme::for_display(&WidgetExt::display(window));
```

**Lines 182-183**: Gets icon theme for the display.

```rust
    icon_theme.add_resource_path("/com/github/cosmicfusion/fedora-kernel-manager/icons/");
    icon_theme.add_resource_path(
        "/com/github/cosmicfusion/fedora-kernel-manager/icons/scalable/actions/",
    );
}
```

**Lines 185-188**: Adds resource paths so custom icons can be found by name.

---

## src/content/mod.rs - Main Content Logic {#srccontentmodrs}

This module contains the main content page logic, kernel branch management, and kernel information display.

### Imports

```rust
use crate::{kernel_pkg, sched_ext, KernelBranch, RunningKernelInfo};
use adw::prelude::*;
use async_channel::Receiver;
use duct::cmd;
use glib::*;
use gtk::*;
use homedir::get_my_home;
use std::cell::RefCell;
use std::path::Path;
use std::process::{Command, Stdio};
use std::rc::Rc;
use std::{fs, time};
use version_compare::Version;
use Vec;
```

**Line 1**: Imports modules and types from other parts of the application.

**Lines 2-14**: Standard imports for UI, async operations, system commands, and file operations.

### content() Function

```rust
pub fn content(
    content_stack: &gtk::Stack,
    selected_kernel_branch: &Rc<RefCell<KernelBranch>>,
    db_load_complete: &Rc<RefCell<bool>>,
    window: &adw::ApplicationWindow,
    window_banner: &adw::Banner,
    theme_changed_action: &gio::SimpleAction,
) -> gtk::Box {
```

**Lines 16-23**: Main content page builder function.
- Takes references to shared state and UI components
- Returns a `Box` widget containing the entire content page

```rust
    let (get_kernel_branches_sender, get_kernel_branches_receiver) = async_channel::unbounded();
    let get_kernel_branches_sender = get_kernel_branches_sender.clone();
```

**Lines 24-26**: Creates channel for kernel branch loading.
- Sender will be used in background thread
- Receiver will be used in async task

```rust
    std::thread::spawn(move || {
        get_kernel_branches_sender
            .send_blocking(get_kernel_branches())
            .expect("channel needs to be open.");
    });
```

**Lines 27-31**: Spawns thread to load kernel branches.
- Calls `get_kernel_branches()` which reads JSON files and downloads databases
- Sends result through channel

```rust
    let loading_spinner = gtk::Spinner::builder()
        .hexpand(true)
        .valign(Align::Start)
        .halign(Align::Center)
        .spinning(true)
        .height_request(128)
        .width_request(128)
        .build();
```

**Lines 33-40**: Creates loading spinner widget.
- Large spinner (128x128) shown while loading

```rust
    let loading_label = gtk::Label::builder()
        .hexpand(true)
        .valign(Align::Start)
        .halign(Align::Center)
        .label(t!("loading_label_label"))
        .build();
```

**Lines 42-47**: Creates loading label ("Downloading Database...").

```rust
    let loading_box = gtk::Box::builder()
        .hexpand(true)
        .vexpand(true)
        .orientation(Orientation::Vertical)
        .build();
    loading_box.append(&loading_spinner);
    loading_box.append(&loading_label);
```

**Lines 49-56**: Creates container for loading UI.
- Vertical box containing spinner and label

```rust
    let content_box = gtk::Box::builder()
        .hexpand(true)
        .vexpand(true)
        .orientation(Orientation::Vertical)
        .sensitive(false)
        .build();
```

**Lines 58-63**: Creates main content container.
- Initially disabled (`sensitive(false)`) until data loads

```rust
    let tux_icon = gtk::Image::builder()
        .pixel_size(128)
        .halign(Align::Center)
        .hexpand(true)
        .margin_start(10)
        .margin_end(10)
        .margin_bottom(20)
        .margin_top(20)
        .build();
    tux_icon.set_icon_name(Some("tux-symbolic"));
    tux_icon.add_css_class("symbolic-accent-bg");
```

**Lines 65-77**: Creates Tux (Linux mascot) icon.
- Large icon with margins
- Uses custom "tux-symbolic" icon from resources
- Applies accent color via CSS class

```rust
    let kernel_badge_box = gtk::Box::builder()
        .hexpand(true)
        .vexpand(true)
        .orientation(Orientation::Vertical)
        .build();
    let sched_ext_badge_box = adw::Bin::builder().hexpand(true).vexpand(true).build();
```

**Lines 79-85**: Creates containers for kernel information badges.
- `kernel_badge_box`: Holds multiple badges (branch, version, etc.)
- `sched_ext_badge_box`: Holds scheduler badge

```rust
    let kernel_branch_expander_row = adw::ExpanderRow::builder()
        .subtitle(t!("kernel_branch_expander_row_subtitle"))
        .build();
```

**Lines 87-89**: Creates expandable row for kernel branch selection.

```rust
    kernel_branch_expander_row.add_row(&kernel_branch_expandable(
        &kernel_branch_expander_row,
        &window_banner,
        &loading_box,
        selected_kernel_branch,
        db_load_complete,
        get_kernel_branches_receiver.clone(),
    ));
```

**Lines 91-98**: Adds expandable content to branch selector.
- `kernel_branch_expandable()` builds the list of branches

```rust
    let kernel_branch_expander_row_boxedlist = gtk::ListBox::builder()
        .selection_mode(SelectionMode::None)
        .hexpand(true)
        .halign(Align::Center)
        .margin_start(10)
        .margin_end(10)
        .margin_bottom(20)
        .margin_top(20)
        .build();
    kernel_branch_expander_row_boxedlist.add_css_class("boxed-list");
    kernel_branch_expander_row_boxedlist.append(&kernel_branch_expander_row);
```

**Lines 100-108**: Creates list box for branch selector with styling.

```rust
    let button_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .margin_start(10)
        .margin_end(10)
        .margin_bottom(20)
        .margin_top(20)
        .hexpand(true)
        .halign(Align::Center)
        .build();
```

**Lines 110-118**: Creates horizontal container for action buttons.

```rust
    let browse_kernels_button = gtk::Button::builder()
        .icon_name("folder-download-symbolic")
        .halign(Align::Start)
        .margin_start(10)
        .margin_end(10)
        .height_request(50)
        .width_request(50)
        .tooltip_text(t!("browse_kernels_button_tooltip_text"))
        .hexpand(true)
        .build();
    browse_kernels_button.add_css_class("circular");
```

**Lines 120-130**: Creates "Browse Kernels" button.
- Circular icon button
- Opens kernel package selection page

```rust
    browse_kernels_button.connect_clicked(clone!(
        #[weak]
        window,
        #[weak]
        content_stack,
        #[weak]
        selected_kernel_branch,
        move |_| {
            kernel_pkg::kernel_pkg_page(
                &content_stack,
                &window,
                &selected_kernel_branch,
            );
            content_stack.set_visible_child_name("kernel_pkg_page")
        }
    ));
```

**Lines 132-147**: Connects button click handler.
- Navigates to kernel package page
- Uses weak references to avoid circular dependencies

```rust
    let config_kernel_button = gtk::Button::builder()
        .icon_name("emblem-system-symbolic")
        .halign(Align::End)
        .margin_start(10)
        .margin_end(10)
        .height_request(50)
        .width_request(50)
        .tooltip_text(t!("config_kernel_button_tooltip_text"))
        .hexpand(true)
        .build();
    config_kernel_button.add_css_class("circular");
```

**Lines 149-159**: Creates "Configure Kernel" button for SCX settings.

```rust
    if !is_scx_kernel() {
        config_kernel_button.set_sensitive(false);
        config_kernel_button.set_tooltip_text(Some(
            &t!("config_kernel_button_tooltip_text_no_scx").to_string(),
        ));
    } else if is_scx_kernel() && !is_scx_installed() {
        config_kernel_button.set_sensitive(false);
        config_kernel_button.set_tooltip_text(Some(
            &t!("config_kernel_button_tooltip_text_no_scx_installed").to_string(),
        ));
    }
```

**Lines 161-171**: Disables button if SCX not supported or not installed.
- Checks if `/sys/kernel/sched_ext` exists (SCX support)
- Checks if `scx` systemd service is installed

```rust
    config_kernel_button.connect_clicked(clone!(
        #[weak]
        content_stack,
        #[weak]
        window,
        #[weak]
        sched_ext_badge_box,
        move |_| {
            content_stack.add_named(
                &sched_ext::sched_ext_page(&content_stack, &window, &sched_ext_badge_box),
                Some("sched_ext_page"),
            );
            content_stack.set_visible_child_name("sched_ext_page")
        }
    ));
```

**Lines 176-190**: Connects button to open SCX configuration page.

```rust
    button_box.append(&browse_kernels_button);
    button_box.append(&config_kernel_button);
    kernel_branch_expander_row_boxedlist.add_css_class("boxed-list");
    kernel_branch_expander_row_boxedlist.append(&kernel_branch_expander_row);
    content_box.append(&loading_box);
    content_box.append(&kernel_badge_box);
    content_box.append(&tux_icon);
    content_box.append(&kernel_branch_expander_row_boxedlist);
    content_box.append(&button_box);
```

**Lines 192-202**: Assembles all widgets into content box.

```rust
    let (load_badge_async_sender, load_badge_async_receiver) = async_channel::unbounded();
    let load_badge_async_sender = load_badge_async_sender.clone();
    // The long running operation runs now in a separate thread
    std::thread::spawn(move || loop {
        load_badge_async_sender
            .send_blocking(true)
            .expect("The channel needs to be open.");
        std::thread::sleep(time::Duration::from_secs(5));
    });
```

**Lines 204-212**: Spawns thread that sends periodic updates every 5 seconds.
- Used to refresh kernel badges periodically

```rust
    let load_badge_async_context = MainContext::default();
    // The main loop executes the asynchronous block
    load_badge_async_context.spawn_local(clone!(
        #[strong]
        content_box,
        #[weak]
        content_box,
        #[weak]
        kernel_badge_box,
        #[weak]
        selected_kernel_branch,
        #[strong]
        db_load_complete,
        #[strong]
        theme_changed_action,
        async move {
            while let Ok(_state) = load_badge_async_receiver.recv().await {
                if *db_load_complete.borrow() == true {
                    let running_kernel_info = get_running_kernel_info();
                    create_kernel_badges(
                        &kernel_badge_box,
                        &running_kernel_info,
                        &selected_kernel_branch,
                        &theme_changed_action,
                    );
                    create_current_sched_badge(
                        &sched_ext_badge_box,
                        &running_kernel_info,
                        &theme_changed_action,
                    );
                    loading_box.set_visible(false);
                    content_box.set_sensitive(true)
                }
            }
        }
    ));
```

**Lines 214-249**: Async task that updates badges every 5 seconds.
- Waits for database to load (`db_load_complete`)
- Gets current kernel info
- Updates all badges
- Hides loading spinner and enables content

```rust
    content_box
}
```

**Line 251**: Returns the completed content box.

### kernel_branch_expandable() Function

```rust
fn kernel_branch_expandable(
    expander_row: &adw::ExpanderRow,
    window_banner: &adw::Banner,
    loading_box: &gtk::Box,
    selected_kernel_branch: &Rc<RefCell<KernelBranch>>,
    db_load_complete: &Rc<RefCell<bool>>,
    get_kernel_branches_receiver: Receiver<Result<Vec<KernelBranch>, reqwest::Error>>,
) -> gtk::ListBox {
```

**Lines 254-261**: Builds the expandable branch selection UI.

```rust
    let searchbar = gtk::SearchEntry::builder().search_delay(500).build();
    searchbar.add_css_class("round-border-only-top");
```

**Lines 262-263**: Creates search entry with 500ms delay.

```rust
    let boxedlist = gtk::ListBox::builder()
        .selection_mode(SelectionMode::None)
        .build();
    boxedlist.append(&searchbar);
```

**Lines 265-269**: Creates list box and adds search bar.

```rust
    let branch_container = gtk::ListBox::builder()
        .selection_mode(SelectionMode::None)
        .build();
    branch_container.add_css_class("boxed-list");
```

**Lines 271-274**: Creates container for branch list items.

```rust
    let null_checkbutton = gtk::CheckButton::builder()
        .label(t!("null_checkbutton_label"))
        .build();
```

**Lines 276-278**: Creates hidden checkbutton for radio button grouping.

```rust
    let get_kernel_branches_loop_context = MainContext::default();
    // The main loop executes the asynchronous block
    get_kernel_branches_loop_context.spawn_local(clone!(
        #[weak]
        expander_row,
        #[weak]
        branch_container,
        #[strong]
        selected_kernel_branch,
        #[weak]
        loading_box,
        #[weak]
        window_banner,
        #[strong]
        db_load_complete,
        async move {
            while let Ok(data) = get_kernel_branches_receiver.recv().await {
                match data {
                    Ok(t) => {
                        for branch in t {
```

**Lines 280-296**: Async task that receives loaded branches.
- Processes each branch from the vector

```rust
                            let branch_clone0 = branch.clone();
                            let branch_clone1 = branch.clone();
                            let branch_checkbutton = gtk::CheckButton::builder()
                                .valign(Align::Center)
                                .can_focus(false)
                                .active(false)
                                .build();
                            let branch_row = adw::ActionRow::builder()
                                .activatable_widget(&branch_checkbutton)
                                .title(branch.name)
                                .build();
                            branch_row.add_prefix(&branch_checkbutton);
                            branch_checkbutton.set_group(Some(&null_checkbutton));
                            branch_container.append(&branch_row);
```

**Lines 300-313**: Creates UI row for each branch.
- Radio button group (only one selectable)
- Action row with branch name

```rust
                            let selected_kernel_branch_clone0 = selected_kernel_branch.clone();
                            branch_checkbutton.connect_toggled(clone!(
                                #[weak]
                                branch_checkbutton,
                                #[weak]
                                expander_row,
                                #[strong]
                                branch_clone0,
                                move |_| {
                                    if branch_checkbutton.is_active() == true {
                                        expander_row.set_title(&branch_row.title());
                                        save_branch_config(&branch_row.title().to_string());
                                        *selected_kernel_branch_clone0.borrow_mut() =
                                            branch_clone0.clone()
                                    }
                                }
                            ));
```

**Lines 315-330**: Connects toggle handler.
- When selected, updates expander title
- Saves selection to config file
- Updates shared state

```rust
                            match get_my_home()
                                .unwrap()
                                .unwrap()
                                .join(".config/fedora-kernel-manager/branch")
                                .exists()
                            {
                                true if fs::read_to_string(
                                    get_my_home()
                                        .unwrap()
                                        .unwrap()
                                        .join(".config/fedora-kernel-manager/branch"),
                                )
                                .unwrap()
                                .trim()
                                .eq(branch_clone1.name.trim()) =>
                                {
                                    branch_checkbutton.set_active(true)
                                }
                                false => branch_container
                                    .first_child()
                                    .unwrap()
                                    .property::<gtk::CheckButton>("activatable_widget")
                                    .set_property("active", true),
                                _ => {}
                            };
```

**Lines 332-356**: Restores saved branch selection.
- Reads config file
- If matches current branch, selects it
- If no config, selects first branch

```rust
                            *db_load_complete.borrow_mut() = true;
                            println!("{} {}", branch_clone0.name, t!("db_load_complete"))
                        }
                    }
                    _ => {
                        window_banner.set_title(&t!("banner_text_url_error").to_string());
                        window_banner.set_revealed(true);
                        loading_box.set_visible(false);
                    }
                }
            }
        }
    ));
```

**Lines 358-367**: Handles errors and completion.
- Sets `db_load_complete` flag
- Shows error banner if download failed

```rust
    let branch_container_viewport = gtk::ScrolledWindow::builder()
        .child(&branch_container)
        .hscrollbar_policy(PolicyType::Never)
        .build();
    branch_container.add_css_class("round-border-only-bottom");
    boxedlist.append(&branch_container_viewport);
```

**Lines 372-379**: Wraps branch list in scrollable viewport.

```rust
    searchbar.connect_search_changed(clone!(
        #[weak]
        searchbar,
        #[weak]
        branch_container,
        move |_| {
            let mut counter = branch_container.first_child();
            while let Some(row) = counter {
                if row.widget_name() == "AdwActionRow" {
                    if !searchbar.text().is_empty() {
                        if row
                            .property::<String>("subtitle")
                            .to_lowercase()
                            .contains(&searchbar.text().to_string().to_lowercase())
                            || row
                                .property::<String>("title")
                                .to_lowercase()
                                .contains(&searchbar.text().to_string().to_lowercase())
                        {
                            row.set_property("visible", true);
                            searchbar.grab_focus();
                        } else {
                            row.set_property("visible", false);
                        }
                    } else {
                        row.set_property("visible", true);
                    }
                }
                counter = row.next_sibling();
            }
        }
    ));
```

**Lines 381-414**: Implements search filtering.
- Hides rows that don't match search text
- Case-insensitive matching on title/subtitle

```rust
    boxedlist
}
```

**Line 416**: Returns the list box.

### create_kernel_badge() Function

```rust
pub fn create_kernel_badge(
    label0_text: &str,
    label1_text: &str,
    css_style: &str,
    theme_changed_action: &gio::SimpleAction,
    group_size: &gtk::SizeGroup,
    group_size0: &gtk::SizeGroup,
    group_size1: &gtk::SizeGroup,
) -> gtk::ListBox {
```

**Lines 419-427**: Creates a badge widget showing two labels.
- `label0_text`: Left label (e.g., "Kernel Branch")
- `label1_text`: Right label (e.g., "kernel-cachyos")
- `css_style`: CSS class for styling
- Size groups ensure consistent sizing across badges

```rust
    let badge_box = gtk::Box::builder().build();
    let label0 = gtk::Label::builder()
        .label(label0_text)
        .margin_start(5)
        .margin_end(5)
        .margin_bottom(1)
        .margin_top(1)
        .valign(Align::Center)
        .halign(Align::Center)
        .hexpand(true)
        .vexpand(true)
        .build();
    group_size0.add_widget(&label0);
```

**Lines 428-441**: Creates left label with margins and adds to size group.

```rust
    let label_seprator = gtk::Separator::builder().build();
```

**Line 443**: Creates vertical separator between labels.

```rust
    let label1 = gtk::Label::builder()
        .label(label1_text)
        .margin_start(3)
        .margin_end(0)
        .margin_bottom(1)
        .margin_top(1)
        .valign(Align::Center)
        .halign(Align::Center)
        .hexpand(true)
        .vexpand(true)
        .build();
    group_size1.add_widget(&label1);
    label1.add_css_class(css_style);
```

**Lines 445-458**: Creates right label with styling class.

```rust
    #[allow(deprecated)]
    let color = label1
        .style_context()
        .lookup_color("accent_bg_color")
        .unwrap();
    if (color.red() * 0.299 + color.green() * 0.587 + color.blue() * 0.114) > 170.0 {
        label1.remove_css_class("white-color-text");
        label1.add_css_class("black-color-text");
    } else {
        label1.remove_css_class("black-color-text");
        label1.add_css_class("white-color-text");
    }
```

**Lines 460-471**: Calculates text color based on background brightness.
- Uses luminance formula (0.299*R + 0.587*G + 0.114*B)
- If background is light (>170), use black text
- Otherwise use white text
- Ensures text is always readable

```rust
    theme_changed_action.connect_activate(clone!(
        #[strong]
        label1,
        move |_, _| {
            #[allow(deprecated)]
            let color = label1
                .style_context()
                .lookup_color("accent_bg_color")
                .unwrap();
            if (color.red() * 0.299 + color.green() * 0.587 + color.blue() * 0.114) > 170.0 {
                label1.remove_css_class("white-color-text");
                label1.add_css_class("black-color-text");
            } else {
                label1.remove_css_class("black-color-text");
                label1.add_css_class("white-color-text");
            }
        }
    ));
```

**Lines 473-490**: Updates text color when theme changes.

```rust
    badge_box.append(&label0);
    badge_box.append(&label_seprator);
    badge_box.append(&label1);
    let boxedlist = gtk::ListBox::builder()
        .selection_mode(SelectionMode::None)
        .halign(Align::Center)
        .valign(Align::End)
        .margin_start(5)
        .margin_end(5)
        .margin_bottom(5)
        .margin_top(5)
        .build();
    boxedlist.add_css_class("boxed-list");
    boxedlist.append(&badge_box);
    group_size.add_widget(&boxedlist);
    boxedlist
}
```

**Lines 492-509**: Assembles badge and returns list box.

### get_kernel_branches() Function

```rust
fn get_kernel_branches() -> Result<Vec<KernelBranch>, reqwest::Error> {
    let mut kernel_branches_array: Vec<KernelBranch> = Vec::new();
    let kernel_branch_files_dir = fs::read_dir("/usr/lib/fedora-kernel-manager/kernel_branches")
        .expect("No Kernel json files found");
```

**Lines 512-515**: Reads kernel branch configuration files.

```rust
    for kernel_branch_file in kernel_branch_files_dir {
        let kernel_branch_file_path = kernel_branch_file
            .expect("couldn't change dir entry to path")
            .path();
        let kernel_branch_data =
            fs::read_to_string(kernel_branch_file_path).expect("some json is invalid");
        let branch: serde_json::Value =
            serde_json::from_str(&kernel_branch_data).expect("some json is invalid");
```

**Lines 516-523**: Reads and parses each JSON file.

```rust
        let branch_name = branch["name"].as_str().to_owned().unwrap().to_string();
        let branch_db_url = branch["db_url"].as_str().to_owned().unwrap().to_string();
        let branch_init_script = branch["init_script"]
            .as_str()
            .to_owned()
            .unwrap()
            .to_string();
```

**Lines 524-530**: Extracts fields from JSON.

```rust
        println!("{} {}.", t!("db_downloading"), &branch_name);
        let branch_db =
            reqwest::blocking::get(branch["db_url"].as_str().to_owned().unwrap().to_string())?
                .text()
                .unwrap();
```

**Lines 531-535**: Downloads kernel database from URL.
- Uses blocking HTTP request
- `?` operator propagates errors

```rust
        let branch = KernelBranch {
            name: branch_name,
            db_url: branch_db_url,
            init_script: branch_init_script,
            db: branch_db,
        };
        println!("{} {}", &branch.name, t!("db_download_complete"));
        println!(
            "{} {} {}",
            t!("db_init_script_run_p1"),
            &branch.name,
            t!("db_init_script_run_p2")
        );
        match cmd!("bash", "-c", &branch.init_script).run() {
            Ok(_) => println!("{} {}", &branch.name, t!("db_init_script_successful")),
            _ => println!("{} {}", &branch.name, t!("db_init_script_failed")),
        };
        kernel_branches_array.push(branch)
    }
    Ok(kernel_branches_array)
}
```

**Lines 536-556**: Creates KernelBranch struct, runs init script, and adds to array.
- Init script typically adds repository configuration
- Returns vector of all branches

### get_running_kernel_info() Function

```rust
pub fn get_running_kernel_info() -> RunningKernelInfo {
    let kernel = match Command::new("uname")
        .arg("-r")
        .stdout(Stdio::piped())
        .output()
    {
        Ok(t) => String::from_utf8(t.stdout).unwrap().trim().to_owned(),
        Err(_) => t!("unknown").to_string(),
    };
```

**Lines 558-566**: Gets full kernel release string (e.g., "6.6.5-200.fc39.x86_64").

```rust
   let version = match Command::new("uname")
        .arg("-r")
        .stdout(Stdio::piped())
        .output()
    {
        Ok(t) => {
            // Convert the output to a String and trim any surrounding whitespace
            let output = String::from_utf8(t.stdout).unwrap().trim().to_owned();
            
            // Split at the first dash and take the part before it
            output.split('-').next().unwrap_or(&output).to_owned()
        },
        Err(_) => "unknown".to_string(),
    };
```

**Lines 568-581**: Extracts version number only (e.g., "6.6.5" from "6.6.5-200.fc39.x86_64").
- Splits on dash and takes first part

```rust
    let info = RunningKernelInfo {
        kernel: kernel,
        version: version.clone(),
        // didn't find a way to accurately get this, outside of sched-ext (https://github.com/CachyOS/kernel-manager/blob/develop/src/schedext-window.cpp)
        sched: get_current_scheduler(version),
    };
    info
}
```

**Lines 583-590**: Creates RunningKernelInfo struct with scheduler detection.

### is_scx_kernel() Function

```rust
fn is_scx_kernel() -> bool {
    if Path::new("/sys/kernel/sched_ext").exists() {
        true
    } else {
        false
    }
}
```

**Lines 593-599**: Checks if kernel supports sched_ext.
- `/sys/kernel/sched_ext` exists only if SCX is supported

### get_current_scheduler() Function

```rust
pub fn get_current_scheduler(version: String) -> String {
    if is_scx_kernel() {
        let scx_sched = match fs::read_to_string("/sys/kernel/sched_ext/root/ops") {
            Ok(t) => t,
            Err(_) => {
                return if bore_check() {
                    "BORE".to_string()
                } else if Version::from(&version) >= Version::from("6.6") {
                    "EEVDF?".to_string()
                } else {
                    "CFS?".to_string()
                };
            }
        };
        "sched_ext: ".to_owned() + &scx_sched
```

**Lines 600-614**: Detects current scheduler.
- If SCX: reads from `/sys/kernel/sched_ext/root/ops`
- Otherwise: checks for BORE, then kernel version (EEVDF for 6.6+, CFS for older)

```rust
    } else if bore_check() {
        "BORE".to_string()
    } else if Version::from(&version) >= Version::from("6.6") {
        "EEVDF?".to_string()
    } else {
        "CFS?".to_string()
    }
}
```

**Lines 615-621**: Fallback scheduler detection logic.

### bore_check() Function

```rust
fn bore_check() -> bool {
    let is_bore = match cmd!("sysctl", "-n", "kernel.sched_bore").read() {
        Ok(t) => {
            if t == "1" {
                true
            } else {
                false
            }
        }
        Err(_) => false,
    };
    is_bore
}
```

**Lines 624-635**: Checks if BORE scheduler is active.
- Reads `kernel.sched_bore` sysctl
- Returns true if value is "1"

### create_kernel_badges() Function

```rust
fn create_kernel_badges(
    badge_box: &gtk::Box,
    running_kernel_info: &RunningKernelInfo,
    selected_kernel_branch: &Rc<RefCell<KernelBranch>>,
    theme_changed_action: &gio::SimpleAction,
) {
```

**Lines 638-643**: Creates all kernel information badges.

```rust
    let selected_kernel_branch_clone = selected_kernel_branch.borrow().clone();
    let kernel_badges_size_group = gtk::SizeGroup::new(SizeGroupMode::Both);
    let kernel_badges_size_group0 = gtk::SizeGroup::new(SizeGroupMode::Both);
    let kernel_badges_size_group1 = gtk::SizeGroup::new(SizeGroupMode::Both);
```

**Lines 644-648**: Creates size groups for consistent badge sizing.

```rust
    let json: serde_json::Value =
        serde_json::from_str(&selected_kernel_branch_clone.db).expect("Unable to parse");
    let kernel_version_deter = match json["latest_kernel_version_deter_pkg"].as_str() {
        Some(t) => t,
        _ => "kernel",
    };
```

**Lines 650-656**: Parses branch database JSON and gets version determination package name.

```rust
    let kernel_version =
        match Command::new("/usr/lib/fedora-kernel-manager/scripts/generate_package_info.sh")
            .args(["version", &kernel_version_deter])
            .output()
        {
            Ok(t) => String::from_utf8(t.stdout).unwrap().trim().to_owned(),
            _ => "0.0.0".to_owned(),
        };
```

**Lines 658-665**: Gets latest available kernel version from package manager.

```rust
    let version_css_style = if &running_kernel_info.version == &kernel_version {
        "background-green-bg"
    } else {
        "background-red-bg"
    };
```

**Lines 667-671**: Determines CSS style based on version match.
- Green if running latest, red if outdated

```rust
    while let Some(widget) = badge_box.last_child() {
        badge_box.remove(&widget);
    }
```

**Lines 673-675**: Clears existing badges before recreating.

```rust
    badge_box.append(&create_kernel_badge(
        &t!("kernel_badge_branch_label").to_string(),
        &selected_kernel_branch_clone.name,
        "background-accent-bg",
        &theme_changed_action,
        &kernel_badges_size_group,
        &kernel_badges_size_group0,
        &kernel_badges_size_group1,
    ));
```

**Lines 677-685**: Creates "Kernel Branch" badge.

```rust
    badge_box.append(&create_kernel_badge(
        &t!("kernel_badge_latest_version_label").to_string(),
        &kernel_version,
        "background-accent-bg",
        &theme_changed_action,
        &kernel_badges_size_group,
        &kernel_badges_size_group0,
        &kernel_badges_size_group1,
    ));
```

**Lines 686-694**: Creates "Latest Version" badge.

```rust
    badge_box.append(&create_kernel_badge(
        &t!("kernel_badge_running_version_label").to_string(),
        &running_kernel_info.version,
        &version_css_style,
        &theme_changed_action,
        &kernel_badges_size_group,
        &kernel_badges_size_group0,
        &kernel_badges_size_group1,
    ));
```

**Lines 695-703**: Creates "Running Version" badge (green/red based on match).

```rust
    badge_box.append(&create_kernel_badge(
        &t!("kernel_badge_running_kernel_label").to_string(),
        &running_kernel_info.kernel,
        &version_css_style,
        &theme_changed_action,
        &kernel_badges_size_group,
        &kernel_badges_size_group0,
        &kernel_badges_size_group1,
    ));
```

**Lines 704-712**: Creates "Running Kernel" badge (full release string).

```rust
    badge_box.append(&create_kernel_badge(
        &t!("kernel_badge_running_sched_label").to_string(),
        &running_kernel_info.sched.trim(),
        "background-accent-bg",
        &theme_changed_action,
        &kernel_badges_size_group,
        &kernel_badges_size_group0,
        &kernel_badges_size_group1,
    ));
```

**Lines 713-721**: Creates "Running Sched" badge (scheduler name).
}

### save_branch_config() Function

```rust
fn save_branch_config(branch: &str) {
    let config_path = get_my_home()
        .unwrap()
        .unwrap()
        .join(".config/fedora-kernel-manager");
    match &config_path.exists() {
        true => fs::write(config_path.join("branch"), branch).unwrap(),
        _ => {
            fs::create_dir(&config_path).unwrap();
            fs::write(config_path.join("branch"), branch).unwrap();
        }
    }
}
```

**Lines 724-736**: Saves selected branch to config file.
- Creates `~/.config/fedora-kernel-manager/branch` file
- Creates directory if it doesn't exist

### create_current_sched_badge() Function

```rust
fn create_current_sched_badge(
    badge_box: &adw::Bin,
    running_kernel_info: &RunningKernelInfo,
    theme_changed_action: &gio::SimpleAction,
) {
```

**Lines 738-742**: Creates scheduler badge in separate container.

```rust
    let kernel_badges_size_group = gtk::SizeGroup::new(SizeGroupMode::Both);
    let kernel_badges_size_group0 = gtk::SizeGroup::new(SizeGroupMode::Both);
    let kernel_badges_size_group1 = gtk::SizeGroup::new(SizeGroupMode::Both);
    badge_box.set_child(Some(&crate::content::create_kernel_badge(
        &t!("kernel_badge_running_sched_label").to_string(),
        &running_kernel_info.sched,
        "background-accent-bg",
        &theme_changed_action,
        &kernel_badges_size_group,
        &kernel_badges_size_group0,
        &kernel_badges_size_group1,
    )));
}
```

**Lines 747-759**: Creates scheduler badge and sets as child of Bin widget.

### is_scx_installed() Function

```rust
fn is_scx_installed() -> bool {
    match Command::new("systemctl").args(["status", "scx"]).output() {
        Ok(t) if t.status.code().unwrap() != 4 => true,
        _ => false,
    }
}
```

**Lines 762-767**: Checks if `scx` systemd service is installed and active.
- Exit code 4 means "not found", anything else means service exists

---

## src/kernel_pkg/mod.rs - Kernel Package Management {#srckernel_pkgmodrs}

This module handles the kernel package browsing and installation page.

### kernel_pkg_page() Function

**Lines 15-157**: Main function that builds the kernel package selection page.

**Lines 22-37**: Creates loading dialog shown while parsing package data.

**Lines 39-43**: Creates main container box.

**Lines 45-59**: Creates title label with branch name.

**Lines 61-72**: Creates Tux download icon.

**Lines 74-76**: Combines icon and label in horizontal box.

**Lines 78-85**: Creates search bar with rounded styling.

**Lines 89-97**: Creates list box for kernel packages with size group for consistent button sizing.

**Lines 99-106**: Calls `add_package_rows()` to populate the list (runs in background thread).

**Lines 108-118**: Wraps package list in scrollable viewport.

**Lines 120-149**: Creates bottom bar with back button that navigates to main page.

**Lines 151-154**: Assembles all widgets into main box.

### add_package_rows() Function

**Lines 159-619**: Main function that adds kernel package rows to the list.

**Lines 167-172**: Gets CPU feature level (x86-64-v1 through v4) by parsing ld-linux output.

**Lines 174-178**: Creates channels for package data and completion notification.

**Lines 180-232**: Spawns thread that:
- Parses branch database JSON
- For each kernel in the database:
  - Extracts name, main package, packages list, min_x86_march
  - Gets package version via `generate_package_info.sh` script
  - Gets package description via same script
  - Creates `KernelPackage` struct
  - Sends through channel
- Sends completion signal when done

**Lines 234-572**: Async task that receives packages and creates UI for each:
- **Lines 244-250**: Extracts package information
- **Lines 252-257**: Creates channels for install log and status updates
- **Lines 259-281**: Spawns thread that checks if package is installed (runs every 6 seconds)
- **Lines 283-328**: Creates expandable row widget:
  - Uses custom `KernelPackageRow` widget
  - Adds status icon (shows when installed)
  - Adds description label
  - Adds install/remove buttons
  - Buttons are initially disabled
- **Lines 330-353**: Async task that updates button states based on installation status
- **Lines 355-457**: Creates installation dialog with:
  - Text view for showing dnf output
  - Auto-scrolling to bottom
  - OK and Reboot buttons
- **Lines 459-512**: Connects install button:
  - Shows dialog
  - Spawns thread that calls `kernel_modify()` function
  - Updates dialog based on success/failure
  - Handles reboot option
- **Lines 513-566**: Connects remove button (same logic as install)
- **Lines 567-569**: Only adds row if CPU feature level meets minimum requirement

**Lines 574-583**: Async task that closes loading dialog when parsing complete.

**Lines 585-618**: Implements search filtering (same pattern as branch selection).

### kernel_modify() Function

**Lines 621-643**: Executes package installation/removal.

**Line 621**: Defines bash script template that calls `modify_package.sh` with pkexec (polkit).

**Lines 626-642**: Function implementation:
- Creates pipe for capturing stdout/stderr
- Runs bash script with package name as argument
- Reads output line by line
- Sends each line through channel for display in UI
- Waits for process completion
- Returns Result for error handling

### get_cpu_feature_level() Function

**Lines 645-671**: Detects CPU microarchitecture level.

**Lines 646-651**: Runs `ld-linux-x86-64.so.2 --help` to get CPU info.

**Lines 652-658**: Greps for "(supported, searched)" line.

**Lines 659-670**: Parses output to extract feature level (x86-64-v1, v2, v3, or v4).
- Defaults to v1 if detection fails

---

## src/kernel_package_row/mod.rs - Custom Widget {#srckernel_package_rowmodrs}

This module defines a custom GTK widget that extends `ExpanderRow`.

**Lines 1-2**: Module declaration and import of implementation.

**Lines 6-10**: `glib::wrapper!` macro creates the Rust wrapper for the GObject.
- `KernelPackageRow`: The public type
- Extends `adw::ExpanderRow` (inherits all its functionality)
- Implements standard GTK interfaces

**Lines 12-16**: `new()` method creates a new instance using the builder pattern.

**Lines 19-23**: Implements `Default` trait for convenience.

---

## src/kernel_package_row/imp.rs - Widget Implementation {#srckernel_package_rowimprs}

This module contains the actual implementation of the custom widget.

**Lines 10-15**: Defines the widget's internal state.
- `package`: Property holding the package name (String)
- `RefCell`: Allows mutable access to immutable reference

**Lines 19-24**: Implements `ObjectSubclass` trait.
- `NAME`: GObject type name
- `Type`: Public wrapper type
- `ParentType`: What this widget extends (ExpanderRow)

**Lines 28-52**: Implements `ObjectImpl` trait (GObject functionality).

**Lines 30-33**: Defines custom signals (none currently, but structure is there).

**Lines 34-52**: `constructed()` method called when widget is created:
- Gets reference to the object
- Creates label widget for displaying package name
- Adds label as suffix (right side) of the expander row
- Binds `package` property to label's `label` property
- `sync_create()`: Updates immediately
- `bidirectional()`: Changes propagate both ways

**Lines 55, 60-62**: Implements required traits for widget hierarchy.

---

## src/sched_ext/mod.rs - Scheduler Extension Configuration {#srcsched_extmodrs}

This module handles SCX (Scheduler Extension) configuration.

### Data Structures

**Lines 14-18**: `ScxSchedMode` struct representing a scheduler mode.
- `name`: Mode name (e.g., "gaming", "low_latency")
- `flags`: Command-line flags for the mode

**Lines 20-24**: `ScxSched` struct representing a scheduler.
- `name`: Scheduler name (e.g., "scx_bpfland")
- `modes`: Vector of available modes

**Lines 26-42**: `ScxSched` implementation.
- `get_sched_from_name()`: Static method that loads scheduler from JSON file
- Checks if name starts with "sched_ext:"
- Reads `/usr/lib/pika/kernel-manager/scx_scheds.json` (note: path seems incorrect, should be `/usr/lib/fedora-kernel-manager/scx_scheds.json`)
- Parses JSON and finds matching scheduler

**Lines 44-47**: `ScxSchedulers` struct wrapping the JSON structure.

### sched_ext_page() Function

**Lines 49-261**: Main function building the SCX configuration page.

**Lines 54-58**: Creates main container box.

**Lines 60-72**: Creates Tux settings icon.

**Lines 74-82**: Creates main label.

**Lines 84-93**: Gets current running kernel info and initializes selected scheduler.

**Lines 95-96**: Creates status dialog for showing operation results.

**Lines 98-104**: Creates expander rows for scheduler and mode selection.

**Lines 106-109**: Creates entry row for custom flags.

**Lines 111-126**: Creates list box containing expander rows.

**Lines 128-133**: Adds expandable content to scheduler expander.

**Lines 135-163**: Creates bottom bar with back and apply buttons.

**Lines 164-219**: Connects apply button:
- Gets selected scheduler name and flags
- Calls `change_scx_scheduler()` function
- Shows success/failure dialog

**Lines 222-248**: Spawns thread that checks every 100ms if selection changed:
- Enables apply button if different from current
- Disables if same

**Lines 251-260**: Assembles all widgets.

### scx_sched_expandable() Function

**Lines 263-433**: Builds the scheduler selection UI.

**Lines 269-276**: Creates search bar and list containers.

**Lines 283-285**: Creates null checkbutton for radio grouping.

**Lines 287-290**: Reads scheduler JSON file.

**Lines 292-293**: Creates mode expandable UI.

**Lines 297-386**: Iterates through schedulers and creates UI rows:
- Creates checkbutton and action row for each
- Connects toggle handler that:
  - Updates selected scheduler
  - Populates mode list based on selected scheduler
  - Clears and rebuilds mode checkbuttons
- Auto-selects current scheduler if running

**Lines 387-395**: Wraps in scrollable viewport.

**Lines 397-430**: Implements search filtering.

### scx_sched_modes_expandable() Function

**Lines 435-498**: Builds the mode selection UI (similar structure to scheduler selection).

**Returns**: Tuple of (ListBox container, ListBox for modes, null CheckButton).

### get_current_scx_scheduler() Function

**Lines 500-507**: Reads current SCX scheduler from `/sys/kernel/sched_ext/root/ops`.
- Returns "disabled" if file doesn't exist

### change_scx_scheduler() Function

**Lines 509-518**: Executes SCX scheduler change via polkit.
- Calls `change_scx.sh` script with scheduler name and flags
- Uses `pkexec` for privilege escalation

### get_scx_flags() Function

**Lines 520-554**: Reads current SCX flags from `/etc/default/scx`.
- Parses file line by line
- Finds `SCX_FLAGS=` line
- Returns the value part
- Skips comments and empty lines
- Returns empty string if not found

---

## Data Files and Scripts {#data-files}

### data/resources.gresource.xml

**Lines 1-11**: GResource definition file.
- Lists all files to bundle into binary
- Icons: tux-symbolic.svg, tux-download-symbolic.svg, tux-settings-symbolic.svg
- CSS: style.css
- All compressed and embedded in binary

### data/scripts/generate_package_info.sh

**Lines 1-12**: Script to get package information from dnf.

**Lines 3-6**: If first argument is "version":
- Runs `dnf info` on package
- Extracts Version field
- Returns version string

**Lines 7-11**: If first argument is "description":
- Runs `dnf info` on package
- Extracts Description field (handles multi-line)
- Returns description text

### data/scripts/modify_package.sh

**Lines 1-11**: Script to install or remove packages.

**Line 3**: `set -e`: Exit on any error.

**Lines 5-10**: Checks if package is installed:
- If installed: removes it with `dnf remove -y`
- If not installed: installs it with `dnf install -y`

**Line 11**: Exits with success code.

### data/scripts/change_scx.sh

**Lines 1-15**: Script to change SCX scheduler.

**Lines 3-4**: If scheduler is "scx_disabled":
- Disables and stops scx systemd service

**Lines 5-14**: Otherwise:
- `set -e`: Exit on error
- Updates `/etc/default/scx` file:
  - Changes `SCX_SCHEDULER=` line
  - Removes old `SCX_FLAGS=` line
  - Adds new `SCX_FLAGS=` line if flags provided
- Enables, starts, and restarts scx service

### data/scripts/kernel-cachyos-init.sh

**Lines 1-22**: Initialization script for CachyOS kernel branch.

**Line 3**: Sets flag to track if repos were added.

**Lines 4-5**: Defines repo file paths.

**Lines 7-11**: Checks if main repo file exists, if not:
- Downloads repo file from COPR
- Sets flag to true

**Lines 13-17**: Checks if addons repo file exists, if not:
- Downloads repo file from COPR
- Sets flag to true

**Lines 19-21**: If repos were added:
- Runs `dnf repoquery` to refresh package database

### data/scx_scheds.json

**Lines 1-86**: JSON file defining available SCX schedulers and their modes.

**Structure**:
- Root object with `scx_schedulers` array
- Each scheduler has:
  - `name`: Scheduler identifier
  - `modes`: Array of mode objects (can be empty)
- Each mode has:
  - `name`: Mode identifier
  - `flags`: Command-line flags for the mode

**Schedulers defined**:
- `scx_disabled`: No modes (disables SCX)
- `scx_bpfland`: 4 modes (low_latency, gaming, powersave, server)
- `scx_central`: No modes
- `scx_flash`: No modes
- `scx_lavd`: 2 modes (performance, powersave)
- `scx_layered`: No modes
- `scx_nest`: No modes
- `scx_qmap`: No modes
- `scx_rlfifo`: No modes
- `scx_rustland`: No modes
- `scx_rusty`: No modes
- `scx_userland`: No modes
- `scx_p2dq`: No modes
- `scx_tickless`: No modes

### data/kernel_branches/kernel.json

**Lines 1-5**: Default kernel branch configuration.
- `name`: "kernel (RPM Default)"
- `db_url`: Points to default kernel database
- `init_script`: "true" (no-op command)

### data/kernel_branches/kernel-cachyos.json

**Lines 1-5**: CachyOS kernel branch configuration.
- `name`: "kernel-cachyos"
- `db_url`: Points to CachyOS kernel database
- `init_script`: Runs polkit script to add COPR repos

### data/locales/en_US.json

**Lines 1-74**: English translation strings.
- All UI text is externalized for translation
- Keys follow pattern: `module_component_description`
- Used by `t!()` macro throughout codebase

### data/style.css

**Lines 1-43**: Custom CSS stylesheet.

**Lines 1-3**: `.symbolic-accent-bg`: Sets text color to accent color.

**Lines 5-7**: `.size-20-font`: Sets font size to 20px.

**Lines 9-11**: `.rounded-all-25`: Rounds all corners with 25px radius.

**Lines 13-17**: `.background-accent-bg`: Background with accent color, 10px radius, 5px padding.

**Lines 19-23**: `.background-green-bg`: Green background for success states.

**Lines 25-29**: `.background-red-bg`: Red background (#ff2a03) for error states.

**Lines 31-36**: `.round-border-only-top`: Rounds only top corners (15px).

**Lines 38-43**: `.round-border-only-bottom`: Rounds only bottom corners (15px).

---

## Summary

This application is a comprehensive kernel management tool for Fedora that:

1. **Manages Kernel Branches**: Allows users to select from different kernel sources (default RPM, CachyOS, etc.)

2. **Displays Kernel Information**: Shows current running kernel, latest available version, scheduler, and branch

3. **Installs/Removes Kernels**: Provides UI for browsing and managing kernel packages with dnf integration

4. **Configures SCX Schedulers**: Allows configuration of Scheduler Extension (sched_ext) schedulers for supported kernels

5. **Multi-language Support**: Uses JSON-based translation system

6. **Modern GTK4/Libadwaita UI**: Uses latest GNOME design patterns and components

The codebase follows Rust best practices with:
- Async/await for non-blocking operations
- Thread-based background tasks for heavy operations
- Channel-based communication between threads
- Proper error handling with Result types
- Custom GTK widgets for specialized UI needs
- Resource bundling for embedded assets

Every line of code serves a specific purpose in creating a user-friendly kernel management experience.

