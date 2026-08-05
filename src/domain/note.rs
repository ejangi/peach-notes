use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub file_path: PathBuf,
    pub content: String,
    pub modified_at: SystemTime,
}

impl Note {
    pub fn new<P: AsRef<Path>>(file_path: P, content: String) -> Self {
        Self::with_modified_at(file_path, content, SystemTime::now())
    }

    pub fn with_modified_at<P: AsRef<Path>>(
        file_path: P,
        content: String,
        modified_at: SystemTime,
    ) -> Self {
        let path_buf = file_path.as_ref().to_path_buf();
        let title = Self::extract_title(&content, &path_buf);
        let id = path_buf
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("untitled")
            .to_string();

        Self {
            id,
            title,
            file_path: path_buf,
            content,
            modified_at,
        }
    }

    /// Extract title from frontmatter `title:` key, or first `# H1` heading, or fallback to file stem.
    pub fn extract_title(content: &str, file_path: &Path) -> String {
        let (frontmatter, main_content) = Self::split_frontmatter(content);

        // Check if title is specified in YAML frontmatter (e.g. title: "My Title")
        if let Some(fm) = frontmatter {
            for line in fm.lines() {
                let trimmed = line.trim();
                if let Some(val) = trimmed.strip_prefix("title:") {
                    let unquoted = val.trim().trim_matches(|c| c == '"' || c == '\'');
                    if !unquoted.is_empty() {
                        return unquoted.to_string();
                    }
                }
            }
        }

        // Search for first `# H1` heading in main content
        for line in main_content.lines() {
            let trimmed = line.trim();
            if let Some(heading) = trimmed.strip_prefix("# ") {
                let heading_trim = heading.trim();
                if !heading_trim.is_empty() {
                    return heading_trim.to_string();
                }
            }
        }

        // Fallback to filename stem
        file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled Note")
            .to_string()
    }

    /// Split content into optional (frontmatter_body, main_markdown_body)
    pub fn split_frontmatter(content: &str) -> (Option<&str>, &str) {
        let trimmed = content.trim_start();
        if let Some(rest) = trimmed.strip_prefix("---") {
            if let Some(end_idx) = rest.find("\n---") {
                let fm_content = &rest[..end_idx];
                let main_body = &rest[end_idx + 4..];
                return (Some(fm_content.trim()), main_body.trim_start());
            } else if let Some(end_idx) = rest.find("\n...") {
                let fm_content = &rest[..end_idx];
                let main_body = &rest[end_idx + 4..];
                return (Some(fm_content.trim()), main_body.trim_start());
            }
        }
        (None, content)
    }

    /// Sanitize a note title into a safe filename (e.g. "Shopping List" -> "Shopping List.md")
    pub fn sanitize_filename(title: &str) -> String {
        let sanitized: String = title
            .chars()
            .map(|c| match c {
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
                _ => c,
            })
            .collect();
        let trimmed = sanitized.trim();
        if trimmed.is_empty() {
            "Untitled Note.md".to_string()
        } else {
            format!("{}.md", trimmed)
        }
    }

    /// Check if a path or filename extension corresponds to a supported image type
    pub fn is_image_file<P: AsRef<Path>>(path: P) -> bool {
        let path = path.as_ref();
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            let lower = ext.to_lowercase();
            matches!(
                lower.as_str(),
                "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp"
            )
        } else {
            false
        }
    }

    /// Format byte size into human readable string (e.g., 1.2 MB, 450 KB)
    pub fn format_file_size(bytes: u64) -> String {
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }

    /// Return the corresponding assets directory name (e.g. "Shopping List.assets")
    pub fn assets_dir_name(title: &str) -> String {
        let sanitized_filename = Self::sanitize_filename(title);
        let stem = Path::new(&sanitized_filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled Note");
        format!("{}.assets", stem)
    }

    /// Extract a clean text preview summary (skipping frontmatter and title line)
    pub fn preview(&self) -> String {
        let (_, main_content) = Self::split_frontmatter(&self.content);

        let lines: Vec<&str> = main_content
            .lines()
            .filter(|line| !line.trim().starts_with("# "))
            .collect();

        let joined = lines.join(" ");
        let cleaned: String = joined.split_whitespace().collect::<Vec<&str>>().join(" ");
        if cleaned.is_empty() {
            "No additional text".to_string()
        } else if cleaned.chars().count() > 80 {
            format!("{}...", cleaned.chars().take(80).collect::<String>())
        } else {
            cleaned
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_title_extraction_from_h1() {
        let content = "# My Great Note\nSome body text here";
        let path = Path::new("/tmp/test.md");
        assert_eq!(Note::extract_title(content, path), "My Great Note");
    }

    #[test]
    fn test_title_extraction_from_frontmatter() {
        let content =
            "---\ntitle: \"Frontmatter Title\"\ndate: 2026-07-25\n---\n\n# Heading 1\nBody";
        let path = Path::new("/tmp/test.md");
        assert_eq!(Note::extract_title(content, path), "Frontmatter Title");
    }

    #[test]
    fn test_title_extraction_after_frontmatter_without_title_key() {
        let content = "---\ndate: 2026-07-25\ntags: [rust]\n---\n\n# Note Title Below FM\nBody";
        let path = Path::new("/tmp/test.md");
        assert_eq!(Note::extract_title(content, path), "Note Title Below FM");
    }

    #[test]
    fn test_title_extraction_fallback_to_filename() {
        let content = "Just body text with no H1 heading";
        let path = Path::new("/tmp/Project Ideas.md");
        assert_eq!(Note::extract_title(content, path), "Project Ideas");
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(Note::sanitize_filename("Shopping List"), "Shopping List.md");
        assert_eq!(
            Note::sanitize_filename("Title/With:Invalid*Chars"),
            "Title_With_Invalid_Chars.md"
        );
    }

    #[test]
    fn test_assets_dir_name() {
        assert_eq!(
            Note::assets_dir_name("Shopping List"),
            "Shopping List.assets"
        );
    }

    #[test]
    fn test_is_image_file() {
        assert!(Note::is_image_file("photo.png"));
        assert!(Note::is_image_file("image.JPG"));
        assert!(Note::is_image_file("vector.svg"));
        assert!(Note::is_image_file("banner.webp"));
        assert!(!Note::is_image_file("document.pdf"));
        assert!(!Note::is_image_file("archive.zip"));
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(Note::format_file_size(512), "512 B");
        assert_eq!(Note::format_file_size(1536), "1.5 KB");
        assert_eq!(Note::format_file_size(2_500_000), "2.4 MB");
    }

    #[test]
    fn test_preview_generation_with_frontmatter() {
        let note = Note::new(
            Path::new("/tmp/note.md"),
            "---\ntitle: Test\n---\n# Title\nThis is preview text after frontmatter.".to_string(),
        );
        assert_eq!(note.preview(), "This is preview text after frontmatter.");
    }
}
