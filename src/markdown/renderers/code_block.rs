use gtk4::prelude::*;
use gtk4::{Box as GtkBox, DropDown, Orientation, StringList, TextBuffer, TextView};
use sourceview5::prelude::*;
use sourceview5::{
    Buffer as SourceBuffer, LanguageManager, StyleSchemeManager, View as SourceView,
};

pub fn resolve_language_id(hint: &str) -> Option<String> {
    let normalized = hint.trim().to_lowercase();
    if normalized.is_empty() {
        return None;
    }
    let mapped = match normalized.as_str() {
        "js" => "javascript",
        "py" => "python",
        "rs" => "rust",
        "ts" => "typescript",
        "sh" | "bash" | "zsh" => "sh",
        "yml" => "yaml",
        "md" => "markdown",
        "cs" => "c-sharp",
        "cpp" | "c++" => "cpp",
        "rb" => "ruby",
        other => other,
    };
    let lm = LanguageManager::default();
    if lm.language(mapped).is_some() {
        Some(mapped.to_string())
    } else if lm.language(&normalized).is_some() {
        Some(normalized)
    } else {
        // Fallback: search by language name or mime type or id case-insensitively
        for id in lm.language_ids() {
            if id.to_lowercase() == normalized || id.to_lowercase() == mapped {
                return Some(id.to_string());
            }
            if let Some(lang) = lm.language(&id) {
                if lang.name().to_lowercase() == normalized || lang.name().to_lowercase() == mapped
                {
                    return Some(id.to_string());
                }
            }
        }
        None
    }
}

fn apply_style_scheme(source_buffer: &SourceBuffer) {
    if !gtk4::is_initialized() {
        return;
    }
    let sm = StyleSchemeManager::default();
    let is_dark = libadwaita::StyleManager::default().is_dark();
    let scheme_id = if is_dark { "Adwaita-dark" } else { "Adwaita" };
    if let Some(scheme) = sm.scheme(scheme_id) {
        source_buffer.set_style_scheme(Some(&scheme));
    } else if is_dark {
        if let Some(scheme) = sm.scheme("solarized-dark") {
            source_buffer.set_style_scheme(Some(&scheme));
        }
    } else if let Some(scheme) = sm.scheme("classic") {
        source_buffer.set_style_scheme(Some(&scheme));
    }
}

