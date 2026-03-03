use chrono::{DateTime, Utc};
use rusqlite::{Connection, Error, types::Type::Text};
use uuid::Uuid;

use crate::{
    MindSafeApp,
    models::{config::SortingScheme, note::Note},
    services::{
        database::DatabaseService,
        encryption::{CryptoError, EncryptedBlob, EncryptionService},
        hash::HashService,
    },
};

pub struct NoteService {}

impl NoteService {
    /// Blob will only be fetched when the note is active in a tab.
    pub fn get_all_notes(conn: &Connection, sorting: &SortingScheme) -> Result<Vec<Note>, Error> {
        // Will only get blob when the user clicks on the note tile
        let query = format!(
            "SELECT id, title, blob, sha_hash, starred, disable_auto_save, created_at, updated_at FROM notes {}",
            sorting.order_by()
        );
        let mut stmt = conn.prepare(&query)?;

        let note_iter = stmt.query_map([], |row| {
            Ok(Note {
                id: {
                    let id_str: String = row.get(0)?;
                    Uuid::parse_str(&id_str)
                        .map_err(|e| Error::FromSqlConversionFailure(0, Text, Box::new(e)))?
                },
                title: row.get::<_, Option<String>>(1)?.unwrap_or_else(String::new),
                blob: row.get::<_, Option<String>>(2)?.unwrap_or_else(String::new),
                sha_hash: row.get::<_, Option<String>>(3)?.unwrap_or_else(String::new),
                text: String::new(),
                new_sha: String::new(),
                tags: None,
                starred: None,
                created_at: row.get::<_, DateTime<Utc>>(6)?,
                updated_at: row.get::<_, DateTime<Utc>>(7)?,
            })
        })?;

        // Collect all rows into a Vec
        note_iter.collect()
    }

    // get and decrypts the blob
    pub(crate) fn get_decrypted_blob(
        app: &MindSafeApp,
        note: &mut Note,
    ) -> Result<String, CryptoError> {
        if let Some(database_service) = &app.database_service {
            note.get_blob(database_service.get_connection());
            match EncryptedBlob::deserialize(&note.blob) {
                Ok(blob) => {
                    match EncryptionService::decrypt_with_key(&app.file_key, &note.id, &blob) {
                        Ok(text) => Ok(String::from_utf8_lossy(&text).to_string()),
                        Err(e) => {
                            println!("Failed to decrypt the blob: {e}");
                            Err(e)
                        }
                    }
                }
                Err(e) => {
                    println!("Failed to deserialize the blob: {e}");
                    Err(e)
                }
            }
        } else {
            Err(CryptoError::Serde(
                "Database not found! Restart the app!".into(),
            ))
        }
    }

    // get and decrypts the blob
    pub(crate) fn get_decrypted_text(
        file_key: &[u8],
        note: &mut Note,
    ) -> Result<String, CryptoError> {
        match EncryptedBlob::deserialize(&note.blob) {
            Ok(blob) => match EncryptionService::decrypt_with_key(file_key, &note.id, &blob) {
                Ok(text) => Ok(String::from_utf8_lossy(&text).to_string()),
                Err(e) => {
                    println!("Failed to decrypt the blob: {e}");
                    Err(e)
                }
            },
            Err(e) => {
                println!("Failed to deserialize the blob: {e}");
                Err(e)
            }
        }
    }

    // Create sha_hash and save encrypted blob
    pub(crate) fn save_note_hash_blob(
        database_service: &Option<DatabaseService>,
        file_key: &[u8],
        note: &mut Note,
    ) -> Result<bool, CryptoError> {
        if let Some(database_service) = database_service {
            let new_sha = HashService::generate_hash(&note.text); // Regenerating hash, for safety

            if new_sha == note.sha_hash {
                return Ok(false);
            }

            note.sha_hash = HashService::generate_hash(&note.text); // Regenerating hash, for safety

            match EncryptionService::encrypt_with_key(file_key, &note.id, note.text.as_ref()) {
                Ok(blob) => match blob.serialize() {
                    Ok(serialized_text) => {
                        note.blob = serialized_text;
                        let conn = database_service.get_connection();
                        note.update_blob(conn); // saves both blob and hash
                        println!("Saved Blob, on id: {}", note.id);

                        let _ = note.save_history(conn);

                        Ok(true)
                    }
                    Err(e) => {
                        println!("Failed to serialize the encrypted blob: {e}");
                        Err(e)
                    }
                },
                Err(e) => {
                    println!("Failed to encrypt the text: {e}");
                    Err(e)
                }
            }
        } else {
            Err(CryptoError::Serde(
                "Database not found! Restart the app!".into(),
            ))
        }
    }

    // saves sha_hash and save encrypted blob notes history

    // get sha_hash and decrypts the blob for a note from notes history
}
