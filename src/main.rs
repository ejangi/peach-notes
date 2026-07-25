pub mod config;
pub mod domain;
pub mod markdown;
pub mod storage;
pub mod ui;

use gtk4::gdk::Display;
use gtk4::prelude::*;
use gtk4::{CssProvider, IconTheme};
use libadwaita::Application;
use std::path::Path;
use ui::MainWindow;

fn main() -> glib::ExitCode {
    env_logger::init();
    libadwaita::init().expect("Failed to initialize Libadwaita");

    let app = Application::builder()
        .application_id("org.gnome.PeachNotes")
        .build();

    app.connect_startup(|_| {
        load_css();
        setup_icons();
    });

    app.connect_activate(|app| {
        let main_window = MainWindow::new(app);
        main_window.window.present();
    });

    app.run()
}

fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_string(
        "
        .navigation-sidebar {
            background-color: @sidebar_bg_color;
        }

        textview text {
            font-size: 15px;
            line-height: 1.5;
        }

        .heading {
            font-weight: bold;
            font-size: 15px;
        }

        .caption {
            font-size: 13px;
        }

        .code-block-container {
            background-color: alpha(@window_fg_color, 0.08);
            border-radius: 8px;
            padding: 12px 16px;
            margin-top: 8px;
            margin-bottom: 12px;
            border: 1px solid alpha(@window_fg_color, 0.12);
        }

        .code-block-text {
            font-family: Monospace, monospace;
            font-size: 14px;
            line-height: 1.5;
            color: @window_fg_color;
        }

        .note-image-container {
            margin-top: 12px;
            margin-bottom: 16px;
            border-radius: 8px;
        }

        .note-image-caption {
            font-size: 13px;
            color: alpha(@window_fg_color, 0.6);
            margin-top: 6px;
        }

        .note-image-missing {
            font-size: 13px;
            font-style: italic;
            color: alpha(@window_fg_color, 0.5);
            padding: 12px;
            background-color: alpha(@window_fg_color, 0.05);
            border-radius: 6px;
        }

        .note-attachment-container {
            background-color: alpha(@window_fg_color, 0.06);
            border-radius: 10px;
            padding: 10px 14px;
            margin-top: 8px;
            margin-bottom: 12px;
            border: 1px solid alpha(@window_fg_color, 0.12);
        }

        .note-attachment-title {
            font-weight: bold;
            font-size: 14px;
        }

        .note-attachment-size {
            font-size: 12px;
            color: alpha(@window_fg_color, 0.6);
        }

        popover.selection-toolbar,
        popover.selection-toolbar > contents,
        popover.selection-toolbar contents {
            background-color: transparent;
            background-image: none;
            box-shadow: none;
            border: none;
            padding: 0;
        }

        .selection-toolbar-box {
            background-color: #1e1e24;
            color: #ffffff;
            border-radius: 20px;
            padding: 4px 6px;
            border: 1px solid rgba(255, 255, 255, 0.15);
            box-shadow: 0 8px 20px rgba(0, 0, 0, 0.5);
        }

        .selection-toolbar-box button {
            color: #ffffff;
            border-radius: 12px;
            min-width: 32px;
            min-height: 32px;
            padding: 4px 8px;
            font-weight: bold;
        }

        .selection-toolbar-box button:hover {
            background-color: rgba(255, 255, 255, 0.18);
        }

        .selection-toolbar-box button:active {
            background-color: rgba(255, 255, 255, 0.30);
        }
        ",
    );

    if let Some(display) = Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn setup_icons() {
    if let Some(display) = Display::default() {
        let icon_theme = IconTheme::for_display(&display);
        let icons_dir = Path::new("assets/icons");
        if icons_dir.exists() {
            icon_theme.add_search_path(icons_dir);
        }
    }
    gtk4::Window::set_default_icon_name("org.gnome.PeachNotes");
}
