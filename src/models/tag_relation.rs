use chrono::{DateTime, Utc};
use rusqlite::Connection;
use uuid::Uuid;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Debug, Clone)]
pub struct TagRelation {
    pub id: Uuid,
    pub note_id: Uuid,
    pub tag_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl TagRelation {
    pub fn new(note_id: Uuid, tag_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            note_id,
            tag_id,
            created_at: Utc::now(),
        }
    }

    // ---- DB Functions
    pub fn insert(&self, conn: &Connection) {
        // NOTE: created_at & updated_at will be set as default by DB
        match conn.execute(
            "INSERT INTO tag_relations (id, note_id, tag_id) VALUES (?1, ?2, ?3);",
            (
                &self.id.to_string(),
                &self.note_id.to_string(),
                &self.tag_id.to_string(),
            ),
        ) {
            Ok(_) => {}
            Err(e) => {
                println!("Error inserting tag_relations to db: {e}");
            }
        }
    }

    pub fn delete(&mut self, conn: &Connection) {
        match conn.execute(
            "DELETE FROM tag_relations WHERE id = ?1;",
            (&self.id.to_string(),),
        ) {
            Ok(_) => {
                self.zeroize();
            }
            Err(e) => {
                println!("Error deleting tag_relations to db: {e}");
            }
        }
    }
}

impl Default for TagRelation {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            note_id: Uuid::new_v4(),
            tag_id: Uuid::new_v4(),
            created_at: Utc::now(),
        }
    }
}

impl Zeroize for TagRelation {
    fn zeroize(&mut self) {
        self.id = Uuid::nil();
        self.note_id = Uuid::nil();
        self.tag_id = Uuid::nil();
        self.created_at = DateTime::<Utc>::from(std::time::UNIX_EPOCH);
    }
}

impl ZeroizeOnDrop for TagRelation {}
