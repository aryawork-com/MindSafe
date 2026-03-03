use ::rusqlite::Connection;
use ::zeroize::{Zeroize, ZeroizeOnDrop};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Clone, Copy, Debug)]
pub enum SystemActivityType {
    Authentication = 1,
    Configuration = 2,
    Notes = 3,
}

impl SystemActivityType {
    pub fn value(&self) -> i32 {
        match self {
            SystemActivityType::Authentication => 1,
            SystemActivityType::Configuration => 2,
            SystemActivityType::Notes => 3,
        }
    }
}

impl Zeroize for SystemActivityType {
    fn zeroize(&mut self) {}
}

impl ZeroizeOnDrop for SystemActivityType {}

#[derive(Clone, Debug)]
pub struct Activities {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub activity_type: SystemActivityType,
    /// in days
    pub retain_duration: i32,
    pub enabled: bool,
    pub requires_review: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Zeroize for Activities {
    fn zeroize(&mut self) {
        self.id = Uuid::nil();
        self.name.zeroize();
        self.description.zeroize();
        self.retain_duration.zeroize();
        self.enabled.zeroize();
        self.activity_type.zeroize();
        self.requires_review.zeroize();
        self.created_at = DateTime::<Utc>::from(std::time::UNIX_EPOCH);
        self.updated_at = DateTime::<Utc>::from(std::time::UNIX_EPOCH);
    }
}

impl ZeroizeOnDrop for Activities {}

impl Activities {
    pub fn new(
        name: String,
        description: String,
        activity_type: SystemActivityType,
        retain_duration: i32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            activity_type,
            retain_duration,
            enabled: true,
            requires_review: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    // ---- DB Functions
    pub fn insert(&self, conn: &Connection) {
        // NOTE: created_at & updated_at will be set as default by DB
        match conn.execute(
            "INSERT INTO activities (id, name, description, activity_type, retain_duration, enabled, requires_review) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7);",
            (&self.id.to_string(), &self.name, &self.description, &self.activity_type.value(), &self.retain_duration, &self.enabled, &self.requires_review),
        ) {
            Ok(_) => {}
            Err(e) => {
                println!("Error inserting activity to db: {e}");
            }
        }
    }

    pub fn update(&mut self, conn: &Connection) {
        match conn.execute(
            "UPDATE activities SET name = ?1, description = ?2, activity_type = ?3, retain_duration = ?4, enabled = ?5,
requires_review = ?6 WHERE id = ?7;",
            (&self.name, &self.description, &self.activity_type.value(), &self.retain_duration, &self.enabled, &self.requires_review, &self.id.to_string(),),
        ) {
            Ok(_) => {
                self.updated_at = Utc::now();
            }
            Err(e) => {
                println!("Error updating activities to db: {e}");
            }
        }
    }

    pub fn update_enabled(&mut self, conn: &Connection) {
        match conn.execute(
            "UPDATE activities SET enabled = ?1 WHERE id = ?2;",
            (&self.enabled, &self.id.to_string()),
        ) {
            Ok(_) => {
                self.updated_at = Utc::now();
            }
            Err(e) => {
                println!("Error updating enabled in activities to db: {e}");
            }
        }
    }

    pub fn update_requires_review(&mut self, conn: &Connection) {
        match conn.execute(
            "UPDATE activities SET requires_review = ?1 WHERE id = ?2;",
            (&self.requires_review, &self.id.to_string()),
        ) {
            Ok(_) => {
                self.updated_at = Utc::now();
            }
            Err(e) => {
                println!("Error updating requires_review in activities to db: {e}");
            }
        }
    }
}
