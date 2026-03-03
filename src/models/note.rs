use chrono::{DateTime, Utc};
use rusqlite::Error;
use rusqlite::{Connection, params};
use uuid::Uuid;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::{constants::HELP_TEXT, models::tag::Tag};

/// Using UUID as Id for all except config to ensure data can be imoported
/// along with all relations intact, as UUID is unique for all
/// devices so there is unlikely two notes will have same id.

#[derive(Debug, Clone)]
pub struct Note {
    pub id: Uuid,
    pub title: String,
    pub blob: String,
    pub sha_hash: String,
    pub text: String,
    pub new_sha: String,
    pub tags: Option<Vec<Tag>>,
    pub starred: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Note {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: name,
            blob: String::new(),
            sha_hash: String::new(),
            text: String::new(),
            new_sha: String::new(),
            tags: None,
            starred: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn refresh_id(&mut self) {
        // re-issuing id to ensure it is unique
        // even though it may not be required, just for extra security
        self.id = Uuid::new_v4();
    }

    pub fn set_default(&mut self) {
        self.id = Uuid::new_v4();
        self.title = String::new();
        self.blob = String::new();
        self.sha_hash = String::new();
        self.text = String::new();
        self.new_sha = String::new();
        self.tags = None;
        self.created_at = Utc::now();
        self.updated_at = Utc::now();
    }

    // ---- DB Functions
    pub fn insert(&self, conn: &Connection) -> bool {
        // NOTE: created_at & updated_at will be set as default by DB
        match conn.execute(
            "INSERT INTO notes (id, title) VALUES (?1, ?2);",
            (&self.id.to_string(), &self.title),
        ) {
            Ok(_) => true,
            Err(e) => {
                println!("Error inserting note to db: {e}");
                false
            }
        }
    }
    pub fn duplicate(&self, conn: &Connection) -> bool {
        // NOTE: created_at & updated_at will be set as default by DB
        match conn.execute(
            "INSERT INTO notes (id, title, blob, sha_hash) VALUES (?1, ?2, ?3, ?4);",
            (
                Uuid::new_v4().to_string(),
                format!("{}-Copy", &self.title),
                &self.blob,
                &self.sha_hash,
            ),
        ) {
            Ok(_) => true,
            Err(e) => {
                println!("Error inserting note to db: {e}");
                false
            }
        }
    }

    pub fn update_title(&mut self, conn: &Connection) -> bool {
        match conn.execute(
            "UPDATE notes SET title = ?1 WHERE id = ?2;",
            (&self.title, &self.id.to_string()),
        ) {
            Ok(_) => {
                self.updated_at = Utc::now();
                true
            }
            Err(e) => {
                println!("Error updating title in notes to db: {e}");
                false
            }
        }
    }

    pub fn get_blob(&mut self, conn: &Connection) {
        let result: rusqlite::Result<String> = conn.query_row(
            "SELECT blob FROM notes WHERE id = ?1;",
            params![self.id.to_string()],
            |row| row.get(0),
        );

        match result {
            Ok(db_blob) => {
                self.blob = db_blob;
                self.updated_at = Utc::now();
            }
            Err(e) => {
                println!("Error fetching blob from notes: {e}");
            }
        }
    }

    pub fn update_blob(&mut self, conn: &Connection) {
        match conn.execute(
            "UPDATE notes SET blob = ?1, sha_hash = ?2 WHERE id = ?3;",
            (&self.blob, &self.sha_hash, &self.id.to_string()),
        ) {
            Ok(_) => {
                self.updated_at = Utc::now();
            }
            Err(e) => {
                println!("Error updating blob in notes to db: {e}");
            }
        }
    }

    pub fn save_history(&mut self, conn: &Connection) -> Result<bool, Error> {
        match conn.execute(
            "INSERT INTO notes_history (id, note_id, blob, sha_hash) VALUES (?1, ?2, ?3, ?4);",
            (
                Uuid::new_v4().to_string(),
                self.id.to_string(),
                &self.blob,
                &self.sha_hash,
            ),
        ) {
            Ok(_) => Ok(true),
            Err(e) => {
                println!("Error inserting note history to db: {e}");
                Err(e)
            }
        }
    }

    /// Hard delete, no need for soft delete
    /// NOTE: Data deleted is non-recoverable intentionally.
    pub fn delete(&mut self, conn: &Connection) -> bool {
        match conn.execute("DELETE FROM notes WHERE id = ?1;", (&self.id.to_string(),)) {
            Ok(_) => {
                self.zeroize();
                true
            }
            Err(e) => {
                println!("Error deleting notes to db: {e}");
                false
            }
        }
    }
    pub fn delete_with_id(note_id: &Uuid, conn: &Connection) -> bool {
        match conn.execute("DELETE FROM notes WHERE id = ?1;", (note_id.to_string(),)) {
            Ok(_) => true,
            Err(e) => {
                println!("Error deleting notes to db: {e}");
                false
            }
        }
    }

    pub fn make_introduction() -> Self {
        Self {
            id: Uuid::new_v4(),
            title: String::new(),
            blob: String::new(),
            sha_hash: String::new(),
            text: HELP_TEXT.into(),
            new_sha: String::new(),
            tags: None,
            starred: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl Default for Note {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            title: String::new(),
            blob: String::new(),
            sha_hash: String::new(),
            text: String::new(),
            new_sha: String::new(),
            tags: None,
            starred: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl Zeroize for Note {
    fn zeroize(&mut self) {
        self.id = Uuid::nil();
        self.title.zeroize();
        self.blob.zeroize();
        self.sha_hash.zeroize();
        self.text.zeroize();
        self.new_sha.zeroize();
        self.tags = None;
        self.created_at = DateTime::<Utc>::from(std::time::UNIX_EPOCH);
        self.updated_at = DateTime::<Utc>::from(std::time::UNIX_EPOCH);
    }
}

impl ZeroizeOnDrop for Note {}
