use crate::markdown::parser::decode_percent_url;
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Label, TextBuffer, TextView};
use std::path::Path;

pub fn render_image_widget(
    buffer: &TextBuffer,
    text_view: &TextView,
    iter: &mut gtk4::TextIter,
    image_dest_url: &str,
    image_alt_text: &str,
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
        .orientation(gtk4::Orientation::Vertical)
        .hexpand(true)
        .css_classes(vec!["note-image-container".to_string()])
        .build();

    container_box.set_widget_name(&format!("IMG|{}|{}", image_dest_url, image_alt_text));

    let decoded_url = decode_percent_url(image_dest_url);
    let raw_path = Path::new(&decoded_url);
    let image_path = if raw_path.is_absolute() {
        raw_path.to_path_buf()
    } else if let Some(dir) = notes_dir {
        dir.join(&decoded_url)
    } else {
        raw_path.to_path_buf()
    };

    if image_path.exists() {
        let picture = gtk4::Picture::new();
        if gdk4::Display::default().is_some() {
            if let Ok(texture) = gdk4::Texture::from_filename(&image_path) {
                let p_w = texture.width();
                let p_h = texture.height();
                let tv_w = text_view.width();
                let max_w = if tv_w > 50 { (tv_w - 48).max(100) } else { 580 };
                let (disp_w, disp_h) = if p_w > max_w {
                    let ratio = max_w as f64 / p_w as f64;
                    (max_w, (p_h as f64 * ratio) as i32)
                } else {
                    (p_w, p_h)
                };
                picture.set_paintable(Some(&texture));
                picture.set_size_request(disp_w, disp_h);
            }
        }
        picture.set_content_fit(gtk4::ContentFit::ScaleDown);
        picture.set_can_shrink(true);
        picture.set_hexpand(true);
        picture.set_halign(Align::Center);
        container_box.append(&picture);
    } else {
        let placeholder_label = Label::builder()
            .label(&format!("📷 Missing Image: {}", image_dest_url))
            .css_classes(vec!["note-image-missing".to_string()])
            .build();
        container_box.append(&placeholder_label);
    }

    if !image_alt_text.is_empty() {
        let caption = Label::builder()
            .label(image_alt_text)
            .css_classes(vec!["note-image-caption".to_string()])
            .build();
        container_box.append(&caption);
    }

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

pub fn resize_all_images_in_buffer(buffer: &TextBuffer, max_w: i32) {
    let mut iter = buffer.start_iter();
    while iter.offset() < buffer.end_iter().offset() {
        if let Some(anchor) = iter.child_anchor() {
            for widget in anchor.widgets() {
                if widget.has_css_class("note-image-container") {
                    if let Ok(container_box) = widget.clone().downcast::<GtkBox>() {
                        let mut img_child = container_box.first_child();
                        while let Some(ic) = img_child {
                            if let Ok(picture) = ic.clone().downcast::<gtk4::Picture>() {
                                if let Some(paintable) = picture.paintable() {
                                    let p_w = paintable.intrinsic_width();
                                    let p_h = paintable.intrinsic_height();
                                    if p_w > 0 && p_h > 0 {
                                        let (disp_w, disp_h) = if p_w > max_w {
                                            let ratio = max_w as f64 / p_w as f64;
                                            (max_w, (p_h as f64 * ratio) as i32)
                                        } else {
                                            (p_w, p_h)
                                        };
                                        picture.set_size_request(disp_w, disp_h);
                                    }
                                }
                            }
                            img_child = ic.next_sibling();
                        }
                    }
                }
            }
        }
        if !iter.forward_char() {
            break;
        }
    }
}
