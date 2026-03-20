/// Command to reloads application in each change - `cargo watch -c -w src -x run`
/// Command to build application - `cargo bundle --release`
/// Get instructions to install [cargo watch](https://crates.io/crates/cargo-watch)
///
/// Use 2 spaces after Icon in format!() for better visibility
// native libraries
use std::time::Instant;

// external libraries
use egui::{Context, Response, Ui, Vec2};
use egui_i18n::tr;
use egui_notify::{Anchor, Toasts};
use uuid::Uuid;
use zeroize::{Zeroize, ZeroizeOnDrop};

// Module Declarations
pub mod constants;
pub mod i18n;
pub mod models;
pub mod services;
pub mod shortcuts;
pub mod ui;

// Internal modules usage
use crate::{
    constants::HELP_TEXT,
    models::{config::Config, note::Note},
    services::{
        authentication::AuthenticationService, database::DatabaseService, notes::NoteService,
        tabs::TabsController,
    },
    ui::{
        modals::{self, Modal, PasswordChange},
        pages::{self, AppErrors, Page},
        widgets,
    },
};

/// Application struct - Stores application level state variables
///
/// NOTE: Do not put SENSITIVE vars here
struct MindSafeApp {
    confirm_text: String,
    password: String,
    db_key: Vec<u8>,
    file_key: Vec<u8>,
    session_id: Uuid,
    workspace_id: String,
    workspace: String,
    hide_password: bool,
    current_page: Page,
    database_service: Option<DatabaseService>,
    tabs_controller: TabsController,
    config: Config,
    has_registered: bool,
    create_workspace: bool,
    text: String,
    toasts: Toasts,
    new_note: Note,
    all_notes: Vec<Note>,
    show_modal: bool,
    current_modal: Modal,
    changed_password_data: PasswordChange,
    current_note: Note,
    last_auto_saved: Instant,
    errors: AppErrors,
}

impl Zeroize for MindSafeApp {
    fn zeroize(&mut self) {
        self.confirm_text.zeroize();
        self.password.zeroize();
        self.db_key.zeroize();
        self.file_key.zeroize();
        self.workspace = String::from("Select Workspace");
        self.workspace_id = String::new();
        self.database_service = None;
        self.has_registered.zeroize();
        self.config.zeroize();
        self.hide_password = true;
        self.create_workspace = false;
        self.current_page = Page::Register;
        self.session_id = Uuid::nil();
        self.text.zeroize();
        self.toasts = Toasts::default();
        self.tabs_controller.zeroize();
        self.new_note.zeroize();
        self.all_notes.zeroize();
        self.show_modal.zeroize();
        self.current_modal = Modal::Settings;
        self.changed_password_data.zeroize();
        self.current_note.zeroize();
        self.last_auto_saved = Instant::now();
        self.errors.zeroize();
    }
}

impl ZeroizeOnDrop for MindSafeApp {}

impl Default for MindSafeApp {
    fn default() -> Self {
        Self {
            confirm_text: String::new(),
            password: match cfg!(debug_assertions) {
                true => String::from("pAssword@2026#"),
                false => String::new(),
            },
            db_key: Vec::new(),
            file_key: Vec::new(),
            session_id: Uuid::new_v4(),
            database_service: None,
            new_note: Note::default(),
            has_registered: AuthenticationService::check_registered(),
            config: Config::default(),
            hide_password: true,
            tabs_controller: TabsController::default(),
            all_notes: Vec::new(),
            current_page: Page::Register,
            workspace_id: String::new(),
            workspace: String::from("Default Workspace"),
            text: String::from(HELP_TEXT),
            toasts: Toasts::default()
                .with_anchor(Anchor::BottomRight)
                // Added margin in toasts for better viewing experience
                .with_margin(Vec2::new(20.0, 20.0)),
            show_modal: false,
            create_workspace: false,
            current_modal: Modal::Settings,
            last_auto_saved: Instant::now(),
            changed_password_data: PasswordChange::default(),
            current_note: Note::default(),
            errors: AppErrors::default(),
        }
    }
}

