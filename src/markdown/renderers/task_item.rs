use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, CheckButton, Orientation, TextBuffer, TextView};

pub fn render_task_item_widget(
    buffer: &TextBuffer,
    text_view: &TextView,
    iter: &mut gtk4::TextIter,
    checked: bool,
) {
    let anchor_offset = iter.offset();
    let anchor = buffer.create_child_anchor(iter);
    buffer.insert(iter, " ");

    let container_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .valign(Align::Center)
        .halign(Align::Start)
        .width_request(22)
        .height_request(16)
        .css_classes(vec!["note-task-container".to_string()])
        .build();

    let check_button = CheckButton::builder()
        .active(checked)
        .valign(Align::Center)
        .width_request(16)
        .height_request(16)
        .css_classes(vec!["note-task-check".to_string()])
        .build();

    let state_str = if checked { "TASK|[x]" } else { "TASK|[ ]" };
    container_box.set_widget_name(state_str);

    let buf_clone = buffer.clone();
    let anchor_offset_clone = anchor_offset;
    let box_clone = container_box.clone();
    check_button.connect_toggled(move |btn| {
        let is_active = btn.is_active();
        let new_name = if is_active { "TASK|[x]" } else { "TASK|[ ]" };
        box_clone.set_widget_name(new_name);

        update_task_line_styling(&buf_clone, anchor_offset_clone, is_active);
        buf_clone.emit_by_name::<()>("changed", &[]);
    });

    container_box.append(&check_button);
    text_view.add_child_at_anchor(&container_box, &anchor);

    if checked {
        update_task_line_styling(buffer, anchor_offset, true);
    }
}

pub fn update_task_line_styling(buffer: &TextBuffer, offset: i32, is_done: bool) {
    let iter = buffer.iter_at_offset(offset);
    let mut line_start = iter;
    line_start.set_line_offset(0);
    let mut line_end = iter;
    if !line_end.ends_line() {
        line_end.forward_to_line_end();
    }

    if let Some(tag) = buffer.tag_table().lookup("task-done") {
        if is_done {
            buffer.apply_tag(&tag, &line_start, &line_end);
        } else {
            buffer.remove_tag(&tag, &line_start, &line_end);
        }
    }
}
