// native libraries
use std::sync::Arc;

// external libraries
use egui::{
    CentralPanel, Color32, ComboBox, Context, CursorIcon, Rangef, RichText, ScrollArea, SidePanel,
    Stroke, TextEdit, TopBottomPanel, Ui, panel::Side, vec2,
};
use egui_i18n::tr;
use egui_material_icons::icons::{
    ICON_CLOSE, ICON_CONTENT_COPY, ICON_DELETE, ICON_EDIT, ICON_FILE_DOWNLOAD, ICON_FILE_UPLOAD,
    ICON_GPP_BAD, ICON_GPP_GOOD, ICON_LOGOUT, ICON_NOTE_ADD, ICON_SAVE, ICON_SETTINGS,
    ICON_TROUBLESHOOT, ICON_UPDATE,
};

// Internal modules usage
use crate::{
    MindSafeApp,
    constants::HELP_TEXT,
    layouter,
    modals::Modal,
    models::{config::SortingScheme, note::Note},
    services::{
        notes::NoteService,
        tabs::{Tab, TabsController},
    },
};

// Top bar having Tabs and Top Buttons
pub(crate) fn create_top_bar(app: &mut MindSafeApp, ctx: &Context) {
    TopBottomPanel::top("toolbar").show(ctx, |ui| {
        ui.vertical(|ui| {
            // Tabs Controller
            // Tabs Controller with wrapping
            ScrollArea::horizontal().show(ui, |ui| {
                ui.add_space(5.0);
                ui.with_layout(
                    egui::Layout::left_to_right(egui::Align::Min).with_main_wrap(true),
                    |ui| {
                        let mut to_close: Option<usize> = None;

                        let tab_count = app.tabs_controller.tabs.len();
                        for i in 0..tab_count {
                            if tab_tile(i, &mut app.tabs_controller, ui) {
                                to_close = Some(i);
                            }
                        }

                        if let Some(i) = to_close {
                            app.tabs_controller.tabs.remove(i);
                        }
                    },
                );
                ui.add_space(3.0);
            });
            ui.separator();
            // Top Buttons Menu
            create_top_button_bar(app, ui);
            ui.add_space(2.0); // Adding for looks
        });
    });
}

