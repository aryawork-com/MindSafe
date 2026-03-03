use std::path::PathBuf;

use chrono::{DateTime, NaiveDateTime, Utc};
use egui_material_icons::icons::{ICON_ARROW_COOL_DOWN, ICON_ARROW_WARM_UP};
use rusqlite::{Connection, Error};
use uuid::Uuid;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::i18n::Language;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum SortingScheme {
    CreatedAtAscending = 1,
    CreatedAtDescending = 2,
    UpdatedAtAscending = 3,
    UpdatedAtDescending = 4,
    TitleAscending = 5,
    TitleDescending = 6,
}

impl SortingScheme {
    pub fn get_int(&self) -> i16 {
        match self {
            Self::CreatedAtAscending => 1,
            Self::CreatedAtDescending => 2,
            Self::UpdatedAtAscending => 3,
            Self::UpdatedAtDescending => 4,
            Self::TitleAscending => 5,
            Self::TitleDescending => 6,
        }
    }
    pub fn name(&self) -> String {
        match self {
            Self::CreatedAtAscending => format!("Created ASC {ICON_ARROW_WARM_UP}"),
            Self::CreatedAtDescending => format!("Created DESC {ICON_ARROW_COOL_DOWN}"),
            Self::UpdatedAtAscending => format!("Updated ASC {ICON_ARROW_WARM_UP}"),
            Self::UpdatedAtDescending => format!("Updated DESC {ICON_ARROW_COOL_DOWN}"),
            Self::TitleAscending => format!("Title ASC {ICON_ARROW_WARM_UP}"),
            Self::TitleDescending => format!("Title DESC {ICON_ARROW_COOL_DOWN}"),
        }
    }
    pub fn get_scheme(value: i16) -> Self {
        match value {
            1 => Self::CreatedAtAscending,
            2 => Self::CreatedAtDescending,
            3 => Self::UpdatedAtAscending,
            4 => Self::UpdatedAtDescending,
            5 => Self::TitleAscending,
            6 => Self::TitleDescending,
            _ => Self::UpdatedAtDescending,
        }
    }

    pub const ALL_SCHEMES: [SortingScheme; 6] = [
        SortingScheme::CreatedAtAscending,
        SortingScheme::CreatedAtDescending,
        SortingScheme::UpdatedAtAscending,
        SortingScheme::UpdatedAtDescending,
        SortingScheme::TitleAscending,
        SortingScheme::TitleDescending,
    ];

