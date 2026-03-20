use std::{fs, path::PathBuf};

use crate::{MindSafeApp, models::note::Note, services::notes::NoteService};

pub struct DataService {}

impl DataService {
    fn import_text(path: &PathBuf) -> Result<String, String> {
        // Check if file exists
        if !path.exists() {
            return Err(format!("File not found: {:?}", path));
        }

        // Validate extension
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("md") | Some("txt") => {}
            Some(ext) => return Err(format!("Unsupported file type: .{}", ext)),
            None => return Err("File has no extension".to_string()),
        }

        // Read and return the file contents
        fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))
    }

    pub(crate) fn import_notes(app: &mut MindSafeApp, file_path: PathBuf) {
        match DataService::import_text(&file_path) {
            Ok(note_text) => {
                let file_name: String = file_path
                    .file_stem() // name without extension → Option<&OsStr>
                    .and_then(|s| s.to_str()) // OsStr → Option<&str>
                    .unwrap_or("Imported Note") // fallback if path is malformed
                    .to_string();

                let mut imported_note = Note::new(file_name);
                imported_note.text = note_text;

                match NoteService::save_note_hash_blob(
                    &app.workspace_id,
                    &app.database_service,
                    &app.file_key,
                    &mut imported_note,
                    true,
                ) {
                    Ok(_) => {
                        if cfg!(debug_assertions) {
                            println!("Imported note created!");
                        }
                        app.get_all_notes();
                    }
                    Err(e) => {
                        if cfg!(debug_assertions) {
                            println!("Error while creating imported note: {e}");
                        }
                    }
                }
            }
            Err(e) => {
                if cfg!(debug_assertions) {
                    println!("Note File ({file_path:?}) could not be imported because: {e}")
                }
            }
        }
    }
}