// Side Panel having Logo, Notes, Bottom Buttons
pub(crate) fn create_side_panel(app: &mut MindSafeApp, ctx: &Context) {
    SidePanel::new(Side::Left, "side-panel-main")
        .resizable(true)
        .width_range(Rangef::new(250.0, 400.0))
        .show(ctx, |ui| {
            ui.add_space(10.0);
            // Top Logo and Name
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                ui.vertical(|ui| {
                    // Adding for proper alignedment and look
                    ui.add_space(5.0);
                    ui.add(
                        egui::Image::new(egui::include_image!("../assets/icon.svg"))
                            .fit_to_exact_size(vec2(25.0, 25.0)),
                    );
                });
                ui.add_space(5.0);
                ui.vertical(|ui| {
                    ui.scope(|ui| {
                        for (_text_style, font_id) in ui.style_mut().text_styles.iter_mut() {
                            font_id.size = 16.0 // whatever size you want here
                        }
                        ui.label(tr!("mind-safe"))
                            .on_hover_text("https://aryawork.com");
                    });
                    ui.scope(|ui| {
                        ui.style_mut().visuals.interact_cursor = Some(CursorIcon::PointingHand);
                        for (_text_style, font_id) in ui.style_mut().text_styles.iter_mut() {
                            font_id.size = 12.0 // whatever size you want here
                        }
                        if ui
                            .add(egui::Label::new("by Aryawork").sense(egui::Sense::click()))
                            .clicked()
                        {
                            #[cfg(target_os = "windows")]
                            {
                                let _ = std::process::Command::new("cmd")
                                    .args(&["/C", "start", "https://aryawork.com"])
                                    .spawn();
                            }
                            #[cfg(target_os = "macos")]
                            {
                                let _ = std::process::Command::new("open")
                                    .arg("https://aryawork.com")
                                    .spawn();
                            }
                            #[cfg(target_os = "linux")]
                            {
                                let _ = std::process::Command::new("xdg-open")
                                    .arg("https://aryawork.com")
                                    .spawn();
                            }
                        }
                    });
                });

                ui.separator();
                if app.config.safe_copy {
                    ui.colored_label(Color32::LIGHT_GREEN, format!("{ICON_GPP_GOOD}  Safe",))
                } else {
                    ui.colored_label(Color32::LIGHT_RED, format!("{ICON_GPP_BAD}  Unsafe",))
                };
            });
            ui.add_space(10.0);
            ui.separator();

            // ---------- Notes Sorting & Search ----------
            // TODO
            ComboBox::from_id_salt("sorting_dropdown")
                .selected_text(app.config.sorting.name())
                .width(ui.available_width())
                .show_ui(ui, |ui| {
                    for scheme in SortingScheme::ALL_SCHEMES {
                        let resp =
                            ui.selectable_value(&mut app.config.sorting, scheme, scheme.name());
                        if resp.changed() {
                            app.get_all_notes();
                        }
                    }
                });

            ui.add_space(5.0);

            // ---------- Scrollable Notes List ----------
            // Reserve space for footer (85 px)
            let total_height = ui.available_height();
            let footer_height = 85.0;
            let notes_height = (total_height - footer_height).max(0.0);
            ScrollArea::vertical()
                .max_height(notes_height.max(0.0)) // clamp so it never goes negative
                .show(ui, |ui| {
                    let notes = app.all_notes.clone();
                    for note in &notes {
                        if app.note_tile(ui, note).clicked() {
                            // only add if not already open
                            let exists = app
                                .tabs_controller
                                .tabs
                                .iter()
                                .any(|tab| tab.note.id == note.id);

                            if !exists {
                                let new_idx = app.tabs_controller.tabs.len();
                                let new_tab = Tab::new(note, new_idx as u8);
                                app.tabs_controller.tabs.push(new_tab);
                            }

                            // always update the active note
                            app.tabs_controller.active_note_tab_id = note.id;
                        }

                        ui.add_space(1.5);
                    }
                });

            // Force remaining space to be allocated (pushes footer down)
            let remaining_height = ui.available_height();
            if remaining_height > footer_height {
                ui.allocate_space(egui::Vec2::new(0.0, remaining_height - footer_height));
            }

            // Bottom section - always at bottom
            // --- Bottom layered sections ---
            ui.scope(|ui| {
                ui.style_mut().visuals.widgets.noninteractive.bg_stroke =
                    Stroke::new(1.0, Color32::from_gray(90));

                ui.separator();
            });
            ui.add_space(5.0);

            // First bottom layer: version info
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                ui.label(RichText::new(format!("V: {}", env!("CARGO_PKG_VERSION"))));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(format!("{ICON_UPDATE}  Check Update",)).clicked() {
                        app.current_modal = Modal::CheckUpdate;
                        app.show_modal = true;
                    }
                    ui.separator();
                });
            });

            ui.add_space(5.0);
            ui.separator();
            ui.add_space(5.0);

            // Second bottom layer: buttons
            ui.horizontal(|ui| {
                if ui.button(format!("{ICON_SETTINGS}  Settings")).clicked() {
                    app.show_modal = true;
                }
                if ui
                    .button(format!("{ICON_TROUBLESHOOT}  Activities"))
                    .clicked()
                {
                    app.show_modal = true;
                }
                // Logout aligned to right
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button(
                            RichText::new(format!("{ICON_LOGOUT}  Logout"))
                                .color(Color32::LIGHT_RED),
                        )
                        .clicked()
                    {
                        // Handle exit
                    }
                    ui.add_space(10.0);
                });
            });
        });
}
pub(crate) fn create_editor_panel(app: &mut MindSafeApp, ctx: &Context) {
    // Current Active tab Id
    let active_note_id = app.tabs_controller.active_note_tab_id;

    // Get either the active note tab mutably, or fall back to app.text
    let note_text: &mut String = if let Some(active_tab) = app
        .tabs_controller
        .tabs
        .iter_mut()
        .find(|tab| tab.note.id == active_note_id)
    {
        // println!(
        //     "Blob: {}; Text: {}",
        //     active_tab.note.blob, active_tab.note.text
        // );
        if !active_tab.note.blob.is_empty() && active_tab.note.text.is_empty() {
            match NoteService::get_decrypted_text(&app.file_key, &mut active_tab.note) {
                Ok(text) => {
                    active_tab.note.text = text;
                }
                Err(e) => {
                    app.toasts.success(format!("Error while getting text: {e}"));
                }
            }
        }

        &mut active_tab.note.text
    } else {
        &mut app.text
    };

    CentralPanel::default().show(ctx, |ui: &mut Ui| {
        ScrollArea::both().show(ui, |ui| {
            let available = ui.available_size();
            let highlight_enabled = app.config.syntax_highlight;

            // Layouter with syntax highlighting
            let mut md_layouter = move |ui: &egui::Ui,
                                        buf: &dyn egui::TextBuffer,
                                        wrap_width: f32|
                  -> Arc<egui::Galley> {
                let mut job = layouter::layouter(buf.as_str(), highlight_enabled);
                job.wrap.max_width = wrap_width;
                ui.fonts_mut(|f| f.layout_job(job))
            };

            let te = TextEdit::multiline(note_text)
                .code_editor()
                .background_color(Color32::from_rgb(17, 17, 17))
                .lock_focus(true)
                .desired_width(f32::INFINITY)
                .desired_rows(available.y as usize / 18) // estimate rows
                .hint_text(HELP_TEXT)
                .layouter(&mut md_layouter);

            ui.add_sized(available, te);
        });
    });
}
pub(crate) fn create_top_button_bar(app: &mut MindSafeApp, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.horizontal(|ui| {
            if ui.button(format!("{ICON_NOTE_ADD}  New")).clicked() {
                // app.new
                app.current_modal = Modal::CreateNote;
                app.show_modal = true;
            }
            if ui.button(format!("{ICON_SAVE}  Save")).clicked() {
                let active_note_id = app.tabs_controller.active_note_tab_id;

                // First get index instead of mutable reference
                let tab_index = app
                    .tabs_controller
                    .tabs
                    .iter()
                    .position(|tab| tab.note.id == active_note_id);

                if let Some(index) = tab_index {
                    // Now borrow only the note mutably
                    let note: &mut Note = &mut app.tabs_controller.tabs[index].note.clone();

                    match NoteService::save_note_hash_blob(
                        &app.database_service,
                        &app.file_key,
                        note,
                    ) {
                        Ok(_) => {
                            app.toasts.success("Saved!");
                        }
                        Err(e) => {
                            app.toasts.error(format!("Failed to save: {e}!"));
                        }
                    }
                }
            }
            if ui
                .button(format!("{ICON_FILE_DOWNLOAD}  Import",))
                .clicked()
            {
                // app.save_as();
            }
            // if ui.button(format!("{ICON_FILE_UPLOAD}  Export",)).clicked() {
            // }

            ui.separator();

            // Safe copy checkbox
            ui.scope(|ui| {
                if !app.config.safe_copy {
                    ui.style_mut().visuals.widgets.inactive.bg_stroke =
                        Stroke::new(1.0, Color32::LIGHT_RED);
                    ui.style_mut().visuals.widgets.inactive.fg_stroke =
                        Stroke::new(1.0, Color32::LIGHT_RED);
                    ui.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::LIGHT_RED;
                    ui.style_mut().visuals.widgets.inactive.bg_fill = Color32::LIGHT_RED;
                    ui.style_mut().visuals.widgets.hovered.bg_stroke =
                        Stroke::new(1.0, Color32::GREEN);
                    ui.style_mut().visuals.widgets.hovered.fg_stroke =
                        Stroke::new(1.0, Color32::GREEN);
                    ui.style_mut().visuals.widgets.hovered.weak_bg_fill = Color32::GREEN;
                    ui.style_mut().visuals.widgets.hovered.bg_fill = Color32::GREEN;
                }

                ui.checkbox(&mut app.config.safe_copy, "Safe Copy")
                    .on_hover_text(
                        "Safe Copy prevents data from being written directly to disk.\n\
                    Instead, it works with a temporary copy to avoid accidental overwrites.",
                    )
            });

            ui.separator();

            // highlight
            ui.checkbox(&mut app.config.syntax_highlight, "Syntax Highlight")
                .on_hover_text(
                    "Safe Copy prevents data from being written directly to disk.\n\
                    Instead, it works with a temporary copy to avoid accidental overwrites.",
                );

            ui.separator();

            // ui.horizontal(|ui| {
            //     ui.label("Tags:");

            //     ComboBox::from_id_salt("tags_dropdown")
            //         .selected_text(format!("{} min", app.config.auto_save_duration))
            //         .show_ui(ui, |ui| {
            //             for minutes in 0..=60 {
            //                 ui.selectable_value(
            //                     &mut app.config.auto_save_duration,
            //                     minutes,
            //                     format!("{minutes} min"),
            //                 );
            //             }
            //         });
            // });

            // ui.separator();

            ui.horizontal(|ui| {
                ui.label("Auto-Save:");

                ComboBox::from_id_salt("autosave_dropdown")
                    .selected_text(format!("{} min", app.config.auto_save_duration))
                    .show_ui(ui, |ui| {
                        for minutes in 0..=60 {
                            ui.selectable_value(
                                &mut app.config.auto_save_duration,
                                minutes,
                                format!("{minutes} min"),
                            );
                        }
                    });
            });

            ui.separator();

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // ui.add_space(10.0);
                ui.label(format!("Words: {}", app.word_count()));
                ui.separator();
                ui.label(format!("Chars: {}", app.char_count()));
                ui.separator();
            });
        });
        // ui.add_space(3.0)
    });
}
pub(crate) fn note_tile(app: &mut MindSafeApp, ui: &mut Ui, note: &Note) -> egui::Response {
    let color = if app.current_note.id.to_string() == note.id.to_string() {
        Color32::GRAY
    } else {
        ui.visuals().window_stroke.color.gamma_multiply(0.7)
    };
    let width = ui.available_width();

    ui.allocate_exact_size(egui::vec2(width, 0.0), egui::Sense::click());

    // Constrain the width first, before creating the frame
    ui.allocate_ui_with_layout(
        egui::Vec2::new(width, 0.0),
        egui::Layout::top_down(egui::Align::LEFT),
        |ui| {
            egui::Frame::NONE
                .fill(ui.visuals().window_fill)
                .stroke(egui::Stroke::new(1.0, color))
                .corner_radius(egui::CornerRadius::same(8))
                .inner_margin(egui::Margin::same(10))
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        // Title (larger, on its own line)
                        ui.label(
                            RichText::new(&note.title)
                                .strong()
                                .size(13.0)
                                .color(ui.visuals().text_color()),
                        );

                        // Add spacing between title and timestamp
                        ui.add_space(4.0);

                        // Timestamp (smaller, right-aligned)
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                RichText::new(format!(
                                    "{}",
                                    note.updated_at.format("%b %d, %H:%M")
                                ))
                                .size(10.0)
                                .color(ui.visuals().weak_text_color()),
                            );
                            if ui.button(ICON_EDIT).on_hover_text("Edit").clicked() {
                                //self
                                app.current_note = note.clone();
                                app.current_modal = Modal::UpdateNote;
                                app.show_modal = true;
                            }
                            if ui
                                .button(ICON_CONTENT_COPY)
                                .on_hover_text("Duplicate")
                                .clicked()
                            {
                                if let Some(db_service) = &app.database_service {
                                    // TODO: Blob has to be recrypted with new id specific salt
                                    let status = note.duplicate(db_service.get_connection());
                                    if status {
                                        app.get_all_notes();
                                        app.show_modal = false;
                                        app.current_modal = Modal::Settings;
                                        app.toasts.success("Note duplicated!");
                                    } else {
                                        app.toasts.error("Note was not duplicated!");
                                    }
                                } else {
                                    app.toasts.error("Database not found! Restart the app!");
                                }
                            }
                            if ui
                                .button(RichText::new(ICON_DELETE).color(Color32::LIGHT_RED))
                                .on_hover_text("Delete")
                                .clicked()
                            {
                                //self
                                app.current_note = note.clone();
                                app.current_modal = Modal::DeleteNote;
                                app.show_modal = true;
                            }
                        });

                        // Add spacing before tags

                        // Tags section
                        if let Some(tags) = &note.tags
                            && !tags.is_empty()
                        {
                            ui.add_space(8.0);
                            ui.horizontal_wrapped(|ui| {
                                ui.spacing_mut().item_spacing.x = 6.0;
                                ui.spacing_mut().item_spacing.y = 4.0;

                                for tag in tags {
                                    let tag_color = tag.color;
                                    egui::Frame::NONE
                                        .fill(tag_color.gamma_multiply(0.15))
                                        .stroke(egui::Stroke::new(
                                            1.0,
                                            tag_color.gamma_multiply(0.6),
                                        ))
                                        .corner_radius(egui::CornerRadius::same(12))
                                        .inner_margin(egui::Margin::symmetric(10, 4))
                                        .show(ui, |ui| {
                                            ui.label(
                                                RichText::new(&tag.name)
                                                    .size(9.0)
                                                    .color(tag_color.gamma_multiply(1.3)),
                                            );
                                        });
                                }
                            });
                        }
                    });
                });
        },
    )
    .response
}

