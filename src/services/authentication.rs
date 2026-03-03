// native libraries
use std::{
    error::Error,
    fs::{File, create_dir_all},
    io::{Read, Write},
    path::PathBuf,
};

// external libraries
use egui_i18n::tr;
use zeroize::Zeroize;

// Internal modules usage
use crate::{
    MindSafeApp,
    constants::MIGRATIONS,
    models::config::Config,
    pages::Page,
    services::{
        database::DatabaseService,
        encryption::{EncryptedBlob, EncryptionService},
    },
};

pub struct AuthenticationService {}

impl AuthenticationService {
    fn get_storage_path() -> Result<PathBuf, Box<dyn Error>> {
        // Saving in same folder if debug mode is turned on
        if cfg!(debug_assertions) {
            let dir = PathBuf::from("./state.json");
            Ok(dir)
        } else {
            let mut dir = dirs::data_local_dir().ok_or("Could not find local data directory")?;
            dir.push("mindsafe");
            create_dir_all(&dir)?;
            dir.push("state.json");
            Ok(dir)
        }
    }

    fn save_registration_params(params: &EncryptedBlob) -> Result<(), Box<dyn Error>> {
        let path = Self::get_storage_path()?;
        let json = serde_json::to_string_pretty(params)?;
        let mut f = File::create(path)?;
        f.write_all(json.as_bytes())?;
        Ok(())
    }

    fn get_registration_params() -> Result<EncryptedBlob, Box<dyn Error>> {
        let path = Self::get_storage_path()?;
        let mut f = File::open(path)?;
        let mut buf = String::new();
        f.read_to_string(&mut buf)?;
        let info: EncryptedBlob = serde_json::from_str(&buf)?;
        Ok(info)
    }

    pub(crate) fn check_registered() -> bool {
        match AuthenticationService::get_registration_params() {
            Ok(_) => true,
            Err(e) => {
                println!("Error while getting the file : {e}");
                false
            }
        }
    }

    // Function to handle saving password
    fn validate_password(app: &mut MindSafeApp) -> bool {
        // Must be 8-60 chars long
        let len = app.password.chars().count();
        if len < 8 {
            app.errors.register = tr!("error-password-min");
            return false;
        } else if len > 60 {
            app.errors.register = tr!("error-password-max");
            return false;
        }

        let mut has_upper = false;
        let mut has_lower = false;
        let mut has_digit = false;
        let mut has_special = false;

        let specials = "@#$*._!&";

        for c in app.password.chars() {
            // contain atleast one upper alphabet
            if c.is_ascii_uppercase() {
                has_upper = true;
                // contain atleast one lowercase alphabet
            } else if c.is_ascii_lowercase() {
                has_lower = true;
                // contain atleast one numeric
            } else if c.is_ascii_digit() {
                has_digit = true;
                // contain atleast one of "@#$*._!&"
            } else if specials.contains(c) {
                has_special = true;
            }
        }

        if !has_upper {
            app.errors.register = tr!("error-password-upper");
        } else if !has_lower {
            app.errors.register = tr!("error-password-lower");
        } else if !has_digit {
            app.errors.register = tr!("error-password-numeric");
        } else if !has_special {
            app.errors.register = tr!("error-password-special");
        }

        has_upper && has_lower && has_digit && has_special
    }

