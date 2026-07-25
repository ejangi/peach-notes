use crate::domain::Note;
use crate::markdown::{
    parse_markdown_to_buffer, serialize_buffer_to_markdown, setup_text_buffer_tags,
};
use gtk4::prelude::*;
use gtk4::{glib, graphene};
use gtk4::{
    Box as GtkBox, Button, Orientation, Overlay, Popover, PositionType, ScrolledWindow, TextBuffer,
    TextView,
};
use libadwaita::prelude::*;
use libadwaita::HeaderBar;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

#[derive(Clone)]
pub struct Editor {
    pub container: GtkBox,
    pub overlay: Overlay,
    pub text_view: TextView,
    pub text_buffer: TextBuffer,
    pub header_bar: HeaderBar,
    pub delete_button: Button,
    pub selection_popover: Popover,
    pub current_note: Rc<RefCell<Option<Note>>>,
    pub frontmatter: Rc<RefCell<Option<String>>>,
    pub is_loading: Rc<Cell<bool>>,
}

impl Editor {
    pub fn new() -> Self {
        let container = GtkBox::new(Orientation::Vertical, 0);

        // HeaderBar for Editor Pane - Minimalist Apple Notes style
        let header_bar = HeaderBar::new();
        header_bar.add_css_class("flat");

        let delete_button = Button::builder()
            .icon_name("user-trash-symbolic")
            .tooltip_text("Delete Note")
            .css_classes(vec!["destructive-action".to_string()])
            .build();

        header_bar.pack_end(&delete_button);
        container.append(&header_bar);

        // TextView & Buffer inside ScrolledWindow
        let text_buffer = TextBuffer::new(None);
        setup_text_buffer_tags(&text_buffer);

        let text_view = TextView::builder()
            .buffer(&text_buffer)
            .wrap_mode(gtk4::WrapMode::Word)
            .left_margin(24)
            .right_margin(24)
            .top_margin(24)
            .bottom_margin(24)
            .monospace(false)
            .vexpand(true)
            .hexpand(true)
            .build();

        text_view.add_css_class("card");

        let buf_clone_trigger = text_buffer.clone();
        text_buffer.connect_insert_text(move |_buf, _iter, text| {
            if text.contains(' ') {
                let buf = buf_clone_trigger.clone();
                glib::idle_add_local_once(move || {
                    let cursor_offset = buf.cursor_position();
                    let cursor_iter = buf.iter_at_offset(cursor_offset);
                    let mut line_start = cursor_iter;
                    line_start.set_line_offset(0);

                    let line_prefix = buf.text(&line_start, &cursor_iter, true).to_string();
                    if line_prefix == "* " || line_prefix == "- " {
                        let start_offset = line_start.offset();
                        let mut del_start = buf.iter_at_offset(start_offset);
                        let mut del_end = buf.iter_at_offset(cursor_offset);
                        buf.delete(&mut del_start, &mut del_end);

                        let mut ins_iter = buf.iter_at_offset(start_offset);
                        buf.insert(&mut ins_iter, "• ");

                        let tag_start = buf.iter_at_offset(start_offset);
                        let tag_end = buf.iter_at_offset(start_offset + 2);
                        if let Some(tag) = buf.tag_table().lookup("bullet-list") {
                            buf.apply_tag(&tag, &tag_start, &tag_end);
                        }
                    }
                });
            }
        });

        let buf_clone_enter = text_buffer.clone();
        let key_controller = gtk4::EventControllerKey::new();
        key_controller.connect_key_pressed(move |_, keyval, _keycode, _state| {
            if keyval == gdk4::Key::Return || keyval == gdk4::Key::KP_Enter {
                let cursor_offset = buf_clone_enter.cursor_position();
                let cursor_iter = buf_clone_enter.iter_at_offset(cursor_offset);
                let mut line_start = cursor_iter;
                line_start.set_line_offset(0);
                let mut line_end = cursor_iter;
                if !line_end.ends_line() {
                    line_end.forward_to_line_end();
                }

                let line_text = buf_clone_enter
                    .text(&line_start, &line_end, true)
                    .to_string();
                let is_bullet_line = line_text.starts_with("• ")
                    || line_start
                        .tags()
                        .iter()
                        .any(|t| t.name().as_deref() == Some("bullet-list"));

                if is_bullet_line {
                    if line_text == "• " || line_text == "•" || line_text.trim().is_empty() {
                        let mut del_start = line_start;
                        let mut del_end = line_end;
                        buf_clone_enter.delete(&mut del_start, &mut del_end);
                        return glib::Propagation::Stop;
                    } else {
                        let mut ins_iter = buf_clone_enter.iter_at_offset(cursor_offset);
                        buf_clone_enter.insert(&mut ins_iter, "\n• ");
                        let new_offset = buf_clone_enter.cursor_position();
                        let tag_start = buf_clone_enter.iter_at_offset(new_offset - 2);
                        let tag_end = buf_clone_enter.iter_at_offset(new_offset);
                        if let Some(tag) = buf_clone_enter.tag_table().lookup("bullet-list") {
                            buf_clone_enter.apply_tag(&tag, &tag_start, &tag_end);
                        }
                        return glib::Propagation::Stop;
                    }
                }
            }
            glib::Propagation::Proceed
        });
        text_view.add_controller(key_controller);

        let scrolled_window = ScrolledWindow::builder()
            .child(&text_view)
            .vexpand(true)
            .hexpand(true)
            .build();

        // GTK Overlay container for highest Z-index floating selection toolbar
        let overlay = Overlay::new();
        overlay.set_child(Some(&scrolled_window));

        container.append(&overlay);

        // Floating Selection Tooltip Popover (Apple Notes style)
        let selection_popover = Popover::builder()
            .autohide(false)
            .position(PositionType::Top)
            .has_arrow(false)
            .css_classes(vec!["selection-toolbar".to_string()])
            .build();

        selection_popover.set_parent(&overlay);

        let popover_box = GtkBox::new(Orientation::Horizontal, 2);
        popover_box.add_css_class("selection-toolbar-box");
        popover_box.set_margin_top(4);
        popover_box.set_margin_bottom(4);
        popover_box.set_margin_start(4);
        popover_box.set_margin_end(4);

        let bold_btn = Button::builder()
            .label("B")
            .tooltip_text("Bold")
            .css_classes(vec!["flat".to_string(), "bold".to_string()])
            .build();

        let italic_btn = Button::builder()
            .label("I")
            .tooltip_text("Italic")
            .css_classes(vec!["flat".to_string(), "italic".to_string()])
            .build();

        let strike_btn = Button::builder()
            .label("S")
            .tooltip_text("Strikethrough")
            .css_classes(vec!["flat".to_string(), "strikethrough".to_string()])
            .build();

        let code_btn = Button::builder()
            .label("</>")
            .tooltip_text("Code")
            .css_classes(vec!["flat".to_string()])
            .build();

        let h1_btn = Button::builder()
            .label("H1")
            .tooltip_text("Heading 1")
            .css_classes(vec!["flat".to_string()])
            .build();

        let h2_btn = Button::builder()
            .label("H2")
            .tooltip_text("Heading 2")
            .css_classes(vec!["flat".to_string()])
            .build();

        let h3_btn = Button::builder()
            .label("H3")
            .tooltip_text("Heading 3")
            .css_classes(vec!["flat".to_string()])
            .build();

        let link_btn = Button::builder()
            .label("🔗")
            .tooltip_text("Link (URL)")
            .css_classes(vec!["flat".to_string()])
            .build();

        popover_box.append(&bold_btn);
        popover_box.append(&italic_btn);
        popover_box.append(&strike_btn);
        popover_box.append(&code_btn);
        popover_box.append(&h1_btn);
        popover_box.append(&h2_btn);
        popover_box.append(&h3_btn);
        popover_box.append(&link_btn);

        selection_popover.set_child(Some(&popover_box));

        let current_note = Rc::new(RefCell::new(None));
        let frontmatter = Rc::new(RefCell::new(None));
        let is_loading = Rc::new(Cell::new(false));

        let editor = Self {
            container,
            overlay,
            text_view,
            text_buffer,
            header_bar,
            delete_button,
            selection_popover,
            current_note,
            frontmatter,
            is_loading,
        };

        editor.setup_selection_callbacks(
            bold_btn, italic_btn, strike_btn, code_btn, h1_btn, h2_btn, h3_btn, link_btn,
        );

        let buf_clone = editor.text_buffer.clone();
        scrolled_window
            .hadjustment()
            .connect_page_size_notify(move |adj| {
                let width = adj.page_size() as i32;
                if width > 0 {
                    let max_w = (width - 48).max(100);
                    crate::markdown::resize_all_images_in_buffer(&buf_clone, max_w);
                }
            });

        editor
    }

