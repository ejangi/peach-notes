use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation, TextBuffer, TextView};

pub fn render_code_block_widget(
    buffer: &TextBuffer,
    text_view: &TextView,
    iter: &mut gtk4::TextIter,
    code_content: &str,
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
        .orientation(Orientation::Vertical)
        .hexpand(true)
        .css_classes(vec!["code-block-container".to_string()])
        .build();

    let label = Label::builder()
        .label(code_content.trim_end())
        .selectable(true)
        .wrap(false)
        .hexpand(true)
        .xalign(0.0)
        .css_classes(vec!["code-block-text".to_string()])
        .build();

    container_box.append(&label);
    text_view.add_child_at_anchor(&container_box, &anchor);

    let start_iter = buffer.iter_at_offset(anchor_offset);
    let end_iter = buffer.end_iter();
    if let Some(tag) = buffer.tag_table().lookup("code-block") {
        buffer.apply_tag(&tag, &start_iter, &end_iter);
    }
}
