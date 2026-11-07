use std::fs;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct TimerItem {
    pub id: usize,
    pub name: String,
    pub total_seconds: u32,
    pub remaining_seconds: u32,
    pub is_running: bool,
    pub last_tick: Option<Instant>,
}

#[derive(Clone, Debug)]
pub struct NoteItem {
    pub id: usize,
    pub content: String,
    pub is_completed: bool,
}

pub struct ProductivityManager {
    pub timers: Vec<TimerItem>,
    pub notes: Vec<NoteItem>,
    pub parked_files: Vec<PathBuf>,
    next_timer_id: usize,
    next_note_id: usize,
}

impl ProductivityManager {
    pub fn new() -> Self {
        let mut mgr = Self {
            timers: vec![
                TimerItem {
                    id: 1,
                    name: "Focus Timer".to_string(),
                    total_seconds: 25 * 60,
                    remaining_seconds: 25 * 60,
                    is_running: false,
                    last_tick: None,
                },
                TimerItem {
                    id: 2,
                    name: "Short Break".to_string(),
                    total_seconds: 5 * 60,
                    remaining_seconds: 5 * 60,
                    is_running: false,
                    last_tick: None,
                },
            ],
            notes: Vec::new(),
            parked_files: Vec::new(),
            next_timer_id: 3,
            next_note_id: 1,
        };
        mgr.load_notes();
        mgr
    }

    pub fn tick_timers(&mut self) -> Option<String> {
        let mut finished_timer = None;
        for timer in &mut self.timers {
            if timer.is_running {
                let now = Instant::now();
                if let Some(last) = timer.last_tick {
                    let elapsed = now.duration_since(last).as_secs() as u32;
                    if elapsed >= 1 {
                        if timer.remaining_seconds > elapsed {
                            timer.remaining_seconds -= elapsed;
                        } else {
                            timer.remaining_seconds = 0;
                            timer.is_running = false;
                            finished_timer = Some(timer.name.clone());
                        }
                        timer.last_tick = Some(now);
                    }
                } else {
                    timer.last_tick = Some(now);
                }
            }
        }
        finished_timer
    }

    pub fn toggle_timer(&mut self, id: usize) {
        if let Some(t) = self.timers.iter_mut().find(|t| t.id == id) {
            t.is_running = !t.is_running;
            t.last_tick = if t.is_running { Some(Instant::now()) } else { None };
        }
    }

    pub fn reset_timer(&mut self, id: usize) {
        if let Some(t) = self.timers.iter_mut().find(|t| t.id == id) {
            t.remaining_seconds = t.total_seconds;
            t.is_running = false;
            t.last_tick = None;
        }
    }

    pub fn add_note(&mut self, content: &str) {
        if !content.trim().is_empty() {
            self.notes.push(NoteItem {
                id: self.next_note_id,
                content: content.trim().to_string(),
                is_completed: false,
            });
            self.next_note_id += 1;
            self.save_notes();
        }
    }

    pub fn toggle_note(&mut self, id: usize) {
        if let Some(n) = self.notes.iter_mut().find(|n| n.id == id) {
            n.is_completed = !n.is_completed;
            self.save_notes();
        }
    }

    pub fn remove_note(&mut self, id: usize) {
        self.notes.retain(|n| n.id != id);
        self.save_notes();
    }

    pub fn park_file(&mut self, path: PathBuf) {
        if !self.parked_files.contains(&path) {
            self.parked_files.push(path);
        }
    }

    pub fn unpark_file(&mut self, idx: usize) -> Option<PathBuf> {
        if idx < self.parked_files.len() {
            Some(self.parked_files.remove(idx))
        } else {
            None
        }
    }

    fn notes_path() -> PathBuf {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(appdata).join("NotchBar").join("notes.json")
    }

    fn load_notes(&mut self) {
        let path = Self::notes_path();
        if let Ok(data) = fs::read_to_string(path) {
            #[derive(serde::Deserialize)]
            struct StoredNote {
                id: usize,
                content: String,
                is_completed: bool,
            }
            if let Ok(stored) = serde_json::from_str::<Vec<StoredNote>>(&data) {
                self.notes = stored
                    .into_iter()
                    .map(|n| NoteItem {
                        id: n.id,
                        content: n.content,
                        is_completed: n.is_completed,
                    })
                    .collect();
                if let Some(max_id) = self.notes.iter().map(|n| n.id).max() {
                    self.next_note_id = max_id + 1;
                }
            }
        }
    }

    fn save_notes(&self) {
        #[derive(serde::Serialize)]
        struct StoredNote<'a> {
            id: usize,
            content: &'a str,
            is_completed: bool,
        }
        let list: Vec<StoredNote> = self
            .notes
            .iter()
            .map(|n| StoredNote {
                id: n.id,
                content: &n.content,
                is_completed: n.is_completed,
            })
            .collect();
        let path = Self::notes_path();
        let _ = fs::write(path, serde_json::to_string(&list).unwrap_or_default());
    }
}