    pub fn load_note(&self, note: Note, notes_dir: Option<&std::path::Path>) {
        self.is_loading.set(true);
        self.selection_popover.popdown();
        if let Ok(mut curr) = self.current_note.try_borrow_mut() {
            *curr = Some(note.clone());
        }
        let fm =
            parse_markdown_to_buffer(&note.content, &self.text_buffer, &self.text_view, notes_dir);
        if let Ok(mut fm_ref) = self.frontmatter.try_borrow_mut() {
            *fm_ref = fm;
        }
        self.container.set_sensitive(true);
        self.is_loading.set(false);
    }

    pub fn clear(&self) {
        self.is_loading.set(true);
        self.selection_popover.popdown();
        if let Ok(mut curr) = self.current_note.try_borrow_mut() {
            *curr = None;
        }
        if let Ok(mut fm_ref) = self.frontmatter.try_borrow_mut() {
            *fm_ref = None;
        }

        self.text_buffer.set_text("");
        self.container.set_sensitive(false);
        self.is_loading.set(false);
    }

    pub fn get_serialized_content(&self) -> String {
        let fm = if let Ok(f) = self.frontmatter.try_borrow() {
            f.clone()
        } else {
            None
        };
        serialize_buffer_to_markdown(&self.text_buffer, fm.as_deref())
    }

