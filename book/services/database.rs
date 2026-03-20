use ::std::{fs, path::Path};
use std::{error::Error, fmt, fs::create_dir_all, path::PathBuf};

use hex::encode;
use rusqlite::{Connection, Result, params};

#[derive(Debug, Clone)]
pub struct Migration {
    pub serial_id: i16,
    pub name: &'static str,
    pub query: &'static str,
    pub version: &'static str,
}

#[derive(Debug)]
pub struct DatabaseMigration {
    pub id: i64,
    pub serial_id: i16,
    pub name: String,
    pub query: Option<String>,
    pub version: Option<String>,
    pub created_at: String,
    pub updated_on: String,
    pub completed_on: Option<String>,
    pub failed_on: Option<String>,
    pub migration_status: i16,
}

#[derive(Debug)]
pub enum MigrationError {
    DatabaseError(rusqlite::Error),
    MigrationFailed(String),
    IoError(String),
}

impl fmt::Display for MigrationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MigrationError::DatabaseError(err) => write!(f, "Database error: {err}"),
            MigrationError::MigrationFailed(msg) => write!(f, "Migration failed: {msg}",),
            MigrationError::IoError(msg) => write!(f, "Io Operation failed: {msg}",),
        }
    }
}

impl Error for MigrationError {}

impl From<rusqlite::Error> for MigrationError {
    fn from(err: rusqlite::Error) -> Self {
        MigrationError::DatabaseError(err)
    }
}

pub struct DatabaseService {
    conn: Connection,
}

impl DatabaseService {
    fn get_db_path(db_name: String) -> Result<PathBuf, MigrationError> {
        // Saving in same folder if debug mode is turned on
        if cfg!(debug_assertions) {
            let dir = PathBuf::from(format!("./{}.db", db_name));
            Ok(dir)
        } else {
            match dirs::data_local_dir() {
                Some(mut dir) => {
                    dir.push("mindsafe");
                    match create_dir_all(&dir) {
                        Ok(_) => {
                            return Err(MigrationError::DatabaseError(
                                rusqlite::Error::InvalidPath(PathBuf::from(format!(
                                    "./{}.db",
                                    db_name
                                ))),
                            ));
                        }
                        Err(e) => {
                            if cfg!(debug_assertions) {
                                println!("Error while creating local dirs: {e}");
                            }
                        }
                    }
                    dir.push(format!("{}.db", db_name));
                    Ok(dir)
                }
                None => {
                    if cfg!(debug_assertions) {
                        println!("Error while fetching local dir, path not found");
                    }
                    Err(MigrationError::DatabaseError(rusqlite::Error::InvalidPath(
                        PathBuf::from("./mindsafe.db"),
                    )))
                }
            }
        }
    }

    /// Initialize a new DatabaseService with in-memory database
    pub fn new(db_key: &[u8], db_name: String) -> Result<Self, MigrationError> {
        let path = DatabaseService::get_db_path(db_name)?;
        if cfg!(debug_assertions) {
            println!("Database Path: {path:?}");
        }

        let conn: Connection = Connection::open(path)?;

        if cfg!(debug_assertions) {
            // Use this for DEBUG MODE
            // let version: String = conn.query_row("PRAGMA cipher_version;", [], |row| row.get(0))?;
            // println!("Cipher version: {}", version);

            // // conn.execute("PRAGMA key = 'supersafepassword';", [])?;
            // let _: String =
            //     conn.query_row("PRAGMA key = 'supersafepassword';", [], |row| row.get(0))?;

            // conn.execute("PRAGMA synchronous = FULL;", [])?;
            // conn.execute("PRAGMA cache_size = -6000;", [])?;
            // conn.execute("PRAGMA journal_mode = WAL;", [])?;

            // NO KEY AT ALL
        } else {
            let key_hex = encode(db_key);
            let pragma_value = format!("x'{key_hex}'");
            conn.pragma_update(None, "key", pragma_value)?;

            conn.pragma_update(None, "journal_mode", "WAL")?;
            conn.pragma_update(None, "synchronous", "FULL")?;
            conn.pragma_update(None, "cache_size", -6000)?;
        }

        Ok(DatabaseService { conn })
    }

    /// Delete file
    pub fn delete_db(db_name: String) -> Result<(), MigrationError> {
        let path = DatabaseService::get_db_path(db_name)?;

        if cfg!(debug_assertions) {
            println!("Database Path: {path:?}");
        }

        if Path::new(&path).exists() {
            fs::remove_file(&path).map_err(|e| {
                if cfg!(debug_assertions) {
                    println!("Database could not be deleted due to: {path:?}: {e}");
                };
                MigrationError::IoError(e.to_string())
            })?;
        }

        Ok(())
    }
    /// Initialize database and run migrations
    pub fn init(&mut self, migrations: &[Migration]) -> Result<(), MigrationError> {
        self.create_migrations_table()?;
        self.run_migrations(migrations)?;
        Ok(())
    }

