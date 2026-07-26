use gtk4::prelude::*;
use gtk4::{
    Align, Box as GtkBox, Button, Entry, Grid, MenuButton, Orientation, Overlay, Popover,
    TextBuffer, TextView,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TableData {
    pub alignments: Vec<String>,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl TableData {
    pub fn new(alignments: Vec<pulldown_cmark::Alignment>) -> Self {
        let align_strs = alignments
            .into_iter()
            .map(|a| match a {
                pulldown_cmark::Alignment::Left => "left".to_string(),
                pulldown_cmark::Alignment::Center => "center".to_string(),
                pulldown_cmark::Alignment::Right => "right".to_string(),
                pulldown_cmark::Alignment::None => "none".to_string(),
            })
            .collect();
        Self {
            alignments: align_strs,
            headers: Vec::new(),
            rows: Vec::new(),
        }
    }

    pub fn to_markdown(&self) -> String {
        let mut out = String::new();
        if self.headers.is_empty() {
            return out;
        }

        // 1. Header row
        out.push('|');
        for h in &self.headers {
            out.push_str(&format!(" {} |", h));
        }
        out.push('\n');

        // 2. Delimiter row
        out.push('|');
        for (i, _) in self.headers.iter().enumerate() {
            let align = self.alignments.get(i).map(|s| s.as_str()).unwrap_or("none");
            match align {
                "left" => out.push_str(" :--- |"),
                "center" => out.push_str(" :---: |"),
                "right" => out.push_str(" ---: |"),
                _ => out.push_str(" --- |"),
            }
        }
        out.push('\n');

        // 3. Data rows
        for row in &self.rows {
            out.push('|');
            for (i, _) in self.headers.iter().enumerate() {
                let cell_val = row.get(i).cloned().unwrap_or_default();
                out.push_str(&format!(" {} |", cell_val));
            }
            out.push('\n');
        }

        out
    }
}

pub fn render_table_widget(
    buffer: &TextBuffer,
    text_view: &TextView,
    iter: &mut gtk4::TextIter,
    table_data: &TableData,
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

    let main_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(0)
        .hexpand(true)
        .css_classes(vec!["note-table-container".to_string()])
        .build();

    let json_payload = serde_json::to_string(table_data).unwrap_or_default();
    main_box.set_widget_name(&format!("TABLE|{}", json_payload));

    build_table_ui(&main_box, table_data, buffer, text_view);

    text_view.add_child_at_anchor(&main_box, &anchor);
    text_view.queue_resize();
    text_view.queue_draw();

    let start_iter = buffer.iter_at_offset(anchor_offset);
    let mut end_iter = buffer.iter_at_offset(anchor_offset);
    end_iter.forward_char();
    if let Some(tag) = buffer.tag_table().lookup("image") {
        buffer.apply_tag(&tag, &start_iter, &end_iter);
    }
}

fn build_table_ui(
    main_box: &GtkBox,
    table_data: &TableData,
    buffer: &TextBuffer,
    text_view: &TextView,
) {
    // Clear existing children
    while let Some(child) = main_box.first_child() {
        main_box.remove(&child);
    }

    let grid = Grid::builder()
        .column_spacing(0)
        .row_spacing(0)
        .css_classes(vec!["note-table-grid".to_string()])
        .build();

    let data_ref = std::rc::Rc::new(std::cell::RefCell::new(table_data.clone()));
    let main_box_clone = main_box.clone();
    let buffer_clone = buffer.clone();
    let text_view_clone = text_view.clone();

    let sync_payload = {
        let data_ref = data_ref.clone();
        let main_box_clone = main_box_clone.clone();
        let buffer_clone = buffer_clone.clone();
        move || {
            if let Ok(json) = serde_json::to_string(&*data_ref.borrow()) {
                main_box_clone.set_widget_name(&format!("TABLE|{}", json));
                buffer_clone.emit_by_name::<()>("changed", &[]);
            }
        }
    };

    let rebuild_all = {
        let data_ref = data_ref.clone();
        let main_box_clone = main_box_clone.clone();
        let buffer_clone = buffer_clone.clone();
        let text_view_clone = text_view_clone.clone();
        move || {
            let curr = data_ref.borrow().clone();
            build_table_ui(&main_box_clone, &curr, &buffer_clone, &text_view_clone);
            if let Ok(json) = serde_json::to_string(&curr) {
                main_box_clone.set_widget_name(&format!("TABLE|{}", json));
                buffer_clone.emit_by_name::<()>("changed", &[]);
            }
            text_view_clone.queue_resize();
        }
    };

    let num_cols = table_data.headers.len();
    let num_rows = table_data.rows.len();

    // 1. Headers Row (Grid row 0)
    for (col_idx, header_text) in table_data.headers.iter().enumerate() {
        let overlay = Overlay::new();
        let cell_box = GtkBox::builder()
            .css_classes(vec!["note-table-cell-header".to_string()])
            .hexpand(true)
            .build();

        let initial_width = (header_text.chars().count() as i32 + 5).max(10);
        let entry = Entry::builder()
            .text(header_text)
            .width_chars(initial_width)
            .css_classes(vec!["note-table-input".to_string()])
            .hexpand(true)
            .build();

        let align_str = table_data
            .alignments
            .get(col_idx)
            .map(|s| s.as_str())
            .unwrap_or("none");
        match align_str {
            "left" => entry.set_halign(Align::Start),
            "center" => entry.set_halign(Align::Center),
            "right" => entry.set_halign(Align::End),
            _ => {}
        }

        let sync_h = sync_payload.clone();
        let data_h = data_ref.clone();
        let tv_resize_h = text_view.clone();
        entry.connect_changed(move |e| {
            let text_val = e.text().to_string();
            e.set_width_chars((text_val.chars().count() as i32 + 5).max(10));
            if let Ok(mut d) = data_h.try_borrow_mut() {
                if col_idx < d.headers.len() {
                    d.headers[col_idx] = text_val;
                }
            }
            sync_h();
            tv_resize_h.queue_resize();
        });

        cell_box.append(&entry);
        overlay.set_child(Some(&cell_box));

        // Column options overlay menu
        let col_menu = MenuButton::builder()
            .icon_name("view-more-symbolic")
            .css_classes(vec![
                "flat".to_string(),
                "circular".to_string(),
                "note-table-overlay-menu".to_string(),
            ])
            .tooltip_text("Column options")
            .halign(Align::End)
            .valign(Align::Start)
            .build();

        let popover = Popover::new();
        let pop_box = GtkBox::new(Orientation::Vertical, 6);
        pop_box.set_margin_start(8);
        pop_box.set_margin_end(8);
        pop_box.set_margin_top(8);
        pop_box.set_margin_bottom(8);

        // Add Column Right
        let add_col_right_btn = Button::builder()
            .label("+ Add Column Right")
            .css_classes(vec!["flat".to_string()])
            .build();

        let rebuild_acr = rebuild_all.clone();
        let data_acr = data_ref.clone();
        let pop_acr = popover.clone();
        add_col_right_btn.connect_clicked(move |_| {
            pop_acr.popdown();
            let mut d = data_acr.borrow_mut();
            let new_idx = col_idx + 1;
            let header_title = format!("Header {}", d.headers.len() + 1);
            if new_idx <= d.headers.len() {
                d.headers.insert(new_idx, header_title);
                d.alignments.insert(new_idx, "none".to_string());
                for row in &mut d.rows {
                    row.insert(new_idx, String::new());
                }
            } else {
                d.headers.push(header_title);
                d.alignments.push("none".to_string());
                for row in &mut d.rows {
                    row.push(String::new());
                }
            }
            drop(d);
            rebuild_acr();
        });

        // Add Column Left
        let add_col_left_btn = Button::builder()
            .label("+ Add Column Left")
            .css_classes(vec!["flat".to_string()])
            .build();

        let rebuild_acl = rebuild_all.clone();
        let data_acl = data_ref.clone();
        let pop_acl = popover.clone();
        add_col_left_btn.connect_clicked(move |_| {
            pop_acl.popdown();
            let mut d = data_acl.borrow_mut();
            let header_title = format!("Header {}", d.headers.len() + 1);
            d.headers.insert(col_idx, header_title);
            d.alignments.insert(col_idx, "none".to_string());
            for row in &mut d.rows {
                row.insert(col_idx, String::new());
            }
            drop(d);
            rebuild_acl();
        });

        pop_box.append(&add_col_right_btn);
        pop_box.append(&add_col_left_btn);

        // Delete Column
        if num_cols > 1 {
            let del_col_btn = Button::builder()
                .label("Delete Column")
                .css_classes(vec!["destructive-action".to_string()])
                .build();

            let rebuild_del_col = rebuild_all.clone();
            let data_del_col = data_ref.clone();
            let pop_del = popover.clone();
            del_col_btn.connect_clicked(move |_| {
                pop_del.popdown();
                let mut d = data_del_col.borrow_mut();
                if col_idx < d.headers.len() {
                    d.headers.remove(col_idx);
                    if col_idx < d.alignments.len() {
                        d.alignments.remove(col_idx);
                    }
                    for row in &mut d.rows {
                        if col_idx < row.len() {
                            row.remove(col_idx);
                        }
                    }
                }
                drop(d);
                rebuild_del_col();
            });

            pop_box.append(&del_col_btn);
        }

        popover.set_child(Some(&pop_box));
        col_menu.set_popover(Some(&popover));
        overlay.add_overlay(&col_menu);

        grid.attach(&overlay, col_idx as i32, 0, 1, 1);
    }

    // 2. Data Rows (Grid rows 1..N)
    for (r_idx, row_data) in table_data.rows.iter().enumerate() {
        let grid_r = (r_idx + 1) as i32;

        for (c_idx, cell_text) in row_data.iter().enumerate() {
            if c_idx >= num_cols {
                break;
            }

            let overlay = Overlay::new();
            let cell_box = GtkBox::builder()
                .css_classes(vec!["note-table-cell-data".to_string()])
                .hexpand(true)
                .build();

            let initial_width = (cell_text.chars().count() as i32 + 5).max(10);
            let entry = Entry::builder()
                .text(cell_text)
                .width_chars(initial_width)
                .css_classes(vec!["note-table-input".to_string()])
                .hexpand(true)
                .build();

            let align_str = table_data
                .alignments
                .get(c_idx)
                .map(|s| s.as_str())
                .unwrap_or("none");
            match align_str {
                "left" => entry.set_halign(Align::Start),
                "center" => entry.set_halign(Align::Center),
                "right" => entry.set_halign(Align::End),
                _ => {}
            }

            let sync_c = sync_payload.clone();
            let data_c = data_ref.clone();
            let tv_resize_c = text_view.clone();
            entry.connect_changed(move |e| {
                let text_val = e.text().to_string();
                e.set_width_chars((text_val.chars().count() as i32 + 5).max(10));
                if let Ok(mut d) = data_c.try_borrow_mut() {
                    if r_idx < d.rows.len() && c_idx < d.rows[r_idx].len() {
                        d.rows[r_idx][c_idx] = text_val;
                    }
                }
                sync_c();
                tv_resize_c.queue_resize();
            });

            cell_box.append(&entry);
            overlay.set_child(Some(&cell_box));

            // Row menu on last column cell as floating overlay
            if c_idx == num_cols - 1 {
                let row_menu = MenuButton::builder()
                    .icon_name("view-more-symbolic")
                    .css_classes(vec![
                        "flat".to_string(),
                        "circular".to_string(),
                        "note-table-overlay-menu".to_string(),
                    ])
                    .tooltip_text("Row options")
                    .halign(Align::End)
                    .valign(Align::Start)
                    .build();

                let popover = Popover::new();
                let pop_box = GtkBox::new(Orientation::Vertical, 6);
                pop_box.set_margin_start(8);
                pop_box.set_margin_end(8);
                pop_box.set_margin_top(8);
                pop_box.set_margin_bottom(8);

                // Add Row Below
                let add_row_below_btn = Button::builder()
                    .label("+ Add Row Below")
                    .css_classes(vec!["flat".to_string()])
                    .build();

                let rebuild_arb = rebuild_all.clone();
                let data_arb = data_ref.clone();
                let pop_arb = popover.clone();
                add_row_below_btn.connect_clicked(move |_| {
                    pop_arb.popdown();
                    let mut d = data_arb.borrow_mut();
                    let col_count = d.headers.len();
                    let new_idx = r_idx + 1;
                    if new_idx <= d.rows.len() {
                        d.rows.insert(new_idx, vec![String::new(); col_count]);
                    } else {
                        d.rows.push(vec![String::new(); col_count]);
                    }
                    drop(d);
                    rebuild_arb();
                });

                // Add Row Above
                let add_row_above_btn = Button::builder()
                    .label("+ Add Row Above")
                    .css_classes(vec!["flat".to_string()])
                    .build();

                let rebuild_ara = rebuild_all.clone();
                let data_ara = data_ref.clone();
                let pop_ara = popover.clone();
                add_row_above_btn.connect_clicked(move |_| {
                    pop_ara.popdown();
                    let mut d = data_ara.borrow_mut();
                    let col_count = d.headers.len();
                    d.rows.insert(r_idx, vec![String::new(); col_count]);
                    drop(d);
                    rebuild_ara();
                });

                pop_box.append(&add_row_below_btn);
                pop_box.append(&add_row_above_btn);

                // Delete Row
                if num_rows > 1 {
                    let del_row_btn = Button::builder()
                        .label("Delete Row")
                        .css_classes(vec!["destructive-action".to_string()])
                        .build();

                    let rebuild_del_row = rebuild_all.clone();
                    let data_del_row = data_ref.clone();
                    let pop_dr = popover.clone();
                    del_row_btn.connect_clicked(move |_| {
                        pop_dr.popdown();
                        let mut d = data_del_row.borrow_mut();
                        if r_idx < d.rows.len() {
                            d.rows.remove(r_idx);
                        }
                        drop(d);
                        rebuild_del_row();
                    });

                    pop_box.append(&del_row_btn);
                }

                popover.set_child(Some(&pop_box));
                row_menu.set_popover(Some(&popover));
                overlay.add_overlay(&row_menu);
            }

            grid.attach(&overlay, c_idx as i32, grid_r, 1, 1);
        }
    }

    main_box.append(&grid);
}
