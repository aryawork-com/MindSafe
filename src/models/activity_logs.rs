use ::rusqlite::Connection;
use ::uuid::Uuid;
use ::zeroize::{Zeroize, ZeroizeOnDrop};
use chrono::{DateTime, Utc};

#[derive(Clone, Debug)]
pub struct ActivityLog {
    pub id: Uuid,
    pub activity_id: Uuid,
    pub session_id: Uuid,
    pub item_id: Option<Uuid>,
    pub details: Option<String>,
    pub reviewed: bool,
    pub starred: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Zeroize for ActivityLog {
    fn zeroize(&mut self) {
        self.id = Uuid::nil();
        self.activity_id = Uuid::nil();
        self.session_id = Uuid::nil();
        self.item_id = None;
        self.details.zeroize();
        self.reviewed.zeroize();
        self.starred.zeroize();
        self.created_at = DateTime::<Utc>::from(std::time::UNIX_EPOCH);
        self.updated_at = DateTime::<Utc>::from(std::time::UNIX_EPOCH);
    }
}

impl ZeroizeOnDrop for ActivityLog {}

impl ActivityLog {
    pub fn new(
        activity_id: Uuid,
        session_id: Uuid,
        item_id: Option<Uuid>,
        details: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            activity_id,
            session_id,
            item_id,
            details,
            reviewed: false,
            starred: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    // ---- DB Functions
    pub fn insert(&self, conn: &Connection) {
        // NOTE: created_at & updated_at will be set as default by DB
        if let Some(item_id) = self.item_id {
            match conn.execute(
                "INSERT INTO activity_logs (id, activity_id, session_id, item_id, details, reviewed, starred) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7);",
                (&self.id.to_string(), &self.activity_id.to_string(), item_id.to_string(), &self.session_id.to_string(), &self.details, &self.reviewed, &self.starred),
            ) {
                Ok(_) => {}
                Err(e) => {
                    println!("Error inserting activity log to db: {e}");
                }
            }
        } else {
            match conn.execute(
                "INSERT INTO activity_logs (id, activity_id, session_id, details, reviewed, starred) VALUES (?1, ?2, ?3, ?4, ?5, ?6);",
                (&self.id.to_string(), &self.activity_id.to_string(), &self.session_id.to_string(), &self.details, &self.reviewed, &self.starred),
            ) {
                Ok(_) => {}
                Err(e) => {
                    println!("Error inserting activity log to db: {e}");
                }
            }
        }
    }

    pub fn update_reviewed(&mut self, conn: &Connection) {
        match conn.execute(
            "UPDATE activity_logs SET reviewed = ?1 WHERE id = ?2;",
            (&self.reviewed, &self.id.to_string()),
        ) {
            Ok(_) => {
                self.updated_at = Utc::now();
            }
            Err(e) => {
                println!("Error updating reviewed in activity_logs to db: {e}");
            }
        }
    }

    pub fn update_starred(&mut self, conn: &Connection) {
        match conn.execute(
            "UPDATE activity_logs SET starred = ?1 WHERE id = ?2;",
            (&self.starred, &self.id.to_string()),
        ) {
            Ok(_) => {
                self.updated_at = Utc::now();
            }
            Err(e) => {
                println!("Error updating starred in activity_logs to db: {e}");
            }
        }
    }
    pub fn update_details(&mut self, conn: &Connection) {
        match conn.execute(
            "UPDATE activity_logs SET details = ?1 WHERE id = ?2;",
            (&self.details, &self.id.to_string()),
        ) {
            Ok(_) => {
                self.updated_at = Utc::now();
            }
            Err(e) => {
                println!("Error updating starred in activity_logs to db: {e}");
            }
        }
    }
}
