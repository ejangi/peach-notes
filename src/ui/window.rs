use crate::config::AppConfig;
use crate::domain::Note;
use crate::storage::StorageManager;
use crate::ui::editor::Editor;
use crate::ui::preferences::create_preferences_window;
use crate::ui::sidebar::Sidebar;
use gtk4::glib;
use gtk4::prelude::*;
use libadwaita::prelude::*;
use libadwaita::{ApplicationWindow, NavigationPage, NavigationSplitView};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

#[derive(Clone)]
pub struct MainWindow {
    pub window: ApplicationWindow,
    pub sidebar: Sidebar,
    pub editor: Editor,
    pub storage: Rc<RefCell<StorageManager>>,
    pub config: Rc<RefCell<AppConfig>>,
    pub notes: Rc<RefCell<Vec<Note>>>,
    pub selected_note_id: Rc<RefCell<Option<String>>>,
    pub save_version: Rc<Cell<u64>>,
}

impl MainWindow {
    pub fn new(app: &libadwaita::Application) -> Self {
        let config = Rc::new(RefCell::new(AppConfig::load()));
        let storage_mgr = StorageManager::new(&config.borrow().notes_dir)
            .expect("Failed to initialize storage manager");
        let storage = Rc::new(RefCell::new(storage_mgr));

        let sidebar = Sidebar::new();
        let editor = Editor::new();

        let sidebar_page = NavigationPage::new(&sidebar.container, "Notes");
        let editor_page = NavigationPage::new(&editor.container, "Editor");

        let split_view = NavigationSplitView::new();
        split_view.set_sidebar(Some(&sidebar_page));
        split_view.set_content(Some(&editor_page));
        split_view.set_min_sidebar_width(260.0);
        split_view.set_max_sidebar_width(400.0);

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Peach Notes")
            .default_width(config.borrow().window_width)
            .default_height(config.borrow().window_height)
            .content(&split_view)
            .build();

        if config.borrow().is_maximized {
            window.maximize();
        }

        let initial_note_id = config.borrow().last_opened_note.clone();

        let main_win = Self {
            window,
            sidebar,
            editor,
            storage,
            config,
            notes: Rc::new(RefCell::new(Vec::new())),
            selected_note_id: Rc::new(RefCell::new(initial_note_id)),
            save_version: Rc::new(Cell::new(0)),
        };

        main_win.setup_callbacks();
        main_win.refresh_notes();
        main_win
    }

    pub fn refresh_notes(&self) {
        let notes_list = if let Ok(storage) = self.storage.try_borrow() {
            storage.list_notes().unwrap_or_default()
        } else {
            Vec::new()
        };

        if let Ok(mut notes) = self.notes.try_borrow_mut() {
            *notes = notes_list.clone();
        }

        let query = self.sidebar.search_entry.text().to_string();
        self.sidebar.populate_notes(&notes_list, &query);

        let curr_selected = if let Ok(id_ref) = self.selected_note_id.try_borrow() {
            id_ref.clone()
        } else {
            None
        };

        let mut note_found = false;
        if let Some(ref target_id) = curr_selected {
            if notes_list.iter().any(|n| n.id == *target_id) {
                note_found = true;
                self.select_note(target_id);
            }
        }

        if !note_found {
            if let Some(first_note) = notes_list.first() {
                let first_id = first_note.id.clone();
                self.select_note(&first_id);
            } else {
                if let Ok(mut id_ref) = self.selected_note_id.try_borrow_mut() {
                    *id_ref = None;
                }
                if let Ok(mut cfg) = self.config.try_borrow_mut() {
                    cfg.last_opened_note = None;
                    let _ = cfg.save();
                }
                self.editor.clear();
            }
        }
    }

