use ::egui::Key;
// external libraries
use ::zeroize::{Zeroize, ZeroizeOnDrop};
use egui::{Align2, Color32, ComboBox, Context, RichText, TextEdit, Vec2};
use egui_material_icons::icons::{
    ICON_BACKUP, ICON_BOOKMARK_MANAGER, ICON_CLOSE, ICON_DELETE, ICON_FILE_OPEN, ICON_FOLDER_OPEN,
    ICON_KEY, ICON_LOCK, ICON_UPDATE, ICON_WARNING,
};
use rfd::FileDialog;

// Internal modules usage
use crate::{
    MindSafeApp,
    models::note::Note,
    services::{
        authentication::AuthenticationService, data::DataService, database::DatabaseService,
    },
};

/// Modal enum - "equivalent" to Modal Routes Definition
pub(crate) enum Modal {
    Settings = 0,
    ChangePassword = 1,
    ChangeKey = 2,
    CheckUpdate = 3,
    CreateNote = 4,
    DeleteNote = 5,
    UpdateNote = 6,
    ImportNote = 7,
    DeleteWorkspace = 8,
    UpdateWorkspace = 9,
}

#[derive(Default)]
pub(crate) struct PasswordChange {
    current_password: String,
    new_password: String,
}

impl Zeroize for PasswordChange {
    fn zeroize(&mut self) {
        self.current_password.zeroize();
        self.new_password.zeroize();
    }
}

impl ZeroizeOnDrop for PasswordChange {}

pub(crate) fn settings_modal(app: &mut MindSafeApp, ctx: &Context) {
    egui::Window::new("Settings")
        .collapsible(false)
        .resizable(false)
        .default_size(Vec2::new(600.0, 400.0))
        .min_size(Vec2::new(600.0, 400.0))
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                // Directory
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.add_space(5.0);

                    if app.config.main_directory().is_dir() {
                        // Directory path in a non-editable "box"
                        ui.add(
                            TextEdit::singleline(
                                &mut app.config.main_directory().to_string_lossy().to_string(),
                            )
                            .hint_text("Main directory")
                            .desired_width(250.0)
                            .interactive(false), // make it read-only
                        );

                        if ui
                            .button(format!("{ICON_BOOKMARK_MANAGER}  Change"))
                            .clicked()
                            && let Some(directory) = FileDialog::new().pick_folder()
                        {
                            app.config.set_main_directory(directory);
                        }
                    } else {
                        ui.add(
                            TextEdit::singleline(&mut "No directory selected".to_string())
                                .desired_width(250.0)
                                .interactive(false),
                        );

                        if ui.button(format!("{ICON_FOLDER_OPEN}  Select")).clicked()
                            && let Some(directory) = FileDialog::new().pick_folder()
                        {
                            app.config.set_main_directory(directory);
                        }
                    }

                    ui.add_space(5.0);
                });

                ui.add_space(15.0);

                // Auto Lock
                ui.horizontal(|ui| {
                    ui.add_space(5.0);
                    ui.label("Auto-lock:");

                    ComboBox::from_id_salt("auto_lock_duration")
                        .selected_text(format!("{} min", app.config.auto_lock_duration))
                        .show_ui(ui, |ui| {
                            for minutes in 0..=60 {
                                ui.selectable_value(
                                    &mut app.config.auto_lock_duration,
                                    minutes,
                                    format!("{minutes} mins"),
                                );
                            }
                        });
                });
                ui.horizontal(|ui| {
                    ui.add_space(5.0);
                    ui.colored_label(Color32::YELLOW, "0 means never!");
                });

                ui.add_space(15.0);

                // Auto Save
                ui.horizontal(|ui| {
                    ui.add_space(5.0);
                    ui.label("Auto-save:");

                    ComboBox::from_id_salt("auto_save_duration")
                        .selected_text(format!("{} min", app.config.auto_save_duration))
                        .show_ui(ui, |ui| {
                            for minutes in 0..=60 {
                                ui.selectable_value(
                                    &mut app.config.auto_save_duration,
                                    minutes,
                                    format!("{minutes} mins"),
                                );
                            }
                        });
                });
                ui.horizontal(|ui| {
                    ui.add_space(5.0);
                    ui.colored_label(Color32::YELLOW, "0 means never!");
                });

                ui.add_space(15.0);

                ui.horizontal(|ui| {
                    ui.add_space(5.0);

                    // Change password
                    if ui.button(format!("{ICON_LOCK}  Change Password")).clicked() {
                        //app.pass
                        app.current_modal = Modal::ChangePassword;
                    }
                    ui.add_space(15.0);

                    // Change master key
                    if ui
                        .button(format!("{ICON_KEY}  Change Master Key"))
                        .clicked()
                    {
                        //app.pass
                        app.current_modal = Modal::ChangeKey;
                    }
                });

                ui.add_space(15.0);

                // Backup directory

                ui.horizontal(|ui| {
                    ui.add_space(5.0);

                    if app.config.backup_directory().is_dir() {
                        // Directory path in a non-editable "box"
                        ui.add(
                            TextEdit::singleline(
                                &mut app.config.backup_directory().to_string_lossy().to_string(),
                            )
                            .hint_text("Backup directory")
                            .desired_width(250.0)
                            .interactive(false), // make it read-only
                        );

                        if ui
                            .button(format!("{ICON_BOOKMARK_MANAGER}  Change"))
                            .clicked()
                            && let Some(directory) = FileDialog::new().pick_folder()
                        {
                            app.config.set_backup_directory(directory);
                        }

                        // Backup button
                        if ui.button(format!("{ICON_BACKUP}  Backup")).clicked() {
                            // app.backup
                        }
                    } else {
                        ui.add(
                            TextEdit::singleline(&mut "No backup directory selected".to_string())
                                .desired_width(250.0)
                                .interactive(false),
                        );

                        if ui.button(format!("{ICON_FOLDER_OPEN}  Select")).clicked()
                            && let Some(directory) = FileDialog::new().pick_folder()
                        {
                            app.config.set_backup_directory(directory);
                        }
                    }

                    ui.add_space(5.0);
                });
                ui.horizontal(|ui| {
                    ui.add_space(5.0);
                    ui.colored_label(Color32::YELLOW, "Should be different from main directory!");
                });

                ui.add_space(20.0);

                if ui
                    .button(RichText::new(format!("{ICON_CLOSE}  Close")).color(Color32::LIGHT_RED))
                    .clicked()
                {
                    app.show_modal = false;
                }
            });
        });
}

