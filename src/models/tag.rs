use ::rusqlite::Connection;
use chrono::{DateTime, Utc};
use egui::Color32;
use uuid::Uuid;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Debug, Clone)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    pub color: Color32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Tag {
    pub fn new(name: String, color: Color32) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            color,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    // ---- DB Functions
    pub fn insert(&self, conn: &Connection) {
        // NOTE: created_at & updated_at will be set as default by DB
        match conn.execute(
            "INSERT INTO tags (id, name, color) VALUES (?1, ?2, ?3);",
            (&self.id.to_string(), &self.name, &self.color.to_hex()),
        ) {
            Ok(_) => {}
            Err(e) => {
                println!("Error inserting tags to db: {e}");
            }
        }
    }

    pub fn update(&mut self, conn: &Connection) {
        match conn.execute(
            "UPDATE tags SET name = ?1, color = ?2 WHERE id = ?3;",
            (&self.name, &self.color.to_hex(), &self.id.to_string()),
        ) {
            Ok(_) => {
                self.updated_at = Utc::now();
            }
            Err(e) => {
                println!("Error updating tag to db: {e}");
            }
        }
    }

    pub fn update_name(&mut self, conn: &Connection) {
        match conn.execute(
            "UPDATE tags SET name = ?1 WHERE id = ?2;",
            (&self.name, &self.id.to_string()),
        ) {
            Ok(_) => {
                self.updated_at = Utc::now();
            }
            Err(e) => {
                println!("Error updating name in tags to db: {e}");
            }
        }
    }

    pub fn update_color(&mut self, conn: &Connection) {
        match conn.execute(
            "UPDATE tags SET color = ?1 WHERE id = ?2;",
            (&self.color.to_hex(), &self.id.to_string()),
        ) {
            Ok(_) => {
                self.updated_at = Utc::now();
            }
            Err(e) => {
                println!("Error updating color in tags to db: {e}");
            }
        }
    }

    pub fn delete(&mut self, conn: &Connection) {
        match conn.execute("DELETE FROM tags WHERE id = ?1;", (&self.id.to_string(),)) {
            Ok(_) => {
                self.zeroize();
            }
            Err(e) => {
                println!("Error deleting tags to db: {e}");
            }
        }
    }
}

impl Default for Tag {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: String::new(),
            color: Color32::LIGHT_GREEN,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl Zeroize for Tag {
    fn zeroize(&mut self) {
        self.id = Uuid::nil();
        self.name.zeroize();
        self.color = Color32::BLACK;
        self.created_at = DateTime::<Utc>::from(std::time::UNIX_EPOCH);
        self.updated_at = DateTime::<Utc>::from(std::time::UNIX_EPOCH);
    }
}

impl ZeroizeOnDrop for Tag {}