    fn setup_callbacks(&self) {
        let win = self.clone();
        // Search entry query changed
        self.sidebar
            .search_entry
            .connect_search_changed(move |entry| {
                let query = entry.text().to_string();
                if let Ok(notes) = win.notes.try_borrow() {
                    win.sidebar.populate_notes(&notes, &query);
                }
            });

        let win = self.clone();
        // New Note button clicked
        self.sidebar.new_button.connect_clicked(move |_| {
            if let Ok(storage) = win.storage.try_borrow() {
                if let Ok(new_note) = storage.create_note("New Note", "") {
                    win.refresh_notes();
                    win.select_note(&new_note.id);
                    win.editor.text_view.grab_focus();
                }
            }
        });

        let win = self.clone();
        // Preferences button clicked
        self.sidebar.pref_button.connect_clicked(move |_| {
            let win_clone = win.clone();
            let pref_dialog =
                create_preferences_window(&win.window, win.config.clone(), move |new_path| {
                    if let Ok(new_storage) = StorageManager::new(new_path) {
                        if let Ok(mut s) = win_clone.storage.try_borrow_mut() {
                            *s = new_storage;
                            if let Ok(mut cfg) = win_clone.config.try_borrow_mut() {
                                cfg.last_opened_note = None;
                                let _ = cfg.save();
                            }
                            if let Ok(mut selected) = win_clone.selected_note_id.try_borrow_mut() {
                                *selected = None;
                            }
                            win_clone.editor.clear();
                            win_clone.refresh_notes();
                        }
                    }
                });
            pref_dialog.present();
        });

        let win = self.clone();
        // ListBox row selection
        self.sidebar.list_box.connect_row_selected(move |_, row| {
            if win.sidebar.is_populating.get() {
                return;
            }
            if let Some(row_widget) = row {
                let note_id = row_widget.widget_name().to_string();
                win.select_note(&note_id);
            }
        });

        let win = self.clone();
        // Editor buffer text change -> Debounced autosave (500ms)
        self.editor.text_buffer.connect_changed(move |_| {
            if win.editor.is_loading.get() {
                return;
            }

            let version = win.save_version.get() + 1;
            win.save_version.set(version);

            let win_inner = win.clone();
            glib::timeout_add_local_once(Duration::from_millis(500), move || {
                if win_inner.save_version.get() == version {
                    win_inner.save_current_note();
                }
            });
        });

        let win = self.clone();
        // Delete Note button
        self.editor.delete_button.connect_clicked(move |_| {
            let note_opt = if let Ok(n) = win.editor.current_note.try_borrow() {
                n.clone()
            } else {
                None
            };
            if let Some(note) = note_opt {
                if let Ok(storage) = win.storage.try_borrow() {
                    let _ = storage.delete_note(&note);
                    if let Ok(mut cfg) = win.config.try_borrow_mut() {
                        cfg.last_opened_note = None;
                        let _ = cfg.save();
                    }
                    if let Ok(mut selected) = win.selected_note_id.try_borrow_mut() {
                        *selected = None;
                    }
                    win.editor.clear();
                    win.refresh_notes();
                }
            }
        });

        let win = self.clone();
        let window_config_version = Rc::new(Cell::new(0u64));

        let schedule_window_state_save = move |window: &ApplicationWindow| {
            let is_maximized = window.is_maximized();
            let width = window.default_width();
            let height = window.default_height();

            let version = window_config_version.get() + 1;
            window_config_version.set(version);

            let win_inner = win.clone();
            let version_inner = window_config_version.clone();

            glib::timeout_add_local_once(Duration::from_millis(500), move || {
                if version_inner.get() == version {
                    if let Ok(mut cfg) = win_inner.config.try_borrow_mut() {
                        let mut changed = false;
                        if cfg.is_maximized != is_maximized {
                            cfg.is_maximized = is_maximized;
                            changed = true;
                        }
                        if !is_maximized {
                            if width > 0 && cfg.window_width != width {
                                cfg.window_width = width;
                                changed = true;
                            }
                            if height > 0 && cfg.window_height != height {
                                cfg.window_height = height;
                                changed = true;
                            }
                        }
                        if changed {
                            let _ = cfg.save();
                        }
                    }
                }
            });
        };

        let save_cb1 = schedule_window_state_save.clone();
        self.window.connect_default_width_notify(move |w| save_cb1(w));

        let save_cb2 = schedule_window_state_save.clone();
        self.window.connect_default_height_notify(move |w| save_cb2(w));

        let save_cb3 = schedule_window_state_save;
        self.window.connect_maximized_notify(move |w| save_cb3(w));

        let win = self.clone();
        // Listen for system theme dark/light mode toggle
        libadwaita::StyleManager::default().connect_dark_notify(move |_| {
            crate::markdown::setup_text_buffer_tags(&win.editor.text_buffer);
            let note_opt = if let Ok(n) = win.editor.current_note.try_borrow() {
                n.clone()
            } else {
                None
            };
            if let Some(note) = note_opt {
                let notes_dir = if let Ok(storage) = win.storage.try_borrow() {
                    Some(storage.notes_dir().to_path_buf())
                } else {
                    None
                };
                if let Some(dir) = notes_dir {
                    win.editor.load_note(note, Some(&dir));
                }
            }
        });

        let win = self.clone();
        // File Drag & Drop onto Editor
        self.editor.setup_drop_target(move |dropped_files, x, y| {
            let note_opt = if let Ok(n) = win.editor.current_note.try_borrow() {
                n.clone()
            } else {
                None
            };

            if let Some(note) = note_opt {
                if let Ok(storage) = win.storage.try_borrow() {
                    if let Ok(assets_dir) = storage.ensure_assets_dir(&note.title) {
                        let assets_dir_name = Note::assets_dir_name(&note.title);

                        let (win_x, win_y) = win.editor.text_view.window_to_buffer_coords(
                            gtk4::TextWindowType::Widget,
                            x as i32,
                            y as i32,
                        );
                        let mut drop_iter =
                            match win.editor.text_view.iter_at_location(win_x, win_y) {
                                Some(iter) => iter,
                                None => win.editor.text_buffer.end_iter(),
                            };

                        for source_path in dropped_files {
                            let orig_filename = source_path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("file");

                            let mut target_filename = orig_filename.to_string();
                            let mut target_path = assets_dir.join(&target_filename);

                            let mut counter = 1;
                            while target_path.exists() {
                                let stem = std::path::Path::new(orig_filename)
                                    .file_stem()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("file");
                                let ext = std::path::Path::new(orig_filename)
                                    .extension()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("");
                                if ext.is_empty() {
                                    target_filename = format!("{} {}", stem, counter);
                                } else {
                                    target_filename = format!("{} {}.{}", stem, counter, ext);
                                }
                                target_path = assets_dir.join(&target_filename);
                                counter += 1;
                            }

                            if std::fs::copy(&source_path, &target_path).is_ok() {
                                let relative_url =
                                    format!("{}/{}", assets_dir_name, target_filename);
                                let notes_dir_path = storage.notes_dir().to_path_buf();

                                if Note::is_image_file(&source_path) {
                                    let win_clone = win.clone();
                                    let rel_url_clone = relative_url.clone();
                                    let drop_offset = drop_iter.offset();

                                    let dialog = libadwaita::MessageDialog::builder()
                                        .transient_for(&win.window)
                                        .heading("Add Image Caption")
                                        .body(format!("Attached: {}", target_filename))
                                        .build();

                                    let entry = gtk4::Entry::builder()
                                        .placeholder_text("Caption (optional)")
                                        .margin_top(8)
                                        .margin_bottom(8)
                                        .margin_start(12)
                                        .margin_end(12)
                                        .build();

                                    dialog.set_extra_child(Some(&entry));
                                    dialog.add_response("cancel", "Skip");
                                    dialog.add_response("add", "Add Image");
                                    dialog.set_response_appearance(
                                        "add",
                                        libadwaita::ResponseAppearance::Suggested,
                                    );
                                    dialog.set_default_response(Some("add"));

                                    let entry_clone = entry.clone();
                                    dialog.connect_response(None, move |_, response| {
                                        let caption = if response == "add" {
                                            entry_clone.text().to_string()
                                        } else {
                                            String::new()
                                        };

                                        let mut iter = win_clone
                                            .editor
                                            .text_buffer
                                            .iter_at_offset(drop_offset);
                                        crate::markdown::render_image_widget(
                                            &win_clone.editor.text_buffer,
                                            &win_clone.editor.text_view,
                                            &mut iter,
                                            &rel_url_clone,
                                            &caption,
                                            Some(&notes_dir_path),
                                        );
                                        win_clone.editor.text_view.queue_resize();
                                        win_clone.editor.text_view.queue_draw();
                                        win_clone.save_current_note();
                                    });

                                    dialog.present();
                                } else {
                                    crate::markdown::render_attachment_widget(
                                        &win.editor.text_buffer,
                                        &win.editor.text_view,
                                        &mut drop_iter,
                                        &relative_url,
                                        &target_filename,
                                        Some(&notes_dir_path),
                                    );
                                    win.editor.text_view.queue_resize();
                                    win.editor.text_view.queue_draw();
                                    win.save_current_note();
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    fn select_note(&self, note_id: &str) {
        if let Ok(mut selected_id) = self.selected_note_id.try_borrow_mut() {
            *selected_id = Some(note_id.to_string());
        }

        if let Ok(mut cfg) = self.config.try_borrow_mut() {
            if cfg.last_opened_note.as_deref() != Some(note_id) {
                cfg.last_opened_note = Some(note_id.to_string());
                let _ = cfg.save();
            }
        }

        let mut child = self.sidebar.list_box.first_child();
        while let Some(row_widget) = child {
            if row_widget.widget_name().as_str() == note_id {
                if let Ok(row) = row_widget.downcast::<gtk4::ListBoxRow>() {
                    if self.sidebar.list_box.selected_row().as_ref() != Some(&row) {
                        self.sidebar.list_box.select_row(Some(&row));
                    }
                }
                break;
            }
            child = row_widget.next_sibling();
        }
        let found_note = if let Ok(notes) = self.notes.try_borrow() {
            notes.iter().find(|n| n.id == note_id).cloned()
        } else {
            None
        };

        if let Some(note) = found_note {
            let notes_dir = if let Ok(storage) = self.storage.try_borrow() {
                Some(storage.notes_dir().to_path_buf())
            } else {
                None
            };
            if let Some(dir) = notes_dir {
                self.editor.load_note(note, Some(&dir));
            }
        }
    }

    fn save_current_note(&self) {
        if self.editor.is_loading.get() {
            return;
        }

        let note_opt = if let Ok(n) = self.editor.current_note.try_borrow() {
            n.clone()
        } else {
            return;
        };

        if let Some(mut note) = note_opt {
            let new_content = self.editor.get_serialized_content();
            if new_content != note.content {
                let saved = if let Ok(storage) = self.storage.try_borrow() {
                    storage.save_note(&mut note, &new_content).is_ok()
                } else {
                    false
                };

                if saved {
                    if let Ok(mut curr) = self.editor.current_note.try_borrow_mut() {
                        *curr = Some(note);
                    }
                    // Refresh sidebar list items without re-loading or replacing active editor buffer
                    if let Ok(storage) = self.storage.try_borrow() {
                        if let Ok(notes_list) = storage.list_notes() {
                            if let Ok(mut notes) = self.notes.try_borrow_mut() {
                                *notes = notes_list.clone();
                            }
                            let query = self.sidebar.search_entry.text().to_string();
                            self.sidebar.populate_notes(&notes_list, &query);
                        }
                    }
                }
            }
        }
    }
}