pub(crate) fn import_modal(app: &mut MindSafeApp, ctx: &Context) {
    egui::Window::new("Import Notes")
        .collapsible(false)
        .resizable(false)
        .default_size(Vec2::new(600.0, 400.0))
        .min_size(Vec2::new(600.0, 400.0))
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                // File
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.add_space(5.0);

                    ui.add(
                        TextEdit::singleline(&mut "Select a note file(s) to import".to_string())
                            .desired_width(250.0)
                            .interactive(false),
                    );

                    if ui
                        .button(format!("{ICON_FILE_OPEN}  Select File(s)"))
                        .clicked()
                        && let Some(directory) = FileDialog::new().pick_files()
                    {
                        if cfg!(debug_assertions) {
                            println!("File to be imported is {directory:?}");
                        }

                        for file in directory {
                            DataService::import_notes(app, file);
                        }

                        app.current_modal = Modal::Settings;
                        app.show_modal = false;
                    }

                    ui.add_space(5.0);
                });

                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.add_space(5.0);
                    ui.colored_label(
                        Color32::YELLOW,
                        "Import note(s) from a Markdown (MD) or Text (txt) file(s).",
                    );
                });

                ui.add_space(20.0);

                if ui
                    .button(RichText::new(format!("{ICON_CLOSE}  Close")).color(Color32::LIGHT_RED))
                    .clicked()
                {
                    app.current_modal = Modal::Settings;
                    app.show_modal = false;
                }
            });
        });
}

