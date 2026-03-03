// external libraries
use egui::{
    Align, CentralPanel, Color32, Context, CursorIcon, Key, KeyboardShortcut, Layout, Modifiers,
    RichText, TextEdit, Ui,
};
use egui_i18n::tr;
use egui_material_icons::icons::{
    ICON_LOGIN, ICON_PRIORITY_HIGH, ICON_VISIBILITY, ICON_VISIBILITY_OFF, ICON_ZOOM_IN,
    ICON_ZOOM_OUT,
};
use zeroize::{Zeroize, ZeroizeOnDrop};

// Internal modules usage
use crate::{MindSafeApp, i18n, services::authentication::AuthenticationService};

/// Page enum - "equivalent" to Routes Definition
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Page {
    Register = 0,
    Login = 1,
    Editor = 2,
}

impl Page {
    pub(crate) fn is_register(&self) -> bool {
        matches!(self, Page::Register)
    }
}

#[derive(Default, Debug)]
pub(crate) struct AppErrors {
    pub register: String,
    pub login: String,
}

impl Zeroize for AppErrors {
    fn zeroize(&mut self) {
        self.register.zeroize();
        self.login.zeroize();
    }
}

impl ZeroizeOnDrop for AppErrors {}

pub(crate) fn register_page(app: &mut MindSafeApp, ctx: &Context) {
    CentralPanel::default().show(ctx, |ui| {
        let available_height = ui.available_size().y;
        // let available_width = ui.available_size().x;
        // Make a full-height column and center inside it
        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            ui.add_space(available_height * 0.2); // push content downward a bit

            mind_safe_logo(ui);

            ui.colored_label(Color32::LIGHT_GRAY, tr!("welcome-text"));
            ui.add_space(60.0);

            i18n::language_dropdown(ui, &mut app.config.selected_language);
            ui.add_space(10.0);

            ui.columns_const(|[_col0, col1, col2, _col3]| {
                col1.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                    ui.label(tr!("password"));
                    ui.colored_label(Color32::KHAKI, tr!("password-hint-1"));
                    ui.colored_label(Color32::KHAKI, tr!("password-hint-2"));
                    ui.colored_label(Color32::KHAKI, tr!("password-hint-3"));
                });
                col2.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                    password_input(ui, app)
                });
            });

            ui.add_space(20.0);

            ui.colored_label(Color32::LIGHT_RED, app.errors.register.clone());

            ui.add_space(50.0);

            if ui
                .button(format!("{ICON_LOGIN}  {}", tr!("register")))
                .clicked()
            {
                AuthenticationService::save_password(app);
            }

            zoom_buttons(ui, ctx)
        });
    });
}

fn password_input(ui: &mut Ui, app: &mut MindSafeApp) {
    ui.horizontal(|ui| {
        ui.add(
            TextEdit::singleline(&mut app.password)
                .password(app.hide_password)
                .char_limit(50)
                .hint_text(tr!("enter-password")),
        );
        if app.hide_password {
            if ui
                .button(format!("{ICON_VISIBILITY}  {}", tr!("view")))
                .clicked()
            {
                app.hide_password = !app.hide_password;
            }
        } else if ui
            .button(format!("{ICON_VISIBILITY_OFF}  {}", tr!("hide")))
            .clicked()
        {
            app.hide_password = !app.hide_password;
        }
    });
}

fn mind_safe_logo(ui: &mut Ui) {
    ui.add(
        egui::Image::new(egui::include_image!("../assets/icon.svg"))
            .max_width(70.0)
            .corner_radius(10.0),
    );
    ui.add_space(10.0);

    ui.heading(RichText::new(tr!("mind-safe")).color(Color32::WHITE));
    ui.scope(|ui| {
        ui.style_mut().visuals.interact_cursor = Some(CursorIcon::PointingHand);
        for (_text_style, font_id) in ui.style_mut().text_styles.iter_mut() {
            font_id.size = 12.0
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
    ui.add_space(30.0);
}

fn zoom_buttons(ui: &mut Ui, ctx: &Context) {
    ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
        ui.horizontal(|ui| {
            if ui
                .button(format!("{ICON_ZOOM_IN}  {}", tr!("zoom-in")))
                .clicked()
            {
                zoom(ctx, 1.1);
            }

            if ui
                .button(format!("{ICON_ZOOM_OUT}  {}", tr!("zoom-out")))
                .clicked()
            {
                zoom(ctx, 1.0 / 1.1);
            }
        });
    });

    // ----- Keyboard Shortcuts -----
    // MacOS -> Command, Others -> Ctrl
    #[cfg(target_os = "macos")]
    let zoom_mod = Modifiers::COMMAND;
    #[cfg(not(target_os = "macos"))]
    let zoom_mod = Modifiers::CTRL;

    // Cmd/Ctrl + Plus
    if ctx.input_mut(|i| i.consume_shortcut(&KeyboardShortcut::new(zoom_mod, Key::Plus))) {
        zoom(ctx, 1.1);
    }

    // Cmd/Ctrl + Minus
    if ctx.input_mut(|i| i.consume_shortcut(&KeyboardShortcut::new(zoom_mod, Key::Minus))) {
        zoom(ctx, 1.0 / 1.1);
    }

    // Cmd/Ctrl + 0 -> reset zoom
    if ctx.input_mut(|i| i.consume_shortcut(&KeyboardShortcut::new(zoom_mod, Key::Num0))) {
        ctx.set_pixels_per_point(1.0);
    }
}

fn zoom(ctx: &egui::Context, factor: f32) {
    let mut scale = ctx.pixels_per_point();
    scale *= factor;
    ctx.set_pixels_per_point(scale.clamp(0.5, 3.0)); // keep sane limits
}

pub(crate) fn login_page(app: &mut MindSafeApp, ctx: &Context) {
    CentralPanel::default().show(ctx, |ui| {
        let available_height = ui.available_size().y;
        // let available_width = ui.available_size().x;
        // Make a full-height column and center inside it
        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            ui.add_space(available_height * 0.2); // push content downward a bit

            mind_safe_logo(ui);

            ui.colored_label(Color32::LIGHT_GRAY, tr!("welcome-back"));
            ui.add_space(50.0);

            i18n::language_dropdown(ui, &mut app.config.selected_language);
            ui.add_space(10.0);

            ui.columns_const(|[_col0, col1, col2, _col3]| {
                col1.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                    ui.label(tr!("password"));
                });
                col2.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                    password_input(ui, app)
                });
            });

            ui.add_space(20.0);

            ui.colored_label(Color32::LIGHT_RED, &app.errors.login);
            if app.errors.login.is_empty() {
                ui.colored_label(
                    Color32::LIGHT_BLUE,
                    format!("{ICON_PRIORITY_HIGH}  Once you press Login or Enter, it will take some time to load!"),
                );
            }

            ui.add_space(50.0);
            if ui.button(tr!("login")).clicked() {
                app.errors.login = String::new();
                AuthenticationService::verify_password(app);
            }
            if ctx.input_mut(|i| i.key_down(Key::Enter)) {
                app.errors.login = String::new();
                AuthenticationService::verify_password(app);
            };
            zoom_buttons(ui, ctx)
        });
    });
}

pub(crate) fn editor_page(app: &mut MindSafeApp, ctx: &Context) {
    // Size panel for app
    // Side panel before top bar to ensure top bar remains separate.
    app.create_side_panel(ctx);

    // Top bar for app
    app.create_top_bar(ctx);

    // Editor Section
    app.create_editor_panel(ctx);
}