    pub(crate) fn save_password(app: &mut MindSafeApp) {
        if !AuthenticationService::validate_password(app) {
            return;
        } else {
            app.errors.register = String::new();
        }

        // Random master key with full entropy
        let mut master_key: Vec<u8> = EncryptionService::generate_master_key();
        if cfg!(debug_assertions) {
            println!("Master Key Original (hex): {}", hex::encode(&master_key));
        }

        match EncryptionService::encrypt_with_password(&app.password, &master_key) {
            Ok(mut registration_params) => {
                app.password.zeroize(); // !!WARNING zeroizing password here as not needed further
                match AuthenticationService::save_registration_params(&registration_params) {
                    Ok(_) => {
                        registration_params.zeroize();
                    }
                    Err(e) => {
                        app.errors.register = format!("{e}");
                        return;
                    }
                }
            }
            Err(e) => {
                println!("Error when saving params: {e}");
                return;
            }
        }

        // Performing Auto-Login
        match AuthenticationService::get_registration_params() {
            Ok(mut loaded) => {
                // checking if blob saved correctly
                loaded.zeroize();

                let (db_key, file_key) = EncryptionService::generate_keys(&master_key);
                master_key.zeroize(); // !!WARNING zeroizing master_key here as not needed further

                app.db_key = db_key; // moved to main state no need to zeroize
                app.file_key = file_key; // moved to main state no need to zeroize

                // Initialize SQLCipher database service with derived created db_key
                match DatabaseService::new(&app.db_key) {
                    Ok(mut db_service) => {
                        // Run migrations
                        match db_service.init(MIGRATIONS) {
                            // Inserting default config
                            Ok(()) => {
                                // Saving current app config in db
                                app.config.insert(db_service.get_connection());
                                app.database_service = Some(db_service);
                                app.toasts.success("Signed up!");
                                app.current_page = Page::Editor
                            }
                            Err(_e) => {
                                println!("Error while running migrations db: {_e}");
                            }
                        }
                    }
                    Err(_e) => {
                        println!("Error while getting db while registering: {_e}");
                    }
                };
            }
            Err(e) => {
                println!("Error while fetching json: {e}");
                app.errors.register = "Coud not fetch saved".to_string();
            }
        }
    }

    // Function to handle verifying password
    pub(crate) fn verify_password(app: &mut MindSafeApp) {
        match AuthenticationService::get_registration_params() {
            Ok(saved_blob) => {
                // println!("loaded = {loaded:?}",);
                match EncryptionService::decrypt_with_password(&app.password, &saved_blob) {
                    Ok(mut master_key) => {
                        app.password.zeroize(); // !!WARNING zeroizing password here as not needed further

                        let (db_key, file_key) = EncryptionService::generate_keys(&master_key);
                        master_key.zeroize(); // !!WARNING zeroizing master_key here as not needed further

                        app.db_key = db_key; // moved to main state no need to zeroize
                        app.file_key = file_key; // moved to main state no need to zeroize

                        // Starting DB
                        match DatabaseService::new(&app.db_key) {
                            Ok(mut db_service) =>
                            // Run migrations
                            {
                                match db_service.init(MIGRATIONS) {
                                    // Inserting default config
                                    Ok(()) => match Config::get(db_service.get_connection()) {
                                        Ok(config) => {
                                            // Getting saved config from db
                                            app.config = config;
                                            app.database_service = Some(db_service);
                                            app.toasts.success("Logged in!");
                                            app.get_all_notes();
                                            app.current_page = Page::Editor;
                                        }
                                        Err(e) => {
                                            println!("Error fetching config: {e}");

                                            // Adding new config
                                            let config = Config::default();
                                            config.insert(db_service.get_connection());
                                            app.database_service = Some(db_service);
                                            app.toasts.success("Logged in!");
                                            app.get_all_notes();
                                            app.current_page = Page::Editor;
                                        }
                                    },
                                    Err(_e) => {
                                        println!("Error while running migrations db: {_e}");
                                    }
                                }
                            }
                            Err(_e) => {
                                println!("Error while getting db while loggin: {_e}");
                            }
                        };
                    }
                    Err(e) => {
                        app.errors.login = "Incorrect Password!".to_string();
                        if cfg!(debug_assertions) {
                            println!("Error while decrypting: {e}");
                        }
                    }
                }
            }
            Err(e) => {
                if cfg!(debug_assertions) {
                    println!("Error while fetching json: {e}");
                }
                app.errors.login = "Coud not fetch saved".to_string();
            }
        }
    }
}