pub fn render_code_block_widget(
    buffer: &TextBuffer,
    text_view: &TextView,
    iter: &mut gtk4::TextIter,
    code_content: &str,
    lang_hint: Option<&str>,
) -> Option<SourceView> {
    if !gtk4::is_initialized() || gdk4::Display::default().is_none() {
        return None;
    }
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

    let initial_lang_id = lang_hint.and_then(resolve_language_id).unwrap_or_default();

    let container_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .hexpand(true)
        .css_classes(vec!["code-block-container".to_string()])
        .name(format!("CODE_BLOCK|{}", initial_lang_id))
        .build();

    let lm = LanguageManager::default();
    let mut language_entries: Vec<(String, String)> = Vec::new(); // (display_name, lang_id)
    language_entries.push(("Plain Text".to_string(), "".to_string()));

    let mut ids: Vec<String> = lm.language_ids().iter().map(|s| s.to_string()).collect();
    ids.sort();

    for id in ids {
        if let Some(lang) = lm.language(&id) {
            language_entries.push((lang.name().to_string(), id));
        }
    }

    let display_names: Vec<&str> = language_entries
        .iter()
        .map(|(name, _)| name.as_str())
        .collect();
    let string_list = StringList::new(&display_names);

    let initial_idx = language_entries
        .iter()
        .position(|(_, id)| id == &initial_lang_id)
        .unwrap_or(0);

    let expression = gtk4::PropertyExpression::new(
        gtk4::StringObject::static_type(),
        None::<&gtk4::Expression>,
        "string",
    );

    let dropdown = DropDown::builder()
        .model(&string_list)
        .expression(&expression)
        .enable_search(true)
        .selected(initial_idx as u32)
        .halign(gtk4::Align::End)
        .margin_bottom(4)
        .css_classes(vec!["code-block-lang-dropdown".to_string()])
        .build();

    let source_buffer = SourceBuffer::new(None);
    source_buffer.set_text(code_content.trim_end());

    if !initial_lang_id.is_empty() {
        if let Some(lang) = lm.language(&initial_lang_id) {
            source_buffer.set_language(Some(&lang));
        }
    }

    apply_style_scheme(&source_buffer);

    let source_view = SourceView::with_buffer(&source_buffer);
    source_view.set_editable(true);
    source_view.set_focusable(true);
    source_view.set_can_focus(true);
    source_view.set_show_line_numbers(false);
    source_view.set_wrap_mode(gtk4::WrapMode::None);
    source_view.set_monospace(true);
    source_view.set_hexpand(true);
    source_view.set_vexpand(false);
    source_view.add_css_class("code-block-text");

    let gesture = gtk4::GestureClick::new();
    gesture.set_propagation_phase(gtk4::PropagationPhase::Capture);
    let source_view_grab = source_view.clone();
    let parent_buffer_clone = buffer.clone();
    gesture.connect_pressed(move |_, _, _, _| {
        source_view_grab.grab_focus();
        let pos = parent_buffer_clone.cursor_position();
        let iter = parent_buffer_clone.iter_at_offset(pos);
        parent_buffer_clone.place_cursor(&iter);
    });
    source_view.add_controller(gesture);

    let sv_key_controller = gtk4::EventControllerKey::new();
    sv_key_controller.set_propagation_phase(gtk4::PropagationPhase::Capture);
    let parent_buf_key = buffer.clone();
    let parent_tv_key = text_view.clone();
    sv_key_controller.connect_key_pressed(move |_, keyval, _keycode, state| {
        let pos = parent_buf_key.cursor_position();
        let iter = parent_buf_key.iter_at_offset(pos);
        parent_buf_key.place_cursor(&iter);

        if keyval == gdk4::Key::Escape
            || (keyval == gdk4::Key::Return && state.contains(gdk4::ModifierType::SHIFT_MASK))
        {
            parent_tv_key.grab_focus();
            return glib::Propagation::Stop;
        }

        glib::Propagation::Proceed
    });
    source_view.add_controller(sv_key_controller);

    let container_clone = container_box.clone();
    let source_buffer_clone = source_buffer.clone();
    let entries_clone = language_entries.clone();

    dropdown.connect_selected_notify(move |dd| {
        let idx = dd.selected() as usize;
        if let Some((_, lang_id)) = entries_clone.get(idx) {
            container_clone.set_widget_name(&format!("CODE_BLOCK|{}", lang_id));
            if lang_id.is_empty() {
                source_buffer_clone.set_language(None);
            } else if let Some(lang) = LanguageManager::default().language(lang_id) {
                source_buffer_clone.set_language(Some(&lang));
            }
        }
    });

    let source_buffer_style_clone = source_buffer.clone();
    if gtk4::is_initialized() {
        libadwaita::StyleManager::default().connect_dark_notify(move |_| {
            apply_style_scheme(&source_buffer_style_clone);
        });
    }

    let header_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .hexpand(true)
        .build();
    header_box.append(&dropdown);

    container_box.append(&header_box);
    container_box.append(&source_view);

    let tv_w = text_view.width();
    let init_w = if tv_w > 80 { (tv_w - 48).max(200) } else { 600 };
    container_box.set_size_request(init_w, -1);

    let container_box_resize = container_box.clone();
    text_view.connect_notify_local(Some("width"), move |tv, _| {
        let width = tv.width();
        if width > 80 {
            container_box_resize.set_size_request(width - 48, -1);
        }
    });

    text_view.add_child_at_anchor(&container_box, &anchor);

    let start_iter = buffer.iter_at_offset(anchor_offset);
    let end_iter = buffer.end_iter();
    if let Some(tag) = buffer.tag_table().lookup("code-block") {
        buffer.apply_tag(&tag, &start_iter, &end_iter);
    }

    Some(source_view)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_gtk_for_tests() -> bool {
        if gtk4::is_initialized() {
            return gtk4::is_initialized_main_thread() && gdk4::Display::default().is_some();
        }
        gtk4::init().is_ok() && gdk4::Display::default().is_some()
    }

    #[test]
    fn test_resolve_language_id_aliases() {
        if init_gtk_for_tests() {
            assert_eq!(resolve_language_id("rs"), Some("rust".to_string()));
            assert_eq!(resolve_language_id("py"), Some("python".to_string()));
            assert_eq!(resolve_language_id("js"), Some("javascript".to_string()));
            assert_eq!(resolve_language_id(""), None);
        }
    }
}
