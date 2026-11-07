use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub fn relative_time(updated_ms: u64) -> String {
    let now = now_ms();
    let diff = now.saturating_sub(updated_ms);
    let secs = diff / 1000;
    if secs < 60 {
        "just now".to_string()
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else if secs < 86400 {
        format!("{}h", secs / 3600)
    } else {
        format!("{}d", secs / 86400)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum NoteKind {
    Text,
    Checklist,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub text: String,
    pub done: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Note {
    pub id: u64,
    pub title: String,
    pub kind: NoteKind,
    pub body: String,
    pub items: Vec<ChecklistItem>,
    pub pinned: bool,
    pub created: u64,
    pub updated: u64,
}

impl Note {
    pub fn new_text() -> Self {
        let now = now_ms();
        Self {
            id: now,
            title: String::new(),
            kind: NoteKind::Text,
            body: String::new(),
            items: Vec::new(),
            pinned: false,
            created: now,
            updated: now,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.title.trim().is_empty()
            && self.body.trim().is_empty()
            && self.items.iter().all(|i| i.text.trim().is_empty())
    }

    pub fn display_title(&self) -> String {
        if !self.title.trim().is_empty() {
            self.title.clone()
        } else if !self.body.trim().is_empty() {
            self.body.lines().next().unwrap_or("").to_string()
        } else if !self.items.is_empty() {
            self.items[0].text.clone()
        } else {
            String::new()
        }
    }

    pub fn preview(&self) -> String {
        match self.kind {
            NoteKind::Text => {
                let first_line = self.body.lines().next().unwrap_or("");
                if first_line.len() > 28 {
                    format!("{}...", &first_line[..26])
                } else {
                    first_line.to_string()
                }
            }
            NoteKind::Checklist => {
                let done = self.items.iter().filter(|i| i.done).count();
                let total = self.items.len();
                if total == 0 {
                    "Empty checklist".to_string()
                } else {
                    format!("{}/{} done", done, total)
                }
            }
        }
    }

    pub fn checklist_progress(&self) -> (usize, usize) {
        let done = self.items.iter().filter(|i| i.done).count();
        (done, self.items.len())
    }
}

pub struct NotesManager {
    pub notes: Vec<Note>,
    pub selected_id: Option<u64>,
    pub pending_delete: Option<(Note, Instant)>,
    pub search_text: String,
    pub search_active: bool,
    pub toast_message: Option<(String, Instant)>,
    pub last_typed: Option<Instant>,
    pub autosave_pending: bool,
}

impl NotesManager {
    pub fn new() -> Self {
        let mut mgr = Self {
            notes: Vec::new(),
            selected_id: None,
            pending_delete: None,
            search_text: String::new(),
            search_active: false,
            toast_message: None,
            last_typed: None,
            autosave_pending: false,
        };
        mgr.load();
        mgr
    }

    fn notes_path() -> PathBuf {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(appdata).join("NotchBar").join("notes.json")
    }

    pub fn load(&mut self) {
        let path = Self::notes_path();
        if let Ok(data) = fs::read_to_string(&path) {
            // Support deserializing both new format and old format
            #[derive(Deserialize)]
            struct OldNote {
                id: usize,
                content: String,
                #[serde(default)]
                is_completed: bool,
            }

            if let Ok(notes) = serde_json::from_str::<Vec<Note>>(&data) {
                // Migration: filter out old placeholder notes
                self.notes = notes
                    .into_iter()
                    .filter(|n| !is_placeholder(n))
                    .collect();
            } else if let Ok(old_notes) = serde_json::from_str::<Vec<OldNote>>(&data) {
                self.notes = old_notes
                    .into_iter()
                    .filter(|n| !n.content.starts_with("New task or quick memo") && !n.content.starts_with("New Note"))
                    .map(|n| {
                        let now = now_ms() + n.id as u64;
                        Note {
                            id: now,
                            title: String::new(),
                            kind: NoteKind::Text,
                            body: n.content,
                            items: Vec::new(),
                            pinned: false,
                            created: now,
                            updated: now,
                        }
                    })
                    .collect();
            }
        }
        self.sort_notes();
        if let Some(first) = self.notes.first() {
            self.selected_id = Some(first.id);
        }
    }

    pub fn save(&self) {
        let path = Self::notes_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&self.notes) {
            let tmp = path.with_extension("json.tmp");
            if fs::write(&tmp, &json).is_ok() {
                let _ = fs::rename(&tmp, &path);
            }
        }
    }

    pub fn sort_notes(&mut self) {
        self.notes.sort_by(|a, b| {
            b.pinned
                .cmp(&a.pinned)
                .then(b.updated.cmp(&a.updated))
        });
    }

    pub fn add_note(&mut self) -> u64 {
        // If current note is completely empty, don't create another duplicate empty note
        if let Some(sel_id) = self.selected_id {
            if let Some(note) = self.notes.iter().find(|n| n.id == sel_id) {
                if note.is_empty() {
                    return sel_id;
                }
            }
        }
        let note = Note::new_text();
        let id = note.id;
        self.notes.insert(0, note);
        self.selected_id = Some(id);
        self.save();
        id
    }

    pub fn selected_note(&self) -> Option<&Note> {
        self.selected_id.and_then(|id| self.notes.iter().find(|n| n.id == id))
    }

    pub fn selected_note_mut(&mut self) -> Option<&mut Note> {
        self.selected_id.and_then(|id| self.notes.iter_mut().find(|n| n.id == id))
    }

    pub fn update_selected_title(&mut self, title: &str) {
        if let Some(note) = self.selected_note_mut() {
            if note.title != title {
                note.title = title.to_string();
                note.updated = now_ms();
                self.autosave_pending = true;
                self.last_typed = Some(Instant::now());
            }
        }
    }

    pub fn update_selected_body(&mut self, body: &str) {
        if let Some(note) = self.selected_note_mut() {
            if note.body != body {
                note.body = body.to_string();
                note.updated = now_ms();
                self.autosave_pending = true;
                self.last_typed = Some(Instant::now());
            }
        }
    }

    pub fn flush_save_if_pending(&mut self) {
        if self.autosave_pending {
            self.sort_notes();
            self.save();
            self.autosave_pending = false;
        }
    }

    pub fn delete_note(&mut self, id: u64) {
        if let Some(pos) = self.notes.iter().position(|n| n.id == id) {
            let note = self.notes.remove(pos);
            self.pending_delete = Some((note, Instant::now()));
            self.toast_message = Some(("Note deleted · Undo".to_string(), Instant::now()));
            if self.selected_id == Some(id) {
                self.selected_id = self.notes.first().map(|n| n.id);
            }
            self.save();
        }
    }

    pub fn undo_delete(&mut self) {
        if let Some((note, _)) = self.pending_delete.take() {
            let id = note.id;
            self.notes.push(note);
            self.sort_notes();
            self.selected_id = Some(id);
            self.save();
            self.toast_message = None;
        }
    }

    pub fn toggle_pin(&mut self, id: u64) {
        if let Some(note) = self.notes.iter_mut().find(|n| n.id == id) {
            note.pinned = !note.pinned;
            note.updated = now_ms();
        }
        self.sort_notes();
        self.save();
    }

    pub fn delete_empty_selected(&mut self) {
        if let Some(id) = self.selected_id {
            let is_empty = self.notes.iter().find(|n| n.id == id).map(|n| n.is_empty()).unwrap_or(false);
            if is_empty {
                self.notes.retain(|n| n.id != id);
                self.selected_id = self.notes.first().map(|n| n.id);
                self.save();
            }
        }
    }

    pub fn tick_toast(&mut self) -> bool {
        if let Some((_, start)) = &self.toast_message {
            if start.elapsed() > Duration::from_secs(5) {
                self.toast_message = None;
                self.pending_delete = None;
                return true;
            }
        }
        false
    }

    pub fn filtered_notes(&self) -> Vec<&Note> {
        if self.search_text.trim().is_empty() {
            self.notes.iter().collect()
        } else {
            let q = self.search_text.to_lowercase();
            self.notes
                .iter()
                .filter(|n| {
                    n.title.to_lowercase().contains(&q)
                        || n.body.to_lowercase().contains(&q)
                        || n.items.iter().any(|i| i.text.to_lowercase().contains(&q))
                })
                .collect()
        }
    }

    pub fn latest_note_title(&self) -> String {
        self.notes
            .first()
            .map(|n| {
                let t = n.display_title();
                if t.is_empty() { "Untitled".to_string() } else { t }
            })
            .unwrap_or_default()
    }
}

fn is_placeholder(note: &Note) -> bool {
    let content = format!("{} {}", note.title, note.body).trim().to_string();
    content.starts_with("New task or quick memo")
        || content.starts_with("New Note")
        || (note.title.trim() == "Title" && note.body.trim().is_empty())
}

