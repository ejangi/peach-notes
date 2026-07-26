use gtk4::prelude::*;
use gtk4::{Label, TextBuffer};

pub fn serialize_buffer_to_markdown(buffer: &TextBuffer, frontmatter: Option<&str>) -> String {
    let mut result = String::new();

    if let Some(fm) = frontmatter {
        if !fm.trim().is_empty() {
            result.push_str("---\n");
            result.push_str(fm.trim());
            result.push_str("\n---\n\n");
        }
    }

    let line_count = buffer.line_count();

    for line_idx in 0..line_count {
        if let Some(line_start) = buffer.iter_at_line(line_idx) {
            let mut line_end = line_start;
            if !line_end.ends_line() {
                line_end.forward_to_line_end();
            }

            let mut handled = false;
            let mut check_iter = line_start;
            while check_iter.offset() <= line_end.offset() {
                if let Some(anchor) = check_iter.child_anchor() {
                    for widget in anchor.widgets() {
                        let name = widget.widget_name();
                        if name.starts_with("IMG|") {
                            let parts: Vec<&str> = name.splitn(3, '|').collect();
                            if parts.len() >= 2 {
                                let url = parts[1];
                                let alt = if parts.len() == 3 { parts[2] } else { "" };
                                result.push_str(&format!("![{}]({})\n", alt, url));
                                handled = true;
                                break;
                            }
                        } else if name.starts_with("ATTACHMENT|") {
                            let parts: Vec<&str> = name.splitn(3, '|').collect();
                            if parts.len() >= 2 {
                                let url = parts[1];
                                let text = if parts.len() == 3 {
                                    parts[2]
                                } else {
                                    "Attachment"
                                };
                                let display_text = if text.starts_with("📎 ") {
                                    text.to_string()
                                } else {
                                    format!("📎 {}", text)
                                };
                                result.push_str(&format!("[{}]({})\n", display_text, url));
                                handled = true;
                                break;
                            }
                        } else if name.starts_with("TABLE|") {
                            let json_str = name.trim_start_matches("TABLE|");
                            if let Ok(table_data) =
                                serde_json::from_str::<crate::markdown::TableData>(json_str)
                            {
                                result.push_str(&table_data.to_markdown());
                                handled = true;
                                break;
                            }
                        } else if name.starts_with("TASK|") {
                            if name == "TASK|[x]" {
                                result.push_str("- [x] ");
                            } else {
                                result.push_str("- [ ] ");
                            }
                            // Don't mark handled = true so the line text after task anchor is serialized
                        } else if let Ok(container) = widget.clone().downcast::<Label>() {
                            let code_text = container.text().to_string();
                            if !code_text.is_empty() {
                                result.push_str("```\n");
                                result.push_str(&code_text);
                                result.push_str("\n```\n");
                                handled = true;
                                break;
                            }
                        } else if let Ok(container) = widget.downcast::<gtk4::Box>() {
                            if let Some(first_child) = container.first_child() {
                                if let Ok(label) = first_child.downcast::<Label>() {
                                    let code_text = label.text().to_string();
                                    if !code_text.is_empty() {
                                        result.push_str("```\n");
                                        result.push_str(&code_text);
                                        result.push_str("\n```\n");
                                        handled = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                if handled || !check_iter.forward_char() || check_iter.offset() > line_end.offset()
                {
                    break;
                }
            }

            if handled {
                continue;
            }

            let raw_line_text = buffer.text(&line_start, &line_end, true).to_string();
            let line_text = raw_line_text.trim_start_matches('\u{FFFC}').to_string();
            if line_text.is_empty() && line_idx < line_count - 1 {
                result.push('\n');
                continue;
            }

            let tags = line_start.tags();
            let is_h1 = tags
                .iter()
                .any(|t| t.name().as_deref() == Some("heading-1"));
            let is_h2 = tags
                .iter()
                .any(|t| t.name().as_deref() == Some("heading-2"));
            let is_h3 = tags
                .iter()
                .any(|t| t.name().as_deref() == Some("heading-3"));
            let is_bullet = tags
                .iter()
                .any(|t| t.name().as_deref() == Some("bullet-list"));
            let is_quote = tags
                .iter()
                .any(|t| t.name().as_deref() == Some("blockquote"));

            if is_h1 && !line_text.starts_with("# ") {
                result.push_str("# ");
            } else if is_h2 && !line_text.starts_with("## ") {
                result.push_str("## ");
            } else if is_h3 && !line_text.starts_with("### ") {
                result.push_str("### ");
            } else if is_bullet && !line_text.starts_with("- ") && !line_text.starts_with("• ") {
                result.push_str("- ");
            } else if is_quote && !line_text.starts_with("> ") {
                result.push_str("> ");
            }

            let (clean_line_text, prefix_char_len) =
                if let Some(stripped) = line_text.strip_prefix("• ") {
                    result.push_str("- ");
                    (stripped, "• ".chars().count())
                } else if let Some(stripped) = line_text.strip_prefix("☑ ") {
                    result.push_str("- [x] ");
                    (stripped, "☑ ".chars().count())
                } else if let Some(stripped) = line_text.strip_prefix("☐ ") {
                    result.push_str("- [ ] ");
                    (stripped, "☐ ".chars().count())
                } else {
                    (&line_text[..], 0)
                };

            result.push_str(&serialize_line_inline(
                buffer,
                &line_start,
                &line_end,
                clean_line_text,
                prefix_char_len,
            ));

            if line_idx < line_count - 1 {
                result.push('\n');
            }
        }
    }

    result
}

fn serialize_line_inline(
    buffer: &TextBuffer,
    line_start: &gtk4::TextIter,
    line_end: &gtk4::TextIter,
    raw_text: &str,
    prefix_char_len: usize,
) -> String {
    if raw_text.is_empty() {
        return String::new();
    }

    let mut line_out = String::new();
    let mut curr_iter = *line_start;

    if prefix_char_len > 0 {
        curr_iter.forward_chars(prefix_char_len as i32);
    }

    while curr_iter.offset() < line_end.offset() {
        let mut next_iter = curr_iter;
        if !next_iter.forward_char() || next_iter.offset() <= curr_iter.offset() {
            break;
        }

        let slice = buffer.text(&curr_iter, &next_iter, true).to_string();
        let tags = curr_iter.tags();

        let is_bold = tags.iter().any(|t| t.name().as_deref() == Some("bold"));
        let is_italic = tags.iter().any(|t| t.name().as_deref() == Some("italic"));
        let is_code = tags
            .iter()
            .any(|t| t.name().as_deref() == Some("monospace"));
        let is_cb = tags
            .iter()
            .any(|t| t.name().as_deref() == Some("code-block"));
        let is_strike = tags
            .iter()
            .any(|t| t.name().as_deref() == Some("strikethrough"));
        let link_tag = tags.iter().find(|t| {
            let name = t.name();
            let s = name.as_deref().unwrap_or("");
            s == "link" || s.starts_with("link:")
        });

        if let Some(lt) = link_tag {
            let name = lt.name().unwrap();
            let url = name.strip_prefix("link:").unwrap_or("");
            if !slice.is_empty() {
                if url.is_empty() {
                    line_out.push_str(&slice);
                } else {
                    line_out.push_str(&format!("[{}]({})", slice, url));
                }
            }
        } else if is_code || is_cb {
            line_out.push_str(&slice);
        } else {
            if is_bold {
                line_out.push_str("**");
            }
            if is_italic {
                line_out.push('*');
            }
            if is_strike {
                line_out.push_str("~~");
            }

            line_out.push_str(&slice);

            if is_strike {
                line_out.push_str("~~");
            }
            if is_italic {
                line_out.push('*');
            }
            if is_bold {
                line_out.push_str("**");
            }
        }

        curr_iter = next_iter;
    }

    collapse_contiguous_links(&line_out)
}

fn collapse_contiguous_links(input: &str) -> String {
    let mut result = String::new();
    let mut remaining = input;

    while !remaining.is_empty() {
        if let Some(start_idx) = remaining.find('[') {
            result.push_str(&remaining[..start_idx]);
            let rest = &remaining[start_idx..];

            let mut combined_text = String::new();
            let mut target_url = None;
            let mut curr_rest = rest;
            let mut matched_len = 0;

            while curr_rest.starts_with('[') {
                if let Some(close_bracket) = curr_rest.find("](") {
                    if let Some(close_paren) = curr_rest[close_bracket..].find(')') {
                        let end_paren_idx = close_bracket + close_paren;
                        let text = &curr_rest[1..close_bracket];
                        let url = &curr_rest[close_bracket + 2..end_paren_idx];

                        if target_url.is_none() {
                            target_url = Some(url.to_string());
                            combined_text.push_str(text);
                            matched_len += end_paren_idx + 1;
                            curr_rest = &curr_rest[end_paren_idx + 1..];
                        } else if target_url.as_deref() == Some(url) {
                            combined_text.push_str(text);
                            matched_len += end_paren_idx + 1;
                            curr_rest = &curr_rest[end_paren_idx + 1..];
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            if let Some(url) = target_url {
                if !combined_text.trim().is_empty() {
                    result.push_str(&format!("[{}]({})", combined_text, url));
                }
                remaining = &rest[matched_len..];
            } else {
                result.push('[');
                remaining = &rest[1..];
            }
        } else {
            result.push_str(remaining);
            break;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_backspacing_serialization() {
        let input = "[Google](https://google.com)";
        assert_eq!(
            collapse_contiguous_links(input),
            "[Google](https://google.com)"
        );

        // Shortened text via backspacing
        let input_shortened = "[G](https://google.com)";
        assert_eq!(
            collapse_contiguous_links(input_shortened),
            "[G](https://google.com)"
        );

        // Empty text (all characters deleted)
        let input_empty = "[](https://google.com)";
        assert_eq!(collapse_contiguous_links(input_empty), "");
    }

    fn init_gtk_for_tests() -> bool {
        if gtk4::is_initialized() {
            return gtk4::is_initialized_main_thread() && gdk4::Display::default().is_some();
        }
        gtk4::init().is_ok() && gdk4::Display::default().is_some()
    }

    #[test]
    fn test_image_anchor_serialization() {
        if init_gtk_for_tests() {
            let buffer = TextBuffer::new(None);
            let mut iter = buffer.end_iter();

            let container_box = gtk4::Box::builder().build();
            container_box.set_widget_name("IMG|MyNote.assets/test.png|Test Caption");

            let anchor = buffer.create_child_anchor(&mut iter);
            let text_view = gtk4::TextView::new();
            text_view.add_child_at_anchor(&container_box, &anchor);

            let serialized = serialize_buffer_to_markdown(&buffer, None);
            assert_eq!(serialized.trim(), "![Test Caption](MyNote.assets/test.png)");
        }
    }
}