pub(crate) fn change_password_modal(app: &mut MindSafeApp, ctx: &Context) {
    egui::Window::new("Change Password")
        .collapsible(false)
        .resizable(false)
        .default_size(Vec2::new(600.0, 400.0))
        .min_size(Vec2::new(600.0, 400.0))
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                // Directory
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.add_space(5.0);

                    ui.add(
                        TextEdit::singleline(&mut app.changed_password_data.current_password)
                            .hint_text("Current Password")
                            // .desired_width(250.0)
                            .password(app.hide_password),
                    );
                });

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.add_space(5.0);

                    ui.add(
                        TextEdit::singleline(&mut app.changed_password_data.new_password)
                            .hint_text("New Password")
                            .password(app.hide_password)
                            .char_limit(50),
                    );
                });

                ui.horizontal(|ui| {
                    ui.add_space(5.0);
                    ui.colored_label(
                        Color32::LIGHT_RED,
                        format!("{ICON_WARNING} Do not close the app until complete"),
                    );
                });

                ui.add_space(15.0);

                ui.horizontal(|ui| {
                    if ui
                        .button(format!("{ICON_UPDATE}  Update Password"))
                        .clicked()
                    {
                        //app.pass
                        app.current_modal = Modal::ChangePassword;
                    }

                    ui.add_space(20.0);

                    if ui
                        .button(
                            RichText::new(format!("{ICON_CLOSE}  Close")).color(Color32::LIGHT_RED),
                        )
                        .clicked()
                    {
                        app.current_modal = Modal::Settings;
                        app.changed_password_data = PasswordChange::default();
                        app.show_modal = false;
                    }
                });
            });
        });
}

pub(crate) fn change_key_modal(app: &mut MindSafeApp, ctx: &Context) {
    egui::Window::new("Change Master Key")
        .collapsible(false)
        .resizable(false)
        .default_size(Vec2::new(600.0, 400.0))
        .min_size(Vec2::new(600.0, 400.0))
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.add_space(5.0);

                    ui.add(
                        TextEdit::singleline(&mut app.password)
                            .hint_text("Current Password")
                            // .desired_width(250.0)
                            .password(app.hide_password),
                    );
                });

                ui.horizontal(|ui| {
                    ui.add_space(5.0);
                    ui.colored_label(
                        Color32::LIGHT_RED,
                        format!("{ICON_WARNING} Do not close the app until complete"),
                    );
                });

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button(format!("{ICON_UPDATE}  Update Key")).clicked() {
                        //app.pass
                        app.current_modal = Modal::ChangePassword;
                    }

                    ui.add_space(20.0);

                    if ui
                        .button(
                            RichText::new(format!("{ICON_CLOSE}  Close")).color(Color32::LIGHT_RED),
                        )
                        .clicked()
                    {
                        app.current_modal = Modal::Settings;
                        app.password = String::new();
                        app.show_modal = false;
                    }
                });
            });
        });
}

pub(crate) fn new_note_modal(app: &mut MindSafeApp, ctx: &Context) {
    egui::Window::new("Create New Note")
        .collapsible(false)
        .resizable(false)
        .default_size(Vec2::new(600.0, 400.0))
        .min_size(Vec2::new(600.0, 400.0))
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.add_space(10.0);

                ui.add(
                    TextEdit::singleline(&mut app.new_note.title)
                        .char_limit(60)
                        .hint_text("Note Title"),
                );
                ui.add_space(3.0);
                ui.colored_label(Color32::YELLOW, "max 60 chars");
                ui.add_space(15.0);

                ui.horizontal(|ui| {
                    if ui.button(format!("{ICON_UPDATE}  Create Note")).clicked() {
                        if app.new_note.title.is_empty() {
                            app.toasts.error("Name is required!");
                        } else if let Some(db_service) = &app.database_service {
                            app.new_note.refresh_id();
                            let status = app.new_note.insert(db_service.get_connection());
                            if status {
                                app.toasts.success("Note created!");
                                app.get_all_notes();
                                app.new_note.set_default();
                                app.current_modal = Modal::Settings;
                                app.show_modal = false;
                            } else {
                                app.toasts.error("Note was not created!");
                            }
                        } else {
                            app.toasts.error("Database not found! Restart the app!");
                        }
                    }

                    if ctx.input_mut(|i| i.key_down(Key::Enter)) {
                        if app.new_note.title.is_empty() {
                            app.toasts.error("Name is required!");
                        } else if let Some(db_service) = &app.database_service {
                            app.new_note.refresh_id();
                            let status = app.new_note.insert(db_service.get_connection());
                            if status {
                                app.toasts.success("Note created!");
                                app.get_all_notes();
                                app.new_note.set_default();
                                app.current_modal = Modal::Settings;
                                app.show_modal = false;
                            } else {
                                app.toasts.error("Note was not created!");
                            }
                        } else {
                            app.toasts.error("Database not found! Restart the app!");
                        }
                    };

                    ui.add_space(20.0);

                    if ui
                        .button(
                            RichText::new(format!("{ICON_CLOSE}  Close")).color(Color32::LIGHT_RED),
                        )
                        .clicked()
                    {
                        app.new_note.refresh_id();
                        app.current_modal = Modal::Settings;
                        app.show_modal = false;
                    }
                });
            });
        });
}