pub(crate) fn tab_tile(tab_index: usize, tabs: &mut TabsController, ui: &mut egui::Ui) -> bool {
    let tab_id = tabs.tabs[tab_index].note.id;
    let tab_name: String = tabs.tabs[tab_index].name.clone();
    let is_active = tab_id == tabs.active_note_tab_id;

    let mut button_clicked = false;
    let mut tab_clicked = false;

    let fill = if is_active {
        ui.visuals().extreme_bg_color // highlighted background
    } else {
        ui.visuals().faint_bg_color // subtle background
    };

    let _ = egui::Frame::NONE
        .fill(fill)
        .stroke(egui::Stroke::new(
            1.0,
            ui.visuals().widgets.inactive.fg_stroke.color,
        ))
        .corner_radius(egui::CornerRadius::same(3))
        .inner_margin(egui::Margin {
            left: 8,
            right: 8,
            top: 6,
            bottom: 4,
        })
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // clickable area for tab (except close button)
                let tab_resp = ui.add(
                    egui::Label::new(
                        egui::RichText::new(&tab_name)
                            .strong()
                            .size(13.0)
                            .color(ui.visuals().text_color()),
                    )
                    .sense(egui::Sense::click()), // <--- this makes label clickable
                );

                if tab_resp.clicked() {
                    tab_clicked = true;
                }

                if ui.button(ICON_CLOSE).clicked() {
                    println!("Tab Index: {tab_index}");
                    button_clicked = true;
                }
            });
        })
        .response;

    if tab_clicked && !button_clicked {
        tabs.active_note_tab_id = tab_id;
    }

    button_clicked
}