    /// Create the migrations table if it doesn't exist
    fn create_migrations_table(&self) -> Result<(), MigrationError> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS migrations (
                id INTEGER PRIMARY KEY,
                serial_id INTEGER NOT NULL UNIQUE,
                name TEXT NOT NULL,
                query TEXT DEFAULT NULL,
                version TEXT DEFAULT NULL,
                created_at TIMESTAMP DEFAULT (DATETIME('now', 'localtime')),
                updated_on TIMESTAMP DEFAULT (DATETIME('now', 'localtime')),
                completed_on TIMESTAMP DEFAULT NULL,
                failed_on TIMESTAMP DEFAULT NULL,
                migration_status SMALLINT DEFAULT 0
            )",
            [],
        )?;
        Ok(())
    }

    /// Get migration by serial_id
    fn get_migration_by_serial_id(
        &self,
        serial_id: i16,
    ) -> Result<Option<DatabaseMigration>, MigrationError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, serial_id, name, query, version, created_at, updated_on, 
             completed_on, failed_on, migration_status 
             FROM migrations WHERE serial_id = ?",
        )?;

        let mut migration_iter = stmt.query_map([serial_id], |row| {
            Ok(DatabaseMigration {
                id: row.get(0)?,
                serial_id: row.get(1)?,
                name: row.get(2)?,
                query: row.get(3)?,
                version: row.get(4)?,
                created_at: row.get(5)?,
                updated_on: row.get(6)?,
                completed_on: row.get(7)?,
                failed_on: row.get(8)?,
                migration_status: row.get(9)?,
            })
        })?;

        if let Some(migration) = migration_iter.next() {
            Ok(Some(migration?))
        } else {
            Ok(None)
        }
    }

    /// Record successful migration
    fn record_successful_migration(&self, migration: &Migration) -> Result<(), MigrationError> {
        let existing = self.get_migration_by_serial_id(migration.serial_id)?;

        if let Some(_existing_migration) = existing {
            // Update existing migration to successful
            self.conn.execute(
                "UPDATE migrations SET 
                 migration_status = 1, 
                 completed_on = DATETIME('now', 'localtime'),
                 failed_on = NULL,
                 updated_on = DATETIME('now', 'localtime')
                 WHERE serial_id = ?",
                params![migration.serial_id],
            )?;
        } else {
            // Insert new successful migration
            self.conn.execute(
                "INSERT INTO migrations (serial_id, name, query, version, migration_status, completed_on)
                 VALUES (?, ?, ?, ?, 1, DATETIME('now', 'localtime'))",
                params![
                    migration.serial_id,
                    migration.name,
                    migration.query,
                    migration.version
                ],
            )?;
        }
        Ok(())
    }

    /// Record failed migration
    fn record_failed_migration(&self, migration: &Migration) -> Result<(), MigrationError> {
        let existing = self.get_migration_by_serial_id(migration.serial_id)?;

        if let Some(_existing_migration) = existing {
            // Update existing migration to failed
            self.conn.execute(
                "UPDATE migrations SET 
                 migration_status = 2, 
                 failed_on = DATETIME('now', 'localtime'),
                 completed_on = NULL,
                 updated_on = DATETIME('now', 'localtime')
                 WHERE serial_id = ?",
                params![migration.serial_id],
            )?;
        } else {
            // Insert new failed migration
            self.conn.execute(
                "INSERT INTO migrations (serial_id, name, query, version, migration_status, failed_on)
                 VALUES (?, ?, ?, ?, 2, DATETIME('now', 'localtime'))",
                params![
                    migration.serial_id,
                    migration.name,
                    migration.query,
                    migration.version
                ],
            )?;
        }
        Ok(())
    }

    /// Run all migrations that haven't been successfully completed
    pub fn run_migrations(&mut self, migrations: &[Migration]) -> Result<(), MigrationError> {
        for migration in migrations {
            let existing = self.get_migration_by_serial_id(migration.serial_id)?;

            // Skip if migration is already successful
            if let Some(ref existing_migration) = existing
                && existing_migration.migration_status == 1
            {
                if cfg!(debug_assertions) {
                    println!(
                        "Migration with Name: {}, Serial Id: {}, already exists, skipping!",
                        existing_migration.name, existing_migration.serial_id
                    );
                }
                continue;
            }

            // Execute migration
            match self.conn.execute(migration.query, []) {
                Ok(_) => {
                    self.record_successful_migration(migration)?;
                    if cfg!(debug_assertions) {
                        println!(
                            "Migration with Name: {}, Serial Id: {}, completed successfully! Existing: {:?}",
                            migration.name,
                            migration.serial_id,
                            existing.is_some()
                        );
                    }
                }
                Err(err) => {
                    if cfg!(debug_assertions) {
                        eprintln!(
                            "Migration failed: {} - {} | Error: {}",
                            migration.serial_id, migration.name, err
                        );
                    }
                    self.record_failed_migration(migration)?;
                }
            }
        }
        Ok(())
    }

    /// Get all migrations from the database
    pub fn get_all_migrations(&self) -> Result<Vec<DatabaseMigration>, MigrationError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, serial_id, name, query, version, created_at, updated_on, 
             completed_on, failed_on, migration_status 
             FROM migrations ORDER BY serial_id",
        )?;

        let migration_iter = stmt.query_map([], |row| {
            Ok(DatabaseMigration {
                id: row.get(0)?,
                serial_id: row.get(1)?,
                name: row.get(2)?,
                query: row.get(3)?,
                version: row.get(4)?,
                created_at: row.get(5)?,
                updated_on: row.get(6)?,
                completed_on: row.get(7)?,
                failed_on: row.get(8)?,
                migration_status: row.get(9)?,
            })
        })?;

        let mut migrations = Vec::new();
        for migration in migration_iter {
            migrations.push(migration?);
        }
        Ok(migrations)
    }

    /// Get the underlying connection for executing queries
    pub fn get_connection(&self) -> &Connection {
        &self.conn
    }
}