    fn setup_selection_callbacks(
        &self,
        bold_btn: Button,
        italic_btn: Button,
        strike_btn: Button,
        code_btn: Button,
        h1_btn: Button,
        h2_btn: Button,
        h3_btn: Button,
        link_btn: Button,
    ) {
        let view = self.text_view.clone();
        let overlay = self.overlay.clone();
        let popover = self.selection_popover.clone();
        let loading = self.is_loading.clone();

        // Show/hide floating selection popover anchored to active text selection on top-level Overlay
        self.text_buffer.connect_mark_set(move |buf, _iter, mark| {
            if loading.get() {
                return;
            }

            let mark_name = mark.name();
            if mark_name.as_deref() == Some("insert")
                || mark_name.as_deref() == Some("selection_bound")
            {
                if let Some((start, end)) = buf.selection_bounds() {
                    if start.offset() != end.offset() {
                        let view_c = view.clone();
                        let overlay_c = overlay.clone();
                        let popover_c = popover.clone();
                        glib::idle_add_local_once(move || {
                            if let Some((s, e)) = view_c.buffer().selection_bounds() {
                                if s.offset() != e.offset() {
                                    let start_loc = view_c.iter_location(&s);
                                    let (win_x, win_y) = view_c.buffer_to_window_coords(
                                        gtk4::TextWindowType::Widget,
                                        start_loc.x(),
                                        start_loc.y(),
                                    );

                                    let (overlay_x, overlay_y) = view_c
                                        .compute_point(
                                            &overlay_c,
                                            &graphene::Point::new(win_x as f32, win_y as f32),
                                        )
                                        .map(|p| (p.x() as f64, p.y() as f64))
                                        .unwrap_or((win_x as f64, win_y as f64));

                                    let editor_width = view_c.width() as f64;
                                    let third = (editor_width / 3.0).max(100.0);

                                    let target_x = if overlay_x < third {
                                        // Left third: shift target point right so popover extends rightward into editor
                                        (overlay_x + 90.0).min(editor_width - 10.0)
                                    } else if overlay_x > 2.0 * third {
                                        // Right third: shift target point left so popover extends leftward into editor
                                        (overlay_x - 90.0).max(10.0)
                                    } else {
                                        // Center third: center popover over selection
                                        overlay_x
                                    };

                                    let rect = gdk4::Rectangle::new(
                                        target_x as i32,
                                        (overlay_y - 35.0).max(0.0) as i32,
                                        1,
                                        1,
                                    );

                                    popover_c.set_pointing_to(Some(&rect));
                                    popover_c.popup();
                                    return;
                                }
                            }
                            popover_c.popdown();
                        });
                        return;
                    }
                }
                popover.popdown();
            }
        });

        // Format button actions
        let buffer = self.text_buffer.clone();
        bold_btn.connect_clicked(move |_| {
            Self::toggle_tag(&buffer, "bold");
        });

        let buffer = self.text_buffer.clone();
        italic_btn.connect_clicked(move |_| {
            Self::toggle_tag(&buffer, "italic");
        });

        let buffer = self.text_buffer.clone();
        strike_btn.connect_clicked(move |_| {
            Self::toggle_tag(&buffer, "strikethrough");
        });

        let buffer = self.text_buffer.clone();
        code_btn.connect_clicked(move |_| {
            Self::toggle_tag(&buffer, "monospace");
        });

        let buffer = self.text_buffer.clone();
        h1_btn.connect_clicked(move |_| {
            Self::apply_line_tag(&buffer, "heading-1");
        });

        let buffer = self.text_buffer.clone();
        h2_btn.connect_clicked(move |_| {
            Self::apply_line_tag(&buffer, "heading-2");
        });

        let buffer = self.text_buffer.clone();
        h3_btn.connect_clicked(move |_| {
            Self::apply_line_tag(&buffer, "heading-3");
        });

        let buffer = self.text_buffer.clone();
        let view = self.text_view.clone();

        link_btn.connect_clicked(move |_| {
            let parent_win = view.root().and_downcast::<gtk4::Window>();

            let dialog = libadwaita::MessageDialog::builder()
                .heading("Insert Link")
                .body("Enter URL for the selected text:")
                .build();

            if let Some(win) = parent_win.as_ref() {
                dialog.set_transient_for(Some(win));
            }

            let entry = gtk4::Entry::builder()
                .text("https://")
                .placeholder_text("https://example.com")
                .activates_default(true)
                .build();

            dialog.set_extra_child(Some(&entry));
            dialog.add_response("cancel", "Cancel");
            dialog.add_response("insert", "Insert Link");
            dialog.set_response_appearance("insert", libadwaita::ResponseAppearance::Suggested);
            dialog.set_default_response(Some("insert"));

            let buffer_inner = buffer.clone();
            let entry_inner = entry.clone();

            dialog.connect_response(None, move |_, response| {
                if response == "insert" {
                    let url = entry_inner.text().to_string();
                    if !url.trim().is_empty() {
                        let tag_name = format!("link:{}", url.trim());
                        let tag = crate::markdown::create_or_get_link_tag(&buffer_inner, &tag_name);

                        if let Some((start, end)) = buffer_inner.selection_bounds() {
                            buffer_inner.apply_tag(&tag, &start, &end);
                        } else {
                            let start_offset = buffer_inner.cursor_position();
                            buffer_inner.insert_at_cursor("Link");
                            let start = buffer_inner.iter_at_offset(start_offset);
                            let end = buffer_inner.iter_at_offset(start_offset + 4);
                            buffer_inner.apply_tag(&tag, &start, &end);
                        }
                    }
                }
            });

            dialog.present();
        });
    }

