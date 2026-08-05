use crate::domain::Note;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct StorageManager {
    notes_dir: PathBuf,
}

impl StorageManager {
    pub fn new<P: AsRef<Path>>(notes_dir: P) -> io::Result<Self> {
        let dir = notes_dir.as_ref().to_path_buf();
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }
        Ok(Self { notes_dir: dir })
    }

    pub fn notes_dir(&self) -> &Path {
        &self.notes_dir
    }

    /// List all `.md` notes, excluding `.assets` folders and non-`.md` files.
    pub fn list_notes(&self) -> io::Result<Vec<Note>> {
        let mut notes = Vec::new();
        if !self.notes_dir.exists() {
            return Ok(notes);
        }

        for entry in fs::read_dir(&self.notes_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Filter out directories (including .assets folders)
            if path.is_dir() {
                continue;
            }

            // Filter for .md extension and non-hidden files
            if let Some(ext) = path.extension() {
                if ext == "md" {
                    if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
                        if !file_name.starts_with('.') {
                            if let Ok(content) = fs::read_to_string(&path) {
                                let modified_at = entry
                                    .metadata()
                                    .and_then(|m| m.modified())
                                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                                notes.push(Note::with_modified_at(path, content, modified_at));
                            }
                        }
                    }
                }
            }
        }

        // Sort notes by last modified date in descending order, title as tie-breaker
        notes.sort_by(|a, b| {
            b.modified_at
                .cmp(&a.modified_at)
                .then_with(|| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
        });
        Ok(notes)
    }

    /// Create a new note with a title and optional initial content.
    pub fn create_note(&self, title: &str, initial_body: &str) -> io::Result<Note> {
        let title_clean = if title.trim().is_empty() {
            "New Note"
        } else {
            title.trim()
        };

        let content = if initial_body.trim().is_empty() {
            "# \n\n".to_string()
        } else if !initial_body.trim_start().starts_with('#') {
            format!("# {}\n\n{}", title_clean, initial_body)
        } else {
            initial_body.to_string()
        };

        let filename = Note::sanitize_filename(title_clean);
        let mut file_path = self.notes_dir.join(&filename);

        // Ensure unique filename if collision occurs
        let mut counter = 1;
        while file_path.exists() {
            let stem = Path::new(&filename)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("New Note");
            file_path = self.notes_dir.join(format!("{} {}.md", stem, counter));
            counter += 1;
        }

        fs::write(&file_path, &content)?;
        let modified_at = fs::metadata(&file_path)
            .and_then(|m| m.modified())
            .unwrap_or_else(|_| std::time::SystemTime::now());
        Ok(Note::with_modified_at(file_path, content, modified_at))
    }

    /// Save updated content to disk, handling note title changes & file renames.
    pub fn save_note(&self, note: &mut Note, new_content: &str) -> io::Result<()> {
        let new_title = Note::extract_title(new_content, &note.file_path);
        let title_changed = new_title != note.title;

        if title_changed {
            let _old_filename = note
                .file_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            let new_filename = Note::sanitize_filename(&new_title);
            let new_file_path = self.notes_dir.join(&new_filename);

            // Rename .md file if target path doesn't conflict
            if !new_file_path.exists() || new_file_path == note.file_path {
                if note.file_path.exists() {
                    fs::rename(&note.file_path, &new_file_path)?;
                }

                // Rename corresponding .assets directory if it exists
                let old_assets_name = Note::assets_dir_name(&note.title);
                let new_assets_name = Note::assets_dir_name(&new_title);
                let old_assets_path = self.notes_dir.join(&old_assets_name);
                let new_assets_path = self.notes_dir.join(&new_assets_name);

                if old_assets_path.exists() && old_assets_path.is_dir() {
                    let _ = fs::rename(old_assets_path, new_assets_path);
                }

                note.file_path = new_file_path;
                note.id = new_filename;
            }
        }

        fs::write(&note.file_path, new_content)?;
        note.content = new_content.to_string();
        note.title = new_title;
        note.modified_at = fs::metadata(&note.file_path)
            .and_then(|m| m.modified())
            .unwrap_or_else(|_| std::time::SystemTime::now());

        Ok(())
    }

    /// Delete note and its associated .assets directory if present.
    pub fn delete_note(&self, note: &Note) -> io::Result<()> {
        if note.file_path.exists() {
            fs::remove_file(&note.file_path)?;
        }

        let assets_name = Note::assets_dir_name(&note.title);
        let assets_path = self.notes_dir.join(assets_name);
        if assets_path.exists() && assets_path.is_dir() {
            let _ = fs::remove_dir_all(assets_path);
        }

        Ok(())
    }

    /// Ensure the `<note title>.assets` directory exists for asset storage.
    pub fn ensure_assets_dir(&self, note_title: &str) -> io::Result<PathBuf> {
        let assets_name = Note::assets_dir_name(note_title);
        let assets_path = self.notes_dir.join(assets_name);
        if !assets_path.exists() {
            fs::create_dir_all(&assets_path)?;
        }
        Ok(assets_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_test_dir(test_name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("peach_notes_test_{}", test_name));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn test_create_and_list_notes() {
        let test_dir = setup_test_dir("create_list");
        let manager = StorageManager::new(&test_dir).unwrap();

        manager.create_note("First Note", "Body 1").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));
        manager.create_note("Second Note", "Body 2").unwrap();

        // Create an assets folder to test filtering
        fs::create_dir_all(test_dir.join("First Note.assets")).unwrap();

        let notes = manager.list_notes().unwrap();
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0].title, "Second Note");
        assert_eq!(notes[1].title, "First Note");
    }

    #[test]
    fn test_create_empty_note_heading() {
        let test_dir = setup_test_dir("create_empty");
        let manager = StorageManager::new(&test_dir).unwrap();

        let note = manager.create_note("New Note", "").unwrap();
        assert_eq!(note.title, "New Note");
        assert_eq!(note.content, "# \n\n");
    }

    #[test]
    fn test_save_and_rename_note() {
        let test_dir = setup_test_dir("rename_note");
        let manager = StorageManager::new(&test_dir).unwrap();

        let mut note = manager
            .create_note("Original Title", "Some content")
            .unwrap();
        let old_path = note.file_path.clone();

        // Ensure assets directory is created
        let assets_dir = manager.ensure_assets_dir("Original Title").unwrap();
        assert!(assets_dir.exists());

        // Update note title to "Renamed Title"
        manager
            .save_note(&mut note, "# Renamed Title\n\nUpdated content")
            .unwrap();

        assert_ne!(note.file_path, old_path);
        assert!(note.file_path.exists());
        assert!(!old_path.exists());

        // Verify assets directory was renamed
        let new_assets_dir = test_dir.join("Renamed Title.assets");
        assert!(new_assets_dir.exists());
    }

    #[test]
    fn test_delete_note_with_assets() {
        let test_dir = setup_test_dir("delete_note");
        let manager = StorageManager::new(&test_dir).unwrap();

        let note = manager.create_note("Note To Delete", "Content").unwrap();
        let assets_dir = manager.ensure_assets_dir("Note To Delete").unwrap();

        assert!(note.file_path.exists());
        assert!(assets_dir.exists());

        manager.delete_note(&note).unwrap();

        assert!(!note.file_path.exists());
        assert!(!assets_dir.exists());
    }

    #[test]
    fn test_list_notes_sorted_by_modified_date_descending() {
        let test_dir = setup_test_dir("modified_sorting");
        let manager = StorageManager::new(&test_dir).unwrap();

        let mut note1 = manager.create_note("Alpha Note", "Content 1").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));
        let _note2 = manager.create_note("Beta Note", "Content 2").unwrap();

        // Initially Beta Note is newer than Alpha Note
        let notes = manager.list_notes().unwrap();
        assert_eq!(notes[0].title, "Beta Note");
        assert_eq!(notes[1].title, "Alpha Note");

        // Now save Alpha Note so its modified timestamp becomes the newest
        std::thread::sleep(std::time::Duration::from_millis(50));
        manager
            .save_note(&mut note1, "# Alpha Note\n\nUpdated Content 1")
            .unwrap();

        let updated_notes = manager.list_notes().unwrap();
        assert_eq!(updated_notes[0].title, "Alpha Note");
        assert_eq!(updated_notes[1].title, "Beta Note");
    }
}
