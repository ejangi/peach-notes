use crate::domain::Note;
use crate::markdown::parser::decode_percent_url;
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Label, Orientation, TextBuffer, TextView};
use std::path::Path;

pub fn render_attachment_widget(
    buffer: &TextBuffer,
    text_view: &TextView,
    iter: &mut gtk4::TextIter,
    link_dest_url: &str,
    link_text: &str,
    notes_dir: Option<&Path>,
) {
    let end_offset = iter.offset();
    if end_offset > 0 {
        let mut check_iter = *iter;
        check_iter.backward_char();
        if check_iter.char() != '\n' {
            buffer.insert(iter, "\n");
        }
    }

    let anchor_offset = iter.offset();
    let anchor = buffer.create_child_anchor(iter);
    buffer.insert(iter, "\n");

    let container_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .hexpand(true)
        .css_classes(vec!["note-attachment-container".to_string()])
        .build();

    let clean_title = link_text
        .strip_prefix("📎 ")
        .unwrap_or(link_text)
        .to_string();

    container_box.set_widget_name(&format!("ATTACHMENT|{}|{}", link_dest_url, clean_title));

    let icon_label = Label::builder()
        .label("📎")
        .css_classes(vec!["note-attachment-icon".to_string()])
        .build();

    let text_vbox = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(2)
        .hexpand(true)
        .halign(Align::Start)
        .build();

    let title_label = Label::builder()
        .label(&clean_title)
        .halign(Align::Start)
        .css_classes(vec!["note-attachment-title".to_string()])
        .build();

    let decoded_url = decode_percent_url(link_dest_url);
    let full_file_path = if let Some(dir) = notes_dir {
        dir.join(&decoded_url)
    } else {
        Path::new(&decoded_url).to_path_buf()
    };

    let size_str = if let Ok(meta) = std::fs::metadata(&full_file_path) {
        Note::format_file_size(meta.len())
    } else {
        "File Attachment".to_string()
    };

    let size_label = Label::builder()
        .label(&size_str)
        .halign(Align::Start)
        .css_classes(vec!["note-attachment-size".to_string()])
        .build();

    text_vbox.append(&title_label);
    text_vbox.append(&size_label);

    let open_button = Button::builder()
        .label("Open")
        .icon_name("external-link-symbolic")
        .tooltip_text("Open Attachment")
        .build();

    let target_path_clone = full_file_path.clone();
    open_button.connect_clicked(move |_| {
        if target_path_clone.exists() {
            let _ = std::process::Command::new("xdg-open")
                .arg(&target_path_clone)
                .spawn();
        }
    });

    container_box.append(&icon_label);
    container_box.append(&text_vbox);
    container_box.append(&open_button);

    text_view.add_child_at_anchor(&container_box, &anchor);
    text_view.queue_resize();
    text_view.queue_draw();

    let start_iter = buffer.iter_at_offset(anchor_offset);
    let mut end_iter = buffer.iter_at_offset(anchor_offset);
    end_iter.forward_char();
    if let Some(tag) = buffer.tag_table().lookup("image") {
        buffer.apply_tag(&tag, &start_iter, &end_iter);
    }
}