pub(crate) fn update_note_modal(app: &mut MindSafeApp, ctx: &Context) {
    egui::Window::new("Edit Note")
        .collapsible(false)
        .resizable(false)
        .default_size(Vec2::new(600.0, 400.0))
        .min_size(Vec2::new(600.0, 400.0))
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.add_space(10.0);

                ui.add(
                    TextEdit::singleline(&mut app.current_note.title)
                        .char_limit(60)
                        .hint_text("Note Title"),
                );
                ui.add_space(3.0);
                ui.colored_label(Color32::YELLOW, "max 60 chars");
                ui.add_space(15.0);

                ui.horizontal(|ui| {
                    if ui.button(format!("{ICON_UPDATE}  Update")).clicked() {
                        //app.pass
                        if app.current_note.title.is_empty() {
                            app.toasts.error("Name is required!");
                        } else if let Some(db_service) = &app.database_service {
                            let status = app.current_note.update_title(db_service.get_connection());
                            if status {
                                app.get_all_notes();
                                app.current_modal = Modal::Settings;
                                app.show_modal = false;
                                app.toasts.success("Note updated!");
                            } else {
                                app.toasts.error("Note was not updated!");
                            }
                        } else {
                            app.toasts.error("Database not found! Restart the app!");
                        }
                    }

                    ui.add_space(20.0);

                    if ui
                        .button(
                            RichText::new(format!("{ICON_CLOSE}  Close")).color(Color32::LIGHT_RED),
                        )
                        .clicked()
                    {
                        app.current_modal = Modal::Settings;
                        app.show_modal = false;
                    }
                });
            });
        });
}

pub(crate) fn delete_note_modal(app: &mut MindSafeApp, ctx: &Context) {
    egui::Window::new("Delete Note")
        .collapsible(false)
        .resizable(false)
        .default_size(Vec2::new(600.0, 400.0))
        .min_size(Vec2::new(600.0, 400.0))
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.add_space(10.0);

                ui.add(
                    TextEdit::singleline(&mut app.confirm_text)
                        .char_limit(60)
                        .hint_text("Type DELETE"),
                );
                ui.add_space(15.0);

                ui.horizontal(|ui| {
                    if ui
                        .button(
                            RichText::new(format!("{ICON_DELETE}  DELETE"))
                                .color(Color32::LIGHT_RED),
                        )
                        .clicked()
                    {
                        if app.confirm_text == "DELETE" {
                            //app.pass
                            if let Some(db_service) = &app.database_service {
                                let status = app.current_note.delete(db_service.get_connection());
                                if status {
                                    app.current_note = Note::default();
                                    app.get_all_notes();
                                    app.current_modal = Modal::Settings;
                                    app.show_modal = false;
                                    app.confirm_text = String::new(); // Resetting confirm text
                                    app.toasts.success("Note deleted!");
                                } else {
                                    app.toasts.error("Note was not deleted!");
                                }
                            } else {
                                app.toasts.error("Database not found! Restart the app!");
                            }
                        } else {
                            app.toasts.error("Confirmation text was incorrect!");
                        }
                    }

                    ui.add_space(20.0);

                    if ui.button(format!("{ICON_CLOSE}  Close")).clicked() {
                        app.new_note.refresh_id();
                        app.current_modal = Modal::Settings;
                        app.show_modal = false;
                    }
                });
            });
        });
}

