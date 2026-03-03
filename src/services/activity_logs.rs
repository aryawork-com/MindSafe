use ::rusqlite::Connection;
use ::uuid::Uuid;

use crate::models::{activities::Activities, activity_logs::ActivityLog};

pub enum ActivityRecordType {
    // Authentication
    Register = 1,
    Login = 2,
    Logout = 3,
    PasswordReset = 4,
    MasterKeyReset = 5,
    // Configuration
    ConfigUpdate = 6,
    // Notes
    NoteCreated = 7,
    NoteAccessed = 8, // Read
    NoteDeleted = 9,
    // NOTE: No entry for updated as they are already recorded in notes history
}

impl ActivityRecordType {
    pub fn get_activity_id(&self) -> Uuid {
        match self {
            ActivityRecordType::Register => Uuid::new_v4(),
            ActivityRecordType::Login => Uuid::new_v4(),
            ActivityRecordType::Logout => Uuid::new_v4(),
            ActivityRecordType::PasswordReset => Uuid::new_v4(),
            ActivityRecordType::MasterKeyReset => Uuid::new_v4(),
            ActivityRecordType::ConfigUpdate => Uuid::new_v4(),
            ActivityRecordType::NoteCreated => Uuid::new_v4(),
            ActivityRecordType::NoteAccessed => Uuid::new_v4(),
            ActivityRecordType::NoteDeleted => Uuid::new_v4(),
        }
    }
    pub fn get_activity_details(&self, note_name: Option<String>) -> Option<String> {
        match self {
            ActivityRecordType::Register => None,
            ActivityRecordType::Login => None,
            ActivityRecordType::Logout => None,
            ActivityRecordType::PasswordReset => None,
            ActivityRecordType::MasterKeyReset => None,
            ActivityRecordType::ConfigUpdate => Some(String::new()),
            ActivityRecordType::NoteCreated
            | ActivityRecordType::NoteAccessed
            | ActivityRecordType::NoteDeleted => note_name.map(|name| format!("name: {name}")),
        }
    }
}

pub struct ActivityLogService {
    pub activities: Vec<Activities>,
}

impl ActivityLogService {
    pub fn get_all_activities() {}

    /// Save returining log in case of logic for login activity
    pub fn record_activity(
        session_id: &Uuid,
        item_id: Option<&Uuid>, // usually note id
        activity_type: ActivityRecordType,
        conn: &Connection,
        note_name: Option<String>,
    ) -> ActivityLog {
        if let Some(item_id) = item_id {
            let activity_log = ActivityLog::new(
                activity_type.get_activity_id(),
                *session_id,
                Some(*item_id),
                activity_type.get_activity_details(note_name),
            );
            activity_log.insert(conn);
            activity_log
        } else {
            let activity_log = ActivityLog::new(
                activity_type.get_activity_id(),
                *session_id,
                None,
                activity_type.get_activity_details(note_name),
            );
            activity_log.insert(conn);
            activity_log
        }
    }
}