    pub fn order_by(&self) -> &'static str {
        match self {
            Self::CreatedAtAscending => "ORDER BY created_at ASC",
            Self::CreatedAtDescending => "ORDER BY created_at DESC",
            Self::UpdatedAtAscending => "ORDER BY updated_at ASC",
            Self::UpdatedAtDescending => "ORDER BY updated_at DESC",
            Self::TitleAscending => "ORDER BY title ASC",
            Self::TitleDescending => "ORDER BY title DESC",
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for SortingScheme {
    fn default() -> Self {
        Self::UpdatedAtDescending
    }
}

/// auto_save_duration and auto_lock_duration are in minutes
#[derive(Debug, Clone)]
pub struct Config {
    id: i32,
    pub selected_language: Language,
    main_directory: PathBuf,
    backup_directory: PathBuf,
    pub safe_copy: bool,
    pub syntax_highlight: bool,
    // Minutes
    pub auto_save_duration: i64,
    // Minutes
    pub auto_lock_duration: i64,
    pub sorting: SortingScheme,
    last_opened_note: Option<Uuid>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Config {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        selected_language: Language,
        main_directory: PathBuf,
        backup_directory: PathBuf,
        safe_copy: bool,
        syntax_highlight: bool,
        auto_save_duration: i64,
        auto_lock_duration: i64,
        sorting: SortingScheme,
    ) -> Self {
        Self {
            id: 1,
            selected_language,
            main_directory,
            backup_directory,
            safe_copy,
            syntax_highlight,
            sorting,
            auto_save_duration,
            auto_lock_duration,
            last_opened_note: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    // ---- Safe Getter Functions
    pub fn main_directory(&self) -> PathBuf {
        self.main_directory.clone()
    }
    pub fn backup_directory(&self) -> PathBuf {
        self.backup_directory.clone()
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    // ---- Utility Getter Functions
    pub fn language_code(&self) -> &str {
        self.selected_language.get_code()
    }
    pub fn language_name(&self) -> &str {
        self.selected_language.get_name()
    }
    pub fn main_directory_string(&self) -> String {
        self.main_directory.to_string_lossy().to_string()
    }
    pub fn backup_directory_string(&self) -> String {
        self.backup_directory.to_string_lossy().to_string()
    }

    // ---- Safe Setter Functions
    pub fn set_main_directory(&mut self, new_directory: PathBuf) {
        self.main_directory = new_directory;
    }

    pub fn set_backup_directory(&mut self, new_directory: PathBuf) {
        self.backup_directory = new_directory;
    }

    // NOTE: Skipping updated_at as DB will auto implement that on each update, created_at cannot be updated at all!!

    // ---- DB Functions
    pub fn insert(&self, conn: &Connection) {
        // NOTE: created_at & updated_at will be set as default by DB
        match conn.execute(
            "INSERT INTO config (id, selected_language, main_directory, backup_directory, safe_copy, syntax_highlight, auto_save_duration, auto_lock_duration, sorting) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9);",
            (1, &self.language_code(), &self.main_directory_string(),&self.backup_directory_string(), &self.safe_copy, &self.syntax_highlight, &self.auto_save_duration, &self.auto_lock_duration, &self.sorting.get_int()),
        ) {
            Ok(_) => {},
            Err(e) => {
                println!("Error inserting config to db: {e}");
            }
        }
    }
    pub fn get(conn: &Connection) -> Result<Config, Error> {
        conn.query_row(
            "SELECT id, selected_language, main_directory, backup_directory, safe_copy,
                    syntax_highlight, auto_save_duration, auto_lock_duration,
                    last_opened_note, sorting, created_at, updated_at
             FROM config
             WHERE id = ?1;",
            (1,),
            |row| {
                // Parse created_at
                let created_at_str: String = row.get(10)?;
                let created_at = DateTime::<Utc>::from_naive_utc_and_offset(
                    NaiveDateTime::parse_from_str(&created_at_str, "%Y-%m-%d %H:%M:%S").map_err(
                        |e| {
                            rusqlite::Error::FromSqlConversionFailure(
                                10,
                                rusqlite::types::Type::Text,
                                Box::new(e),
                            )
                        },
                    )?,
                    Utc,
                );

                // Parse updated_at
                let updated_at_str: String = row.get(11)?;
                let updated_at = DateTime::<Utc>::from_naive_utc_and_offset(
                    NaiveDateTime::parse_from_str(&updated_at_str, "%Y-%m-%d %H:%M:%S").map_err(
                        |e| {
                            rusqlite::Error::FromSqlConversionFailure(
                                11,
                                rusqlite::types::Type::Text,
                                Box::new(e),
                            )
                        },
                    )?,
                    Utc,
                );

                Ok(Config {
                    id: row.get(0)?,
                    selected_language: Language::parse_code(&row.get::<_, String>(1)?), // adjust parsing as needed
                    main_directory: PathBuf::from(row.get::<_, String>(2)?),
                    backup_directory: PathBuf::from(row.get::<_, String>(3)?),
                    safe_copy: row.get(4)?,
                    syntax_highlight: row.get(5)?,
                    auto_save_duration: row.get(6)?,
                    auto_lock_duration: row.get(7)?,
                    last_opened_note: row
                        .get::<_, Option<String>>(8)?
                        .and_then(|s| Uuid::parse_str(&s).ok()),
                    sorting: SortingScheme::get_scheme(row.get(9)?),
                    created_at,
                    updated_at,
                })
            },
        )
    }

    // NOTE: There will only be one config that is why hard coding id = 1
    pub fn update_selected_language(&mut self, conn: &Connection) {
        match conn.execute(
            "UPDATE config SET selected_language = ?1 WHERE id = 1;",
            (&self.language_code(),),
        ) {
            Ok(_) => {
                self.updated_at = Utc::now();
            }
            Err(e) => {
                println!("Error updating selected_language in config to db: {e}");
            }
        }
    }
    pub fn update_main_directory(&mut self, conn: &Connection) {
        match conn.execute(
            "UPDATE config SET main_directory = ?1 WHERE id = 1;",
            (&self.main_directory_string(),),
        ) {
            Ok(_) => {
                self.updated_at = Utc::now();
            }
            Err(e) => {
                println!("Error updating main_directory in config to db: {e}");
            }
        }
    }

    pub fn update_backup_directory(&mut self, conn: &Connection) {
        match conn.execute(
            "UPDATE config SET backup_directory = ?1 WHERE id = 1;",
            (&self.backup_directory_string(),),
        ) {
            Ok(_) => {
                self.updated_at = Utc::now();
            }
            Err(e) => {
                println!("Error updating backup_directory in config to db: {e}");
            }
        }
    }

    pub fn update_last_opened_note(&mut self, conn: &Connection) {
        if let Some(last_opened_note) = self.last_opened_note {
            match conn.execute(
                "UPDATE config SET last_opened_note = ?1 WHERE id = 1;",
                (last_opened_note.to_string(),),
            ) {
                Ok(_) => {
                    self.updated_at = Utc::now();
                }
                Err(e) => {
                    println!("Error updating last_opened_note in config to db: {e}");
                }
            };
        } else {
            match conn.execute(
                "UPDATE config SET last_opened_note = ?1 WHERE id = 1;",
                ("".to_string(),),
            ) {
                Ok(_) => {
                    self.updated_at = Utc::now();
                }
                Err(e) => {
                    println!("Error updating last_opened_note in config to db: {e}");
                }
            };
        }
    }

    pub fn update_auto_save_duration(&mut self, conn: &Connection) {
        match conn.execute(
            "UPDATE config SET auto_save_duration = ?1 WHERE id = 1;",
            (&self.auto_save_duration,),
        ) {
            Ok(_) => {
                self.updated_at = Utc::now();
            }
            Err(e) => {
                println!("Error updating auto_save_duration in config to db: {e}");
            }
        }
    }

    pub fn update_auto_lock_duration(&mut self, conn: &Connection) {
        match conn.execute(
            "UPDATE config SET auto_lock_duration = ?1 WHERE id = 1;",
            (&self.auto_lock_duration,),
        ) {
            Ok(_) => {
                self.updated_at = Utc::now();
            }
            Err(e) => {
                println!("Error updating auto_lock_duration in config to db: {e}");
            }
        }
    }

    pub fn update_safe_copy(&mut self, conn: &Connection) {
        match conn.execute(
            "UPDATE config SET safe_copy = ?1 WHERE id = 1;",
            (&self.safe_copy,),
        ) {
            Ok(_) => {
                self.updated_at = Utc::now();
            }
            Err(e) => {
                println!("Error updating safe_copy in config to db: {e}");
            }
        }
    }

    pub fn update_sorting(&mut self, conn: &Connection) {
        match conn.execute(
            "UPDATE config SET sorting = ?1 WHERE id = 1;",
            (&self.sorting.get_int(),),
        ) {
            Ok(_) => {
                self.updated_at = Utc::now();
            }
            Err(e) => {
                println!("Error updating sorting in config to db: {e}");
            }
        }
    }

    pub fn update_syntax_highlight(&mut self, conn: &Connection) {
        match conn.execute(
            "UPDATE config SET syntax_highlight = ?1 WHERE id = 1;",
            (&self.syntax_highlight,),
        ) {
            Ok(_) => {
                self.updated_at = Utc::now();
            }
            Err(e) => {
                println!("Error updating syntax_highlight in config to db: {e}");
            }
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            id: 1,
            selected_language: Language::English,
            main_directory: PathBuf::new(),
            backup_directory: PathBuf::new(),
            safe_copy: true,
            syntax_highlight: true,
            sorting: SortingScheme::default(),
            auto_save_duration: 5,
            auto_lock_duration: 30,
            last_opened_note: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl Zeroize for Config {
    fn zeroize(&mut self) {
        self.id.zeroize();
        self.selected_language = Language::English;
        self.main_directory = PathBuf::new();
        self.backup_directory = PathBuf::new();
        self.safe_copy.zeroize();
        self.syntax_highlight.zeroize();
        self.auto_save_duration.zeroize();
        self.auto_lock_duration.zeroize();
        self.last_opened_note = None;
        self.sorting = SortingScheme::default();
        self.created_at = DateTime::<Utc>::from(std::time::UNIX_EPOCH);
        self.updated_at = DateTime::<Utc>::from(std::time::UNIX_EPOCH);
    }
}

impl ZeroizeOnDrop for Config {}