    fn toggle_tag(buffer: &TextBuffer, tag_name: &str) {
        if let Some((start, end)) = buffer.selection_bounds() {
            if let Some(tag) = buffer.tag_table().lookup(tag_name) {
                let has_tag = start.has_tag(&tag);
                if has_tag {
                    buffer.remove_tag(&tag, &start, &end);
                } else {
                    buffer.apply_tag(&tag, &start, &end);
                }
            }
        }
    }

    fn apply_line_tag(buffer: &TextBuffer, tag_name: &str) {
        let cursor_iter = buffer.iter_at_offset(buffer.cursor_position());
        let mut line_start = cursor_iter;
        line_start.set_line_offset(0);
        let mut line_end = cursor_iter;
        if !line_end.ends_line() {
            line_end.forward_to_line_end();
        }

        if let Some(tag) = buffer.tag_table().lookup(tag_name) {
            buffer.apply_tag(&tag, &line_start, &line_end);
        }
    }

    pub fn setup_drop_target<F>(&self, on_files_dropped: F)
    where
        F: Fn(Vec<std::path::PathBuf>, f64, f64) + 'static,
    {
        let drop_target =
            gtk4::DropTarget::new(gdk4::FileList::static_type(), gdk4::DragAction::COPY);
        drop_target.connect_drop(move |_, value, x, y| {
            if let Ok(file_list) = value.get::<gdk4::FileList>() {
                let paths: Vec<std::path::PathBuf> = file_list
                    .files()
                    .iter()
                    .filter_map(|file| file.path())
                    .collect();
                if !paths.is_empty() {
                    on_files_dropped(paths, x, y);
                    return true;
                }
            }
            false
        });
        self.text_view.add_controller(drop_target);
    }
}
