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

pub struct ProductivityManager {
    pub timers: Vec<TimerItem>,
    pub parked_files: Vec<PathBuf>,
    pub next_timer_id: usize,
}

impl ProductivityManager {
    pub fn new() -> Self {
        Self {
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
            parked_files: Vec::new(),
            next_timer_id: 3,
        }
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
}
