// native libraries
use std::{
    error::Error,
    fs::{File, create_dir_all},
    io::{Read, Write},
    path::PathBuf,
};

// external libraries
use ::uuid::Uuid;
use egui::Ui;
use egui_i18n::tr;
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

// Internal modules usage
use crate::{
    MindSafeApp,
    constants::MIGRATIONS,
    models::{activities::Activity, config::Config, note::Note},
    pages::Page,
    services::{
        database::DatabaseService,
        encryption::{EncryptedBlob, EncryptionService},
        notes::NoteService,
        tabs::Tab,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkSpace {
    pub id: String,
    pub workspace_name: String,
    pub params: EncryptedBlob,
}

impl Zeroize for WorkSpace {
    fn zeroize(&mut self) {
        self.id.zeroize();
        self.workspace_name.zeroize();
        self.params.zeroize();
    }
}

impl ZeroizeOnDrop for WorkSpace {}

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

    fn create_registration_params(mut workspaces: Vec<WorkSpace>) -> Result<(), Box<dyn Error>> {
        let path = Self::get_storage_path()?;
        let json = serde_json::to_string_pretty(&workspaces)?;
        let mut f = File::create(path)?;
        f.write_all(json.as_bytes())?;
        workspaces.zeroize();
        Ok(())
    }

    fn save_registration_params(
        workspace_params: &WorkSpace,
        create_workspace: bool,
    ) -> Result<(), Box<dyn Error>> {
        match create_workspace {
            true => match AuthenticationService::get_registration_params() {
                Ok(mut loaded_workspaces) => {
                    loaded_workspaces.push(workspace_params.clone());
                    AuthenticationService::create_registration_params(loaded_workspaces)
                }
                Err(e) => Err(e),
            },
            false => {
                let workspaces = vec![workspace_params.clone()];
                AuthenticationService::create_registration_params(workspaces)
            }
        }
    }

    fn get_registration_params() -> Result<Vec<WorkSpace>, Box<dyn Error>> {
        let path = Self::get_storage_path()?;
        let mut f = File::open(path)?;
        let mut buf = String::new();
        f.read_to_string(&mut buf)?;
        let info: Vec<WorkSpace> = serde_json::from_str(&buf)?;
        if info.is_empty() {
            return Err("No workspace available".into());
        }
        Ok(info)
    }

    pub(crate) fn modify_workspace_name(
        new_name: &str,
        workspace_id: &str,
    ) -> Result<(), Box<dyn Error>> {
        match AuthenticationService::get_registration_params() {
            Ok(mut workspaces) => match workspaces.iter_mut().find(|w| w.id == workspace_id) {
                Some(workspace) => {
                    workspace.workspace_name = new_name.to_string();

                    AuthenticationService::create_registration_params(workspaces)
                }
                None => Err("Workspace not found!".into()),
            },
            Err(e) => {
                if cfg!(debug_assertions) {
                    println!("Error while getting the registeration params : {e}");
                }
                Err(e)
            }
        }
    }

    pub(crate) fn delete_workspace_name(workspace_id: &str) -> Result<(), Box<dyn Error>> {
        match AuthenticationService::get_registration_params() {
            Ok(mut workspaces) => {
                let original_len = workspaces.len();

                workspaces.retain(|w| w.id != workspace_id);

                if workspaces.len() == original_len {
                    return Err("Workspace not found!".into());
                }

                AuthenticationService::create_registration_params(workspaces)
            }
            Err(e) => {
                if cfg!(debug_assertions) {
                    println!("Error while getting the registeration params : {e}");
                }
                Err(e)
            }
        }
    }

    pub(crate) fn check_registered() -> bool {
        match AuthenticationService::get_registration_params() {
            Ok(_) => true,
            Err(e) => {
                if cfg!(debug_assertions) {
                    println!("Error while getting the file : {e}");
                }
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

    /// Signup Function
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

        // Generating workspace id
        let workspace_id = Uuid::new_v4().to_string();

        match EncryptionService::encrypt_with_password(&workspace_id, &app.password, &master_key) {
            Ok(registration_params) => {
                app.password.zeroize(); // !!WARNING zeroizing password here as not needed further
                let mut workspace_entry: WorkSpace = WorkSpace {
                    id: workspace_id,
                    workspace_name: app.workspace.clone(),
                    params: registration_params,
                };
                app.workspace_id = workspace_entry.id.clone();
                match AuthenticationService::save_registration_params(
                    &workspace_entry,
                    app.create_workspace,
                ) {
                    Ok(_) => {
                        workspace_entry.zeroize();
                    }
                    Err(e) => {
                        app.errors.register = format!("{e}");
                        return;
                    }
                }
            }
            Err(e) => {
                if cfg!(debug_assertions) {
                    println!("Error when saving params: {e}");
                }
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
                match DatabaseService::new(&app.db_key, app.workspace_id.clone()) {
                    Ok(mut db_service) => {
                        // Run migrations
                        match db_service.init(MIGRATIONS) {
                            // Inserting default config
                            Ok(()) => {
                                // Saving current app config in db
                                let conn = db_service.get_connection();

                                app.config.insert(conn);

                                if cfg!(debug_assertions) {
                                    println!("Saving Config: {:?}", app.config);
                                }

                                Activity::seed_default_activities(conn);

                                app.database_service = Some(db_service);

                                // Adding Introdution Note
                                let mut introduction_note = Note::make_introduction();
                                match NoteService::save_note_hash_blob(
                                    &app.workspace_id,
                                    &app.database_service,
                                    &app.file_key,
                                    &mut introduction_note,
                                    true,
                                ) {
                                    Ok(_) => {
                                        if cfg!(debug_assertions) {
                                            println!("Introduction note created!");
                                        }
                                        app.get_all_notes();
                                    }
                                    Err(e) => {
                                        if cfg!(debug_assertions) {
                                            println!("Error while creating introduction note: {e}");
                                        }
                                    }
                                }

                                let tab_id = introduction_note.id;

                                let initial_tab = Tab {
                                    name: introduction_note.title.clone(),
                                    index: 0,
                                    note: introduction_note,
                                };

                                app.tabs_controller.tabs.push(initial_tab);
                                app.tabs_controller.active_note_tab_id = tab_id;

                                app.toasts.success("Signed up!");
                                app.current_page = Page::Editor
                            }
                            Err(e) => {
                                if cfg!(debug_assertions) {
                                    println!("Error while running migrations db: {e}");
                                }
                            }
                        }
                    }
                    Err(e) => {
                        if cfg!(debug_assertions) {
                            println!("Error while getting db while registering: {e}");
                        }
                    }
                };
            }
            Err(e) => {
                if cfg!(debug_assertions) {
                    println!("Error while fetching json: {e}");
                }
                app.errors.register = "Coud not fetch saved".to_string();
            }
        }
    }

    // Function to handle verifying password/ Login
    pub(crate) fn verify_password(app: &mut MindSafeApp) {
        match AuthenticationService::get_registration_params() {
            Ok(mut workspaces) => {
                // if cfg!(debug_assertions) {println!("loaded = {loaded:?}",);}
                if let Some(workspace) = workspaces
                    .iter()
                    .find(|w| w.id == app.workspace_id)
                    .or_else(|| workspaces.first())
                {
                    // In case first workspace is selected
                    app.workspace = workspace.workspace_name.clone();
                    app.workspace_id = workspace.id.clone();

                    match EncryptionService::decrypt_with_password(&app.password, &workspace.params)
                    {
                        Ok(mut master_key) => {
                            app.password.zeroize(); // !!WARNING zeroizing password here as not needed further

                            let (db_key, file_key) = EncryptionService::generate_keys(&master_key);
                            master_key.zeroize(); // !!WARNING zeroizing master_key here as not needed further

                            app.db_key = db_key; // moved to main state no need to zeroize
                            app.file_key = file_key; // moved to main state no need to zeroize

                            // Starting DB
                            match DatabaseService::new(&app.db_key, workspace.id.clone()) {
                                Ok(mut db_service) =>
                                // Run migrations
                                {
                                    match db_service.init(MIGRATIONS) {
                                        // Inserting default config
                                        Ok(()) => match Config::get(db_service.get_connection()) {
                                            Ok(config) => {
                                                // Getting saved config from db
                                                Activity::seed_default_activities(
                                                    db_service.get_connection(),
                                                );

                                                app.config = config;
                                                app.database_service = Some(db_service);
                                                app.toasts.success("Logged in!");
                                                app.get_all_notes();
                                                workspaces.zeroize();
                                                app.current_page = Page::Editor;
                                            }
                                            Err(e) => {
                                                if cfg!(debug_assertions) {
                                                    println!("Error fetching config: {e}");
                                                }

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
                                            if cfg!(debug_assertions) {
                                                println!("Error while running migrations db: {_e}");
                                            }
                                        }
                                    }
                                }
                                Err(_e) => {
                                    if cfg!(debug_assertions) {
                                        println!("Error while getting db while loggin: {_e}");
                                    }
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
                } else {
                    app.has_registered = false;
                    app.current_page = Page::Register;
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

    pub(crate) fn workspace_dropdown(ui: &mut Ui, app: &mut MindSafeApp) {
        ui.columns_const(|[_col0, col1, col2, _col3]| {
            col1.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                ui.label("Workspace");
            });
            col2.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                match AuthenticationService::get_registration_params() {
                    Ok(workspaces) => {
                        let default = WorkSpace {
                            id: String::from("Select Workspace"),
                            workspace_name: String::from("Select Workspace"),
                            params: EncryptedBlob::default(),
                        };
                        let workspace = workspaces
                            .iter()
                            .find(|w| w.id == app.workspace_id)
                            .or_else(|| workspaces.first())
                            .unwrap_or(&default);
                        ui.horizontal(|ui| {
                            egui::ComboBox::from_id_salt("workspace-box")
                                .selected_text(workspace.workspace_name.clone())
                                // .width(f32::MAX)
                                .show_ui(ui, |ui| {
                                    for w in &workspaces {
                                        ui.selectable_value(
                                            &mut app.workspace_id,
                                            w.id.clone(),
                                            &w.workspace_name,
                                        );
                                    }
                                });
                            if ui.button("Create Workspace").clicked() {
                                app.current_page = Page::Register;
                                app.create_workspace = true;
                                app.password = String::new();
                                app.workspace = String::from("New Workspace");
                            }
                        });
                    }
                    Err(e) => {
                        ui.label("Workspaces cannot be loaded");
                        if cfg!(debug_assertions) {
                            println!("Cannot fetch workspaces due to: {e}");
                        }
                        if !AuthenticationService::check_registered() {
                            app.has_registered = false;
                            app.current_page = Page::Register;
                        }
                    }
                }
            });
        });
    }
}
