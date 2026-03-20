use egui::{Context, Key, KeyboardShortcut, Modifiers};

use crate::MindSafeApp;

pub(crate) fn editor_shortcuts(app: &mut MindSafeApp, ctx: &Context) {
    // ----- Keyboard Shortcuts for Editor -----

    // MacOS -> Command, Others -> Ctrl
    #[cfg(target_os = "macos")]
    let command_mod = Modifiers::COMMAND;
    #[cfg(not(target_os = "macos"))]
    let command_mod = Modifiers::CTRL;

    // Cmd/Ctrl + S -> save note text in editor
    if ctx.input_mut(|i| i.consume_shortcut(&KeyboardShortcut::new(command_mod, Key::S))) {
        app.save_note_text();
    }
}
