use crate::config::AppConfig;
use gtk4::prelude::*;
use gtk4::{Align, Button, FileDialog};
use libadwaita::prelude::*;
use libadwaita::{ActionRow, PreferencesGroup, PreferencesPage, PreferencesWindow};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

pub fn create_preferences_window<F>(
    parent: &libadwaita::ApplicationWindow,
    current_config: Rc<RefCell<AppConfig>>,
    on_dir_changed: F,
) -> PreferencesWindow
where
    F: Fn(PathBuf) + 'static,
{
    let window = PreferencesWindow::builder()
        .transient_for(parent)
        .modal(true)
        .title("Preferences")
        .build();

    let page = PreferencesPage::new();
    page.set_title("Storage");
    page.set_icon_name(Some("folder-symbolic"));

    let group = PreferencesGroup::new();
    group.set_title("Notes Directory");
    group.set_description(Some(
        "Select the folder on disk where Markdown note files are stored.",
    ));

    let initial_path = current_config.borrow().notes_dir.clone();
    let row = ActionRow::new();
    row.set_title("Notes Folder");
    row.set_subtitle(&initial_path.to_string_lossy());

    let select_button = Button::builder()
        .label("Select Folder...")
        .valign(Align::Center)
        .build();

    let parent_win = parent.clone();
    let config_ref = current_config.clone();
    let row_clone = row.clone();
    let on_dir_changed = Rc::new(on_dir_changed);

    select_button.connect_clicked(move |_| {
        let dialog = FileDialog::builder()
            .title("Select Notes Directory")
            .accept_label("Select")
            .build();

        let config_ref = config_ref.clone();
        let row_clone = row_clone.clone();
        let on_dir_changed = on_dir_changed.clone();

        dialog.select_folder(Some(&parent_win), gio::Cancellable::NONE, move |result| {
            if let Ok(folder) = result {
                if let Some(path) = folder.path() {
                    config_ref.borrow_mut().notes_dir = path.clone();
                    let _ = config_ref.borrow().save();
                    row_clone.set_subtitle(&path.to_string_lossy());
                    on_dir_changed(path);
                }
            }
        });
    });

    row.add_suffix(&select_button);
    group.add(&row);
    page.add(&group);
    window.add(&page);

    window
}