impl MindSafeApp {
    // ---------- Helper Functions
    fn word_count(&self) -> usize {
        // Current Active tab Id
        let active_note_id = self.tabs_controller.active_note_tab_id;
        let tabs_controller = &self.tabs_controller;

        let word_count: usize = if let Some(active_tab) = tabs_controller
            .tabs
            .iter()
            .find(|tab| tab.note.id == active_note_id)
        {
            if !active_tab.note.text.is_empty() {
                active_tab
                    .note
                    .text
                    .split_whitespace()
                    .filter(|w| !w.is_empty())
                    .count()
            } else {
                0
            }
        } else {
            0
        };

        word_count
    }

    fn char_count(&self) -> usize {
        // Current Active tab Id
        let active_note_id = self.tabs_controller.active_note_tab_id;
        let tabs_controller = &self.tabs_controller;

        let word_count: usize = if let Some(active_tab) = tabs_controller
            .tabs
            .iter()
            .find(|tab| tab.note.id == active_note_id)
        {
            if !active_tab.note.text.is_empty() {
                active_tab.note.text.chars().count()
            } else {
                0
            }
        } else {
            0
        };

        word_count
    }

    fn get_all_notes(&mut self) {
        if let Some(db_service) = &self.database_service {
            match NoteService::get_all_notes(db_service.get_connection(), &self.config.sorting) {
                Ok(notes) => {
                    self.all_notes = notes;
                }
                Err(e) => {
                    println!("Error fetching notes: {e}");
                    self.toasts.error("Error fetching notes, restart the app!");
                }
            }
        } else {
            self.toasts.error("Database not found! Restart the app!");
        }
    }

    // ---------- Widget Functions

    fn note_tile(&mut self, ui: &mut Ui, note: &Note) -> Response {
        let available_width = ui.available_width();

        // Let egui decide the height based on children
        let inner = ui.allocate_ui_with_layout(
            egui::vec2(available_width, f32::INFINITY), // width fixed, height grows
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                widgets::note_tile(self, ui, note); // paint title, icons, date, etc.
            },
        );

        // Now take the response and make it clickable
        inner.response.interact(egui::Sense::click())
    }

    // ---------- Layout Element Functions
    fn create_top_bar(&mut self, ctx: &Context) {
        widgets::create_top_bar(self, ctx);
    }

    fn create_side_panel(&mut self, ctx: &Context) {
        widgets::create_side_panel(self, ctx);
    }

    fn create_editor_panel(&mut self, ctx: &Context) {
        widgets::create_editor_panel(self, ctx);
    }

    // ---------- Modal Functions
    fn settings_modal(&mut self, ctx: &Context) {
        modals::settings_modal(self, ctx);
    }

    fn change_password_modal(&mut self, ctx: &Context) {
        modals::change_password_modal(self, ctx);
    }

    fn delete_note_modal(&mut self, ctx: &Context) {
        modals::delete_note_modal(self, ctx);
    }

    fn delete_workspace_modal(&mut self, ctx: &Context) {
        modals::delete_workspace_modal(self, ctx);
    }

    fn update_workspace_modal(&mut self, ctx: &Context) {
        modals::update_workspace_modal(self, ctx);
    }

    fn change_key_modal(&mut self, ctx: &Context) {
        modals::change_key_modal(self, ctx);
    }

    fn new_note_modal(&mut self, ctx: &Context) {
        modals::new_note_modal(self, ctx);
    }

    fn update_note_modal(&mut self, ctx: &Context) {
        modals::update_note_modal(self, ctx);
    }

    fn import_note_modal(&mut self, ctx: &Context) {
        modals::import_modal(self, ctx);
    }

    // ---------- Page Functions
    fn register_page(&mut self, ctx: &Context) {
        pages::register_page(self, ctx);
    }

    fn login_page(&mut self, ctx: &Context) {
        pages::login_page(self, ctx);
    }

    fn editor_page(&mut self, ctx: &Context) {
        pages::editor_page(self, ctx);
    }

    fn auto_save_notes(&mut self) {
        if self.database_service.is_none() {
            return;
        }

        let tabs = &mut self.tabs_controller.tabs;
        let toasts = &mut self.toasts;
        let mut saved: bool = false;
        let mut notes_len: i32 = 0;

        for tab in tabs.iter_mut() {
            if !tab.note.text.is_empty() {
                match NoteService::save_note_hash_blob(
                    &self.workspace_id,
                    &self.database_service,
                    &self.file_key,
                    &mut tab.note,
                    false,
                ) {
                    Ok(_) => {
                        saved = true;
                        notes_len += 1
                    }
                    Err(e) => {
                        toasts.error(format!("Failed to auto save note: {e}!"));
                    }
                }
            }
        }

        if saved {
            toasts.success(format!("{notes_len} notes auto saved!"));
        }
    }

    // helper functions
    pub fn save_note_text(&mut self) {
        let active_note_id = self.tabs_controller.active_note_tab_id;

        // First get index instead of mutable reference
        let tab_index = self
            .tabs_controller
            .tabs
            .iter()
            .position(|tab| tab.note.id == active_note_id);

        if let Some(index) = tab_index {
            // Now borrow only the note mutably
            let note: &mut Note = &mut self.tabs_controller.tabs[index].note.clone();

            match NoteService::save_note_hash_blob(
                &self.workspace_id,
                &self.database_service,
                &self.file_key,
                note,
                false,
            ) {
                Ok(_) => {
                    self.toasts.success("Saved!");
                }
                Err(e) => {
                    self.toasts.error(format!("Failed to save: {e}!"));
                }
            }
        }
    }

    fn logout(&mut self) {
        self.zeroize();
        self.current_page = Page::Login;
    }
}

