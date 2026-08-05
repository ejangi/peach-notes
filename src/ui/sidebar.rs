use crate::domain::Note;
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Label, ListBox, ListBoxRow, Orientation, SearchEntry};
use libadwaita::HeaderBar;

use std::cell::Cell;
use std::rc::Rc;

#[derive(Clone)]
pub struct Sidebar {
    pub container: GtkBox,
    pub search_entry: SearchEntry,
    pub list_box: ListBox,
    pub new_button: gtk4::Button,
    pub pref_button: gtk4::Button,
    pub is_populating: Rc<Cell<bool>>,
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new()
    }
}

impl Sidebar {
    pub fn new() -> Self {
        let container = GtkBox::new(Orientation::Vertical, 0);

        // HeaderBar
        let header_bar = HeaderBar::new();

        let new_button = gtk4::Button::builder()
            .icon_name("document-new-symbolic")
            .tooltip_text("New Note (Ctrl+N)")
            .build();

        let pref_button = gtk4::Button::builder()
            .icon_name("emblem-system-symbolic")
            .tooltip_text("Preferences")
            .build();

        header_bar.pack_start(&new_button);
        header_bar.pack_end(&pref_button);
        header_bar.set_title_widget(Some(&Label::new(Some("Notes"))));

        container.append(&header_bar);

        // Search bar
        let search_box = GtkBox::new(Orientation::Horizontal, 0);
        search_box.set_margin_start(12);
        search_box.set_margin_end(12);
        search_box.set_margin_top(8);
        search_box.set_margin_bottom(8);

        let search_entry = SearchEntry::new();
        search_entry.set_hexpand(true);
        search_entry.set_placeholder_text(Some("Search notes..."));
        search_box.append(&search_entry);

        container.append(&search_box);

        // List Box
        let list_box = ListBox::new();
        list_box.add_css_class("navigation-sidebar");
        list_box.set_selection_mode(gtk4::SelectionMode::Single);

        let scrolled_window = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&list_box)
            .vexpand(true)
            .build();

        container.append(&scrolled_window);
        let is_populating = Rc::new(Cell::new(false));

        Self {
            container,
            search_entry,
            list_box,
            new_button,
            pref_button,
            is_populating,
        }
    }

    pub fn populate_notes(&self, notes: &[Note], filter: &str) {
        self.is_populating.set(true);

        // Clear existing rows
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }

        let query = filter.to_lowercase();

        for note in notes {
            if !query.is_empty() {
                let matches_title = note.title.to_lowercase().contains(&query);
                let matches_content = note.content.to_lowercase().contains(&query);
                if !matches_title && !matches_content {
                    continue;
                }
            }

            let row = ListBoxRow::new();
            let item_box = GtkBox::new(Orientation::Vertical, 4);
            item_box.set_margin_start(12);
            item_box.set_margin_end(12);
            item_box.set_margin_top(10);
            item_box.set_margin_bottom(10);

            let title_label = Label::builder()
                .label(&note.title)
                .halign(Align::Start)
                .css_classes(vec!["heading".to_string()])
                .ellipsize(pango::EllipsizeMode::End)
                .build();

            let preview_label = Label::builder()
                .label(note.preview())
                .halign(Align::Start)
                .css_classes(vec!["dim-label".to_string(), "caption".to_string()])
                .ellipsize(pango::EllipsizeMode::End)
                .build();

            item_box.append(&title_label);
            item_box.append(&preview_label);
            row.set_child(Some(&item_box));

            // Store note ID as row widget name
            row.set_widget_name(&note.id);

            self.list_box.append(&row);
        }

        self.is_populating.set(false);
    }
}
