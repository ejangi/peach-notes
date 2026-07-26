use crate::domain::Note;
use crate::markdown::renderers::{
    render_attachment_widget, render_code_block_widget, render_image_widget, render_table_widget,
    render_task_item_widget, TableData,
};
use glib::translate::IntoGlib;
use gtk4::prelude::*;
use gtk4::{TextBuffer, TextView};
use pulldown_cmark::{Event, HeadingLevel, Options, Tag, TagEnd};
use std::path::Path;

pub fn decode_percent_url(url: &str) -> String {
    let mut result = String::new();
    let bytes = url.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(val) = u8::from_str_radix(&url[i + 1..i + 3], 16) {
                result.push(val as char);
                i += 3;
                continue;
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

pub fn encode_asset_urls_in_markdown(markdown: &str) -> String {
    let mut result = String::with_capacity(markdown.len());
    for line in markdown.lines() {
        if (line.contains("!](") || line.contains("](") || line.contains("!["))
            && (line.contains(".assets/") || line.contains(".assets\\"))
        {
            let mut new_line = String::new();
            let mut i = 0;
            let bytes = line.as_bytes();
            while i < bytes.len() {
                if bytes[i] == b']' && i + 1 < bytes.len() && bytes[i + 1] == b'(' {
                    new_line.push(']');
                    new_line.push('(');
                    i += 2;
                    let start_url = i;
                    while i < bytes.len() && bytes[i] != b')' && bytes[i] != b'\n' {
                        i += 1;
                    }
                    let raw_url = &line[start_url..i];
                    let encoded_url = raw_url.replace(' ', "%20");
                    new_line.push_str(&encoded_url);
                } else {
                    new_line.push(bytes[i] as char);
                    i += 1;
                }
            }
            result.push_str(&new_line);
            result.push('\n');
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }
    result
}

#[derive(Debug, Clone)]
enum ListKind {
    Unordered,
    Ordered(u64),
}

pub fn is_system_dark_mode() -> bool {
    let style_mgr = libadwaita::StyleManager::default();
    if style_mgr.is_dark() {
        return true;
    }

    if let Some(settings) = gtk4::Settings::default() {
        if settings.is_gtk_application_prefer_dark_theme() {
            return true;
        }

        if let Some(theme_name) = settings.gtk_theme_name() {
            let lower = theme_name.to_lowercase();
            if lower.contains("dark") || lower.contains("black") {
                return true;
            }
        }
    }

    false
}

pub fn setup_text_buffer_tags(buffer: &TextBuffer) {
    let tag_table = buffer.tag_table();
    let is_dark = is_system_dark_mode();

    let mono_bg = if is_dark {
        gdk4::RGBA::new(1.0, 1.0, 1.0, 0.12)
    } else {
        gdk4::RGBA::new(0.0, 0.0, 0.0, 0.06)
    };

    let bq_fg = if is_dark {
        gdk4::RGBA::new(0.75, 0.78, 0.82, 1.0)
    } else {
        gdk4::RGBA::new(0.40, 0.42, 0.46, 1.0)
    };

    let link_fg = if is_dark {
        gdk4::RGBA::new(0.45, 0.70, 1.0, 1.0)
    } else {
        gdk4::RGBA::new(0.10, 0.40, 0.80, 1.0)
    };

    let task_done_fg = if is_dark {
        gdk4::RGBA::new(1.0, 1.0, 1.0, 0.45)
    } else {
        gdk4::RGBA::new(0.0, 0.0, 0.0, 0.45)
    };

    if let Some(tag) = tag_table.lookup("task-done") {
        tag.set_foreground_rgba(Some(&task_done_fg));
    } else {
        let tag = gtk4::TextTag::builder()
            .name("task-done")
            .foreground_rgba(&task_done_fg)
            .build();
        tag_table.add(&tag);
    }

    if tag_table.lookup("heading-1").is_none() {
        let tag = gtk4::TextTag::builder()
            .name("heading-1")
            .weight(pango::Weight::Bold.into_glib())
            .scale(1.6)
            .pixels_above_lines(16)
            .pixels_below_lines(16)
            .build();
        tag_table.add(&tag);
    }

    if tag_table.lookup("heading-2").is_none() {
        let tag = gtk4::TextTag::builder()
            .name("heading-2")
            .weight(pango::Weight::Bold.into_glib())
            .scale(1.3)
            .pixels_above_lines(12)
            .pixels_below_lines(12)
            .build();
        tag_table.add(&tag);
    }

    if tag_table.lookup("heading-3").is_none() {
        let tag = gtk4::TextTag::builder()
            .name("heading-3")
            .weight(pango::Weight::Bold.into_glib())
            .scale(1.15)
            .pixels_above_lines(8)
            .pixels_below_lines(8)
            .build();
        tag_table.add(&tag);
    }

    if tag_table.lookup("bold").is_none() {
        let tag = gtk4::TextTag::builder()
            .name("bold")
            .weight(pango::Weight::Bold.into_glib())
            .build();
        tag_table.add(&tag);
    }

    if tag_table.lookup("italic").is_none() {
        let tag = gtk4::TextTag::builder()
            .name("italic")
            .style(pango::Style::Italic)
            .build();
        tag_table.add(&tag);
    }

    if let Some(tag) = tag_table.lookup("monospace") {
        tag.set_background_rgba(Some(&mono_bg));
    } else {
        let tag = gtk4::TextTag::builder()
            .name("monospace")
            .family("Monospace")
            .background_rgba(&mono_bg)
            .build();
        tag_table.add(&tag);
    }

    if tag_table.lookup("code-block").is_none() {
        let tag = gtk4::TextTag::builder()
            .name("code-block")
            .family("Monospace")
            .build();
        tag_table.add(&tag);
    }

    if tag_table.lookup("image").is_none() {
        let tag = gtk4::TextTag::builder().name("image").build();
        tag_table.add(&tag);
    }

    if let Some(tag) = tag_table.lookup("blockquote") {
        tag.set_foreground_rgba(Some(&bq_fg));
    } else {
        let tag = gtk4::TextTag::builder()
            .name("blockquote")
            .left_margin(20)
            .style(pango::Style::Italic)
            .foreground_rgba(&bq_fg)
            .build();
        tag_table.add(&tag);
    }

    if tag_table.lookup("strikethrough").is_none() {
        let tag = gtk4::TextTag::builder()
            .name("strikethrough")
            .strikethrough(true)
            .build();
        tag_table.add(&tag);
    }

    if tag_table.lookup("bullet-list").is_none() {
        let tag = gtk4::TextTag::builder()
            .name("bullet-list")
            .pixels_below_lines(2)
            .build();
        tag_table.add(&tag);
    }

    if tag_table.lookup("list-block-end").is_none() {
        let tag = gtk4::TextTag::builder()
            .name("list-block-end")
            .pixels_below_lines(20)
            .build();
        tag_table.add(&tag);
    }

    if tag_table.lookup("ordered-list").is_none() {
        let tag = gtk4::TextTag::builder().name("ordered-list").build();
        tag_table.add(&tag);
    }

    if tag_table.lookup("task-done").is_none() {
        let tag = gtk4::TextTag::builder()
            .name("task-done")
            .strikethrough(true)
            .foreground_rgba(&gdk4::RGBA::new(0.5, 0.5, 0.5, 1.0))
            .build();
        tag_table.add(&tag);
    }

    if let Some(tag) = tag_table.lookup("link") {
        tag.set_foreground_rgba(Some(&link_fg));
    } else {
        let tag = gtk4::TextTag::builder()
            .name("link")
            .foreground_rgba(&link_fg)
            .underline(pango::Underline::Single)
            .build();
        tag_table.add(&tag);
    }

    if tag_table.lookup("paragraph").is_none() {
        let tag = gtk4::TextTag::builder()
            .name("paragraph")
            .pixels_below_lines(14)
            .build();
        tag_table.add(&tag);
    }
}

pub fn create_or_get_link_tag(buffer: &TextBuffer, tag_name: &str) -> gtk4::TextTag {
    let tag_table = buffer.tag_table();
    if let Some(tag) = tag_table.lookup(tag_name) {
        tag
    } else {
        let is_dark = is_system_dark_mode();
        let link_fg = if is_dark {
            gdk4::RGBA::new(0.45, 0.70, 1.0, 1.0)
        } else {
            gdk4::RGBA::new(0.10, 0.40, 0.80, 1.0)
        };

        let tag = gtk4::TextTag::builder()
            .name(tag_name)
            .foreground_rgba(&link_fg)
            .underline(pango::Underline::Single)
            .build();
        tag_table.add(&tag);
        tag
    }
}

/// Parses GFM Markdown into GtkTextBuffer, inserting widget containers for code blocks and images.
/// Returns extracted frontmatter block (if present).
pub fn parse_markdown_to_buffer(
    markdown: &str,
    buffer: &TextBuffer,
    text_view: &TextView,
    notes_dir: Option<&Path>,
) -> Option<String> {
    setup_text_buffer_tags(buffer);

    buffer.set_text("");

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    options.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);

    let sanitized_markdown = encode_asset_urls_in_markdown(markdown);
    let parser = pulldown_cmark::Parser::new_ext(&sanitized_markdown, options);
    let mut active_tags: Vec<String> = Vec::new();
    let mut list_stack: Vec<ListKind> = Vec::new();
    let mut in_list_item = false;
    let mut in_metadata = false;
    let mut in_code_block = false;
    let mut in_image = false;
    let mut in_link = false;
    let mut link_dest_url = String::new();
    let mut link_text_accumulator = String::new();
    let mut image_dest_url = String::new();
    let mut image_alt_text = String::new();
    let mut code_block_accumulator = String::new();
    let mut extracted_frontmatter = String::new();

    let mut in_table = false;
    let mut current_table: Option<TableData> = None;
    let mut current_table_row: Option<Vec<String>> = None;
    let mut current_cell_text = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::Table(alignments)) => {
                if !in_metadata {
                    in_table = true;
                    current_table = Some(TableData::new(alignments));
                }
            }
            Event::End(TagEnd::Table) => {
                if !in_metadata && in_table {
                    in_table = false;
                    if let Some(table) = current_table.take() {
                        render_table_widget(buffer, text_view, &mut buffer.end_iter(), &table);
                    }
                }
            }
            Event::Start(Tag::TableHead) => {
                if !in_metadata {
                    current_table_row = Some(Vec::new());
                }
            }
            Event::End(TagEnd::TableHead) => {
                if !in_metadata {
                    if let Some(row) = current_table_row.take() {
                        if let Some(ref mut table) = current_table {
                            table.headers = row;
                        }
                    }
                }
            }
            Event::Start(Tag::TableRow) => {
                if !in_metadata {
                    current_table_row = Some(Vec::new());
                }
            }
            Event::End(TagEnd::TableRow) => {
                if !in_metadata {
                    if let Some(row) = current_table_row.take() {
                        if let Some(ref mut table) = current_table {
                            table.rows.push(row);
                        }
                    }
                }
            }
            Event::Start(Tag::TableCell) => {
                if !in_metadata {
                    current_cell_text.clear();
                }
            }
            Event::End(TagEnd::TableCell) => {
                if !in_metadata {
                    if let Some(ref mut row) = current_table_row {
                        row.push(current_cell_text.trim().to_string());
                    }
                }
            }
            Event::Start(Tag::MetadataBlock(_)) => {
                in_metadata = true;
            }
            Event::End(TagEnd::MetadataBlock(_)) => {
                in_metadata = false;
            }
            Event::Start(Tag::List(first_num)) => {
                if !in_metadata {
                    match first_num {
                        Some(start) => list_stack.push(ListKind::Ordered(start)),
                        None => list_stack.push(ListKind::Unordered),
                    }
                }
            }
            Event::End(TagEnd::List(_)) => {
                if !in_metadata {
                    list_stack.pop();
                    if list_stack.is_empty() {
                        let end_offset = buffer.end_iter().offset();
                        if end_offset > 0 {
                            let text_before = buffer
                                .text(&buffer.start_iter(), &buffer.end_iter(), true)
                                .to_string();
                            if !text_before.ends_with("\n\n") {
                                if text_before.ends_with('\n') {
                                    buffer.insert(&mut buffer.end_iter(), "\n");
                                } else {
                                    buffer.insert(&mut buffer.end_iter(), "\n\n");
                                }
                            }
                        }
                    }
                }
            }
            Event::TaskListMarker(checked) => {
                if !in_metadata {
                    let mut end_iter = buffer.end_iter();
                    let end_offset = end_iter.offset();
                    if end_offset >= 2 {
                        let mut start_iter = buffer.iter_at_offset(end_offset - 2);
                        let prev_str = buffer.text(&start_iter, &end_iter, true);
                        if prev_str == "• " || prev_str == "- " || prev_str == "* " {
                            buffer.delete(&mut start_iter, &mut end_iter);
                        }
                    }
                    render_task_item_widget(buffer, text_view, &mut buffer.end_iter(), checked);
                }
            }
            Event::Start(Tag::Image { dest_url, .. }) => {
                if !in_metadata {
                    in_image = true;
                    image_dest_url = dest_url.to_string();
                    image_alt_text.clear();
                }
            }
            Event::End(TagEnd::Image) => {
                if !in_metadata && in_image {
                    in_image = false;
                    render_image_widget(
                        buffer,
                        text_view,
                        &mut buffer.end_iter(),
                        &image_dest_url,
                        &image_alt_text,
                        notes_dir,
                    );
                }
            }
            Event::Start(Tag::CodeBlock(_)) => {
                if !in_metadata {
                    in_code_block = true;
                    code_block_accumulator.clear();
                }
            }
            Event::End(TagEnd::CodeBlock) => {
                if !in_metadata && in_code_block {
                    in_code_block = false;
                    render_code_block_widget(
                        buffer,
                        text_view,
                        &mut buffer.end_iter(),
                        &code_block_accumulator,
                    );
                }
            }
            Event::Start(Tag::Heading { level, .. }) => {
                if in_metadata {
                    continue;
                }
                let end_offset = buffer.end_iter().offset();
                if end_offset > 0 {
                    let mut check_iter = buffer.end_iter();
                    check_iter.backward_char();
                    if check_iter.char() != '\n' {
                        buffer.insert(&mut buffer.end_iter(), "\n");
                    }
                }
                let tag_name = match level {
                    HeadingLevel::H1 => "heading-1",
                    HeadingLevel::H2 => "heading-2",
                    _ => "heading-3",
                };
                active_tags.push(tag_name.to_string());
            }
            Event::End(TagEnd::Heading(_)) => {
                if in_metadata {
                    continue;
                }
                active_tags.retain(|t| t != "heading-1" && t != "heading-2" && t != "heading-3");
                buffer.insert(&mut buffer.end_iter(), "\n");
            }
            Event::Start(Tag::BlockQuote(_)) => {
                if !in_metadata {
                    active_tags.push("blockquote".to_string());
                }
            }
            Event::End(TagEnd::BlockQuote) => {
                if !in_metadata {
                    active_tags.retain(|t| t != "blockquote");
                    buffer.insert(&mut buffer.end_iter(), "\n");
                }
            }
            Event::Start(Tag::Strong) => {
                if !in_metadata {
                    active_tags.push("bold".to_string());
                }
            }
            Event::End(TagEnd::Strong) => {
                if !in_metadata {
                    active_tags.retain(|t| t != "bold");
                }
            }
            Event::Start(Tag::Emphasis) => {
                if !in_metadata {
                    active_tags.push("italic".to_string());
                }
            }
            Event::End(TagEnd::Emphasis) => {
                if !in_metadata {
                    active_tags.retain(|t| t != "italic");
                }
            }
            Event::Start(Tag::Strikethrough) => {
                if !in_metadata {
                    active_tags.push("strikethrough".to_string());
                }
            }
            Event::End(TagEnd::Strikethrough) => {
                if !in_metadata {
                    active_tags.retain(|t| t != "strikethrough");
                }
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                if !in_metadata {
                    in_link = true;
                    link_dest_url = dest_url.to_string();
                    link_text_accumulator.clear();
                    let tag_name = format!("link:{}", dest_url);
                    create_or_get_link_tag(buffer, &tag_name);
                    active_tags.push(tag_name);
                }
            }
            Event::End(TagEnd::Link) => {
                if !in_metadata && in_link {
                    in_link = false;
                    let is_attachment = link_text_accumulator.starts_with("📎 ")
                        || (link_dest_url.contains(".assets/")
                            && !Note::is_image_file(&link_dest_url));

                    active_tags.retain(|t| !t.starts_with("link"));

                    if is_attachment {
                        let text_char_len = link_text_accumulator.chars().count();
                        let mut start_del = buffer.end_iter();
                        start_del.backward_chars(text_char_len as i32);
                        let mut end_del = buffer.end_iter();
                        buffer.delete(&mut start_del, &mut end_del);

                        render_attachment_widget(
                            buffer,
                            text_view,
                            &mut buffer.end_iter(),
                            &link_dest_url,
                            &link_text_accumulator,
                            notes_dir,
                        );
                    }
                }
            }
            Event::Start(Tag::Item) => {
                if !in_metadata {
                    in_list_item = true;
                    if let Some(last) = list_stack.last_mut() {
                        match last {
                            ListKind::Unordered => {
                                active_tags.push("bullet-list".to_string());
                                buffer.insert(&mut buffer.end_iter(), "• ");
                            }
                            ListKind::Ordered(ref mut count) => {
                                active_tags.push("ordered-list".to_string());
                                buffer.insert(&mut buffer.end_iter(), &format!("{}. ", count));
                                *count += 1;
                            }
                        }
                    } else {
                        active_tags.push("bullet-list".to_string());
                        buffer.insert(&mut buffer.end_iter(), "• ");
                    }
                }
            }
            Event::End(TagEnd::Item) => {
                if !in_metadata {
                    in_list_item = false;
                    active_tags
                        .retain(|t| t != "bullet-list" && t != "ordered-list" && t != "task-done");
                    let end_offset = buffer.end_iter().offset();
                    if end_offset > 0 {
                        let mut check_iter = buffer.end_iter();
                        check_iter.backward_char();
                        if check_iter.char() != '\n' {
                            buffer.insert(&mut buffer.end_iter(), "\n");
                        }
                    }
                }
            }
            Event::Code(text) => {
                if in_code_block {
                    code_block_accumulator.push_str(&text);
                } else if in_table {
                    current_cell_text.push_str(&text);
                } else if !in_metadata {
                    let start_offset = buffer.end_iter().offset();
                    buffer.insert(&mut buffer.end_iter(), &text);
                    let start_iter = buffer.iter_at_offset(start_offset);
                    let end_iter = buffer.end_iter();
                    if let Some(tag) = buffer.tag_table().lookup("monospace") {
                        buffer.apply_tag(&tag, &start_iter, &end_iter);
                    }
                }
            }
            Event::Text(text) => {
                if in_metadata {
                    extracted_frontmatter.push_str(&text);
                } else if in_table {
                    current_cell_text.push_str(&text);
                } else if in_code_block {
                    code_block_accumulator.push_str(&text);
                } else if in_image {
                    image_alt_text.push_str(&text);
                } else if in_link {
                    link_text_accumulator.push_str(&text);
                    let start_offset = buffer.end_iter().offset();
                    buffer.insert(&mut buffer.end_iter(), &text);
                    let start_iter = buffer.iter_at_offset(start_offset);
                    let end_iter = buffer.end_iter();

                    for tag_name in &active_tags {
                        if let Some(tag) = buffer.tag_table().lookup(tag_name) {
                            buffer.apply_tag(&tag, &start_iter, &end_iter);
                        }
                    }
                } else {
                    let start_offset = buffer.end_iter().offset();
                    buffer.insert(&mut buffer.end_iter(), &text);
                    let start_iter = buffer.iter_at_offset(start_offset);
                    let end_iter = buffer.end_iter();

                    for tag_name in &active_tags {
                        if let Some(tag) = buffer.tag_table().lookup(tag_name) {
                            buffer.apply_tag(&tag, &start_iter, &end_iter);
                        }
                    }
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if in_metadata {
                    extracted_frontmatter.push('\n');
                } else if in_table {
                    current_cell_text.push(' ');
                } else if in_code_block {
                    code_block_accumulator.push('\n');
                } else {
                    buffer.insert(&mut buffer.end_iter(), "\n");
                }
            }
            Event::Start(Tag::Paragraph) => {
                if !in_metadata && !in_code_block && !in_image && !in_table {
                    active_tags.push("paragraph".to_string());
                }
            }
            Event::End(TagEnd::Paragraph) => {
                if !in_metadata && !in_code_block && !in_image && !in_table {
                    active_tags.retain(|t| t != "paragraph");
                    if !in_list_item {
                        buffer.insert(&mut buffer.end_iter(), "\n");
                    }
                }
            }
            _ => {}
        }
    }

    if extracted_frontmatter.is_empty() {
        None
    } else {
        Some(extracted_frontmatter.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::renderers::resize_all_images_in_buffer;
    use gtk4::Box as GtkBox;
    use std::fs;

    fn init_gtk_for_tests() -> bool {
        if gtk4::is_initialized() {
            return gtk4::is_initialized_main_thread() && gdk4::Display::default().is_some();
        }
        gtk4::init().is_ok() && gdk4::Display::default().is_some()
    }

    #[test]
    fn test_gfm_table_serialization_and_deserialization_roundtrip() {
        let input_md = "| Header 1 | Header 2 |\n| :--- | :---: |\n| Cell 1 | Cell 2 |\n";
        if init_gtk_for_tests() {
            let buffer = TextBuffer::new(None);
            let text_view = TextView::new();
            let _ = parse_markdown_to_buffer(input_md, &buffer, &text_view, None);
            let serialized = crate::markdown::serialize_buffer_to_markdown(&buffer, None);
            assert!(serialized.contains("Header 1"));
            assert!(serialized.contains("Cell 1"));
            assert!(serialized.contains(":---"));
        }
    }

    #[test]
    fn test_gfm_task_item_serialization_and_deserialization_roundtrip() {
        let input_md = "- [ ] Task 1\n- [x] Task 2\n";
        if init_gtk_for_tests() {
            let buffer = TextBuffer::new(None);
            let text_view = TextView::new();
            let _ = parse_markdown_to_buffer(input_md, &buffer, &text_view, None);
            let serialized = crate::markdown::serialize_buffer_to_markdown(&buffer, None);
            assert!(serialized.contains("- [ ] Task 1"));
            assert!(serialized.contains("- [x] Task 2"));
        }
    }

    #[test]
    fn test_image_asset_file_setup() {
        let notes_dir = std::path::Path::new("/home/ejangi/Sites/gnotes/.notes");
        let assets_dir = notes_dir.join("Image Features Test.assets");
        fs::create_dir_all(&assets_dir).unwrap();

        let sample_png: &[u8] = &[
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00,
            0x00, 0x90, 0x77, 0x53, 0xde, 0x00, 0x00, 0x00, 0x0c, 0x49, 0x44, 0x41, 0x54, 0x08,
            0xd7, 0x63, 0xf8, 0xcf, 0xc0, 0x00, 0x00, 0x03, 0x01, 0x01, 0x00, 0x18, 0xdd, 0x8d,
            0xb0, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
        ];

        let img_path = assets_dir.join("sample.png");
        fs::write(&img_path, sample_png).unwrap();
        assert!(img_path.exists());
    }

    #[test]
    fn test_parse_agile_note() {
        let path = std::path::Path::new("/home/ejangi/iCloud Drive/Documents/Notes/Agile.md");
        if path.exists() {
            let content = fs::read_to_string(path).unwrap();
            if init_gtk_for_tests() {
                let buffer = TextBuffer::new(None);
                let text_view = TextView::new();
                let parent_dir = path.parent();
                let fm = parse_markdown_to_buffer(&content, &buffer, &text_view, parent_dir);
                let serialized =
                    crate::markdown::serialize_buffer_to_markdown(&buffer, fm.as_deref());
                assert!(!serialized.is_empty());
            }
        }
    }

    #[test]
    fn test_encode_asset_urls_in_markdown() {
        let input = "![Peach](_Test.assets/no-bg 4.png)";
        let output = encode_asset_urls_in_markdown(input);
        assert_eq!(output.trim(), "![Peach](_Test.assets/no-bg%204.png)");
    }

    #[test]
    fn test_pulldown_cmark_image_events() {
        let raw_input = "# _Test\nThis is my test note\n![Peach](_Test.assets/no-bg 4.png)\n";
        let sanitized = encode_asset_urls_in_markdown(raw_input);
        println!("SANITIZED: {:?}", sanitized);

        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
        options.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);

        let parser = pulldown_cmark::Parser::new_ext(&sanitized, options);
        for ev in parser {
            println!("EV: {:?}", ev);
        }
    }

    #[test]
    fn test_parse_test_md_with_image() {
        if init_gtk_for_tests() {
            let buffer = TextBuffer::new(None);
            let text_view = TextView::new();
            let notes_dir = std::path::Path::new("/home/ejangi/Documents/Notes");
            let content = "# _Test\nThis is my test note\n![Peach](_Test.assets/no-bg 4.png)\n";
            parse_markdown_to_buffer(content, &buffer, &text_view, Some(notes_dir));
            let text = buffer
                .text(&buffer.start_iter(), &buffer.end_iter(), true)
                .to_string();
            println!("BUFFER TEXT:\n{:?}", text);
            // Verify child anchor exists
            let mut iter = buffer.start_iter();
            let mut found_anchor = false;
            while iter.offset() < buffer.end_iter().offset() {
                if let Some(anchor) = iter.child_anchor() {
                    for widget in anchor.widgets() {
                        if widget.widget_name().starts_with("IMG|") {
                            found_anchor = true;
                            break;
                        }
                    }
                }
                if !iter.forward_char() {
                    break;
                }
            }
            assert!(
                found_anchor,
                "Image anchor widget should be present in buffer!"
            );
        }
    }

    #[test]
    fn test_resize_all_images_in_buffer() {
        if init_gtk_for_tests() {
            let buffer = TextBuffer::new(None);
            let text_view = TextView::new();
            let mut iter = buffer.end_iter();

            let notes_dir = std::path::Path::new("/home/ejangi/Documents/Notes");
            render_image_widget(
                &buffer,
                &text_view,
                &mut iter,
                "_Test.assets/no-bg 4.png",
                "Peach",
                Some(notes_dir),
            );

            resize_all_images_in_buffer(&buffer, 400);

            let mut check_iter = buffer.start_iter();
            let mut found_resized_picture = false;
            while check_iter.offset() < buffer.end_iter().offset() {
                if let Some(anchor) = check_iter.child_anchor() {
                    for widget in anchor.widgets() {
                        if widget.has_css_class("note-image-container") {
                            if let Ok(container_box) = widget.clone().downcast::<GtkBox>() {
                                let mut img_child = container_box.first_child();
                                while let Some(ic) = img_child {
                                    if let Ok(picture) = ic.clone().downcast::<gtk4::Picture>() {
                                        assert_eq!(picture.width_request(), 400);
                                        found_resized_picture = true;
                                    }
                                    img_child = ic.next_sibling();
                                }
                            }
                        }
                    }
                }
                if !check_iter.forward_char() {
                    break;
                }
            }
            assert!(found_resized_picture, "Picture should be resized to 400px!");
        }
    }

    #[test]
    fn test_list_item_serialization_and_deserialization_roundtrip() {
        if init_gtk_for_tests() {
            let buffer = TextBuffer::new(None);
            let text_view = TextView::new();

            let md = "- Item 1\n- Item 2\n";
            parse_markdown_to_buffer(md, &buffer, &text_view, None);

            let line_count = buffer.line_count();
            assert_eq!(line_count, 3, "Buffer should have 3 lines for two items!");

            let serialized = crate::markdown::serialize_buffer_to_markdown(&buffer, None);
            assert!(serialized.contains("- Item 1\n- Item 2"));
        }
    }
}