/// Update function to change application frame - "equivalent" to a router
impl eframe::App for MindSafeApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let now: Instant = Instant::now();
        let interval = std::time::Duration::from_secs((self.config.auto_save_duration * 60) as u64);

        if self.has_registered && self.current_page.is_register() && !self.create_workspace {
            self.current_page = Page::Login;
        }

        match self.current_page {
            Page::Register => self.register_page(ctx),
            Page::Login => self.login_page(ctx),
            Page::Editor => self.editor_page(ctx),
        }
        self.toasts.show(ctx);

        // Window must be called every frame if the flag is true
        if self.show_modal {
            match self.current_modal {
                Modal::Settings => self.settings_modal(ctx),
                Modal::ChangePassword => self.change_password_modal(ctx),
                Modal::ChangeKey => self.change_key_modal(ctx),
                Modal::CheckUpdate => self.settings_modal(ctx),
                Modal::CreateNote => self.new_note_modal(ctx),
                Modal::DeleteNote => self.delete_note_modal(ctx),
                Modal::UpdateNote => self.update_note_modal(ctx),
                Modal::ImportNote => self.import_note_modal(ctx),
                Modal::DeleteWorkspace => self.delete_workspace_modal(ctx),
                Modal::UpdateWorkspace => self.update_workspace_modal(ctx),
            }
        }

        if now.duration_since(self.last_auto_saved) >= interval {
            self.auto_save_notes();
            self.last_auto_saved = now;
        }

        // Don’t repaint continuously — only repaint when something happens.
        ctx.request_repaint_after(interval);
    }
}

// Main function
fn main() -> eframe::Result<()> {
    i18n::init_languages();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1920.0, 1080.0])
            .with_min_inner_size([700.0, 500.0])
            .with_icon(
                eframe::icon_data::from_png_bytes(&include_bytes!("../icons/icon.png")[..])
                    .expect("Failed to load icon"),
            )
            .with_title(tr!("mind-safe"))
            .with_clamp_size_to_monitor_size(true)
            .with_decorations(true),

        ..Default::default()
    };

    eframe::run_native(
        "MindSafe",
        options,
        Box::new(|cc| {
            egui_material_icons::initialize(&cc.egui_ctx);
            // For image support
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(MindSafeApp::default()))
        }),
    )
}