pub(crate) fn delete_workspace_modal(app: &mut MindSafeApp, ctx: &Context) {
    egui::Window::new("Delete Workspace")
        .collapsible(false)
        .resizable(false)
        .default_size(Vec2::new(600.0, 400.0))
        .min_size(Vec2::new(600.0, 400.0))
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.add_space(5.0);
                    ui.colored_label(
                        Color32::YELLOW,
                        "Once Deleted Workspace CANNOT be recovered.",
                    );
                });

                ui.add(
                    TextEdit::singleline(&mut app.confirm_text)
                        .char_limit(60)
                        .hint_text("Type DELETE"),
                );
                ui.add_space(15.0);

                ui.horizontal(|ui| {
                    if ui
                        .button(
                            RichText::new(format!("{ICON_DELETE}  DELETE"))
                                .color(Color32::LIGHT_RED),
                        )
                        .clicked()
                    {
                        if app.confirm_text == "DELETE" {
                            //app.pass
                            match AuthenticationService::delete_workspace_name(&app.workspace_id) {
                                Ok(_) => {
                                    match DatabaseService::delete_db(app.workspace_id.clone()) {
                                        Ok(_) => {
                                            app.logout();
                                        }
                                        Err(_) => {
                                            // Already added debug message internally in function
                                        }
                                    }
                                }
                                Err(e) => {
                                    if cfg!(debug_assertions) {
                                        println!("Error while deleting workspace: {e}");
                                    }

                                    app.toasts.error("Workspace was not deleted!");
                                }
                            }
                        } else {
                            app.toasts.error("Confirmation text was incorrect!");
                        }
                    }

                    ui.add_space(20.0);

                    if ui.button(format!("{ICON_CLOSE}  Close")).clicked() {
                        app.new_note.refresh_id();
                        app.current_modal = Modal::Settings;
                        app.show_modal = false;
                    }
                });
            });
        });
}

pub(crate) fn update_workspace_modal(app: &mut MindSafeApp, ctx: &Context) {
    egui::Window::new("Update Workspace")
        .collapsible(false)
        .resizable(false)
        .default_size(Vec2::new(600.0, 400.0))
        .min_size(Vec2::new(600.0, 400.0))
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.add_space(10.0);

                ui.add(
                    TextEdit::singleline(&mut app.workspace)
                        .char_limit(60)
                        .hint_text("Workspace Name"),
                );
                ui.add_space(3.0);
                ui.colored_label(Color32::YELLOW, "max 60 chars");
                ui.add_space(15.0);

                ui.horizontal(|ui| {
                    if ui.button(format!("{ICON_UPDATE}  Update")).clicked() {
                        //app.pass
                        if app.workspace.is_empty() {
                            app.toasts.error("Name is required!");
                        }

                        match AuthenticationService::modify_workspace_name(
                            &app.workspace,
                            &app.workspace_id,
                        ) {
                            Ok(_) => {
                                app.current_modal = Modal::Settings;
                                app.show_modal = false;
                                app.toasts.success("Workspace updated!");
                            }
                            Err(e) => {
                                if cfg!(debug_assertions) {
                                    println!("Error while deleting workspace: {e}");
                                }

                                app.toasts.error("Workspace was not deleted!");
                            }
                        }
                    }

                    ui.add_space(20.0);

                    if ui
                        .button(
                            RichText::new(format!("{ICON_CLOSE}  Close")).color(Color32::LIGHT_RED),
                        )
                        .clicked()
                    {
                        app.current_modal = Modal::Settings;
                        app.show_modal = false;
                    }
                });
            });
        });
}
