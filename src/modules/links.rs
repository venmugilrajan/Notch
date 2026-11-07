use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuickLink {
    pub id: usize,
    pub title: String,
    pub url: String,
    pub desc: String,
}

#[derive(Clone, Debug)]
pub struct LinksManager {
    pub selected_engine_index: usize,
    pub links: Vec<QuickLink>,
    pub next_id: usize,
}

impl LinksManager {
    pub fn new() -> Self {
        let mut mgr = Self {
            selected_engine_index: 0,
            links: Vec::new(),
            next_id: 1,
        };
        mgr.load_links();
        mgr
    }

    fn links_file_path() -> PathBuf {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(appdata).join("NotchBar").join("links.json")
    }

    pub fn load_links(&mut self) {
        let path = Self::links_file_path();
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(loaded) = serde_json::from_str::<Vec<QuickLink>>(&data) {
                if !loaded.is_empty() {
                    self.next_id = loaded.iter().map(|l| l.id).max().unwrap_or(0) + 1;
                    self.links = loaded;
                    return;
                }
            }
        }
        // Defaults
        self.links = vec![
            QuickLink { id: 1, title: "GitHub".to_string(), url: "https://github.com".to_string(), desc: "Code & PRs".to_string() },
            QuickLink { id: 2, title: "YouTube".to_string(), url: "https://youtube.com".to_string(), desc: "Videos & Music".to_string() },
            QuickLink { id: 3, title: "Google".to_string(), url: "https://google.com".to_string(), desc: "Search & Web".to_string() },
            QuickLink { id: 4, title: "ChatGPT".to_string(), url: "https://chatgpt.com".to_string(), desc: "AI Chat".to_string() },
        ];
        self.next_id = 5;
        self.save_links();
    }

    pub fn save_links(&self) {
        let path = Self::links_file_path();
        if let Ok(json) = serde_json::to_string_pretty(&self.links) {
            let _ = fs::create_dir_all(path.parent().unwrap());
            let _ = fs::write(path, json);
        }
    }

    pub fn add_link(&mut self, title: String, url: String, desc: String) {
        self.links.push(QuickLink {
            id: self.next_id,
            title,
            url,
            desc,
        });
        self.next_id += 1;
        self.save_links();
    }

    pub fn remove_link(&mut self, id: usize) {
        self.links.retain(|l| l.id != id);
        self.save_links();
    }
}

