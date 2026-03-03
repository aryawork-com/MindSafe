use uuid::Uuid;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::models::note::Note;

#[derive(Debug)]
pub(crate) struct Tab {
    pub name: String,
    /// will act as sorting value
    pub index: u8,
    pub note: Note,
}

impl Tab {
    pub(crate) fn new(note: &Note, index: u8) -> Self {
        Self {
            name: note.title.clone(),
            index,
            note: note.clone(),
        }
    }
}

impl Zeroize for Tab {
    fn zeroize(&mut self) {
        self.name.zeroize();
        self.index.zeroize();
        self.note.zeroize();
    }
}

impl Default for Tab {
    fn default() -> Self {
        Self {
            name: "Introduction".to_string(),
            index: 0,
            note: Note::make_introduction(),
        }
    }
}

impl ZeroizeOnDrop for Tab {}

#[derive(Debug)]
pub(crate) struct TabsController {
    pub tabs: Vec<Tab>,
    pub active_note_tab_id: Uuid,
}

impl Zeroize for TabsController {
    fn zeroize(&mut self) {
        self.tabs.zeroize();
        self.active_note_tab_id = Uuid::nil();
    }
}

impl ZeroizeOnDrop for TabsController {}

impl Default for TabsController {
    fn default() -> Self {
        Self {
            tabs: vec![Tab::default()],
            active_note_tab_id: Uuid::nil(),
        }
    }
}
