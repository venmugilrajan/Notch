use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Security::Cryptography::*;
use windows::Win32::System::DataExchange::*;
use windows::Win32::System::Memory::*;
use windows::Win32::UI::Shell::*;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ClipboardItemType {
    Text,
    Link,
    Color,
    Image,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClipboardItem {
    pub id: usize,
    pub item_type: ClipboardItemType,
    pub text: String,
    pub preview: String,
    pub is_pinned: bool,
    pub timestamp_label: String,
    pub date_group: String, // "Today", "Yesterday", "Earlier"
    pub extra_info: String, // e.g. "#FF6A2B" for Color, URL host for Link, character count for Text
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ClipboardFilterTab {
    All,
    Text,
    Images,
    Links,
}

pub struct ClipboardManager {
    pub items: Vec<ClipboardItem>,
    pub max_items: usize,
    pub active_tab: ClipboardFilterTab,
    pub selected_item_id: Option<usize>,
    pub search_query: String,
    pub search_active: bool,
    next_id: usize,
    exclude_format: u32,
    can_include_format: u32,
}

impl ClipboardManager {
    pub fn new(max_items: usize) -> Self {
        let exclude_format = unsafe {
            RegisterClipboardFormatW(w!("ExcludeClipboardContentFromMonitorProcessing"))
        };
        let can_include_format = unsafe {
            RegisterClipboardFormatW(w!("CanIncludeInClipboardHistory"))
        };

        let mut mgr = Self {
            items: Vec::with_capacity(max_items),
            max_items,
            active_tab: ClipboardFilterTab::All,
            selected_item_id: None,
            search_query: String::new(),
            search_active: false,
            next_id: 1,
            exclude_format,
            can_include_format,
        };
        mgr.load_history();
        if let Some(first) = mgr.items.first() {
            mgr.selected_item_id = Some(first.id);
        }
        mgr
    }

    pub fn counts(&self) -> (usize, usize, usize, usize) {
        let total = self.items.len();
        let mut text_cnt = 0;
        let mut img_cnt = 0;
        let mut link_cnt = 0;
        for it in &self.items {
            match it.item_type {
                ClipboardItemType::Text | ClipboardItemType::Color => text_cnt += 1,
                ClipboardItemType::Image => img_cnt += 1,
                ClipboardItemType::Link => link_cnt += 1,
            }
        }
        (total, text_cnt, img_cnt, link_cnt)
    }

    pub fn filtered_items(&self) -> Vec<&ClipboardItem> {
        self.items
            .iter()
            .filter(|it| {
                // Tab filter
                let matches_tab = match self.active_tab {
                    ClipboardFilterTab::All => true,
                    ClipboardFilterTab::Text => matches!(it.item_type, ClipboardItemType::Text | ClipboardItemType::Color),
                    ClipboardFilterTab::Images => matches!(it.item_type, ClipboardItemType::Image),
                    ClipboardFilterTab::Links => matches!(it.item_type, ClipboardItemType::Link),
                };
                if !matches_tab {
                    return false;
                }
                // Search filter
                if !self.search_query.is_empty() {
                    let q = self.search_query.to_lowercase();
                    return it.text.to_lowercase().contains(&q) || it.preview.to_lowercase().contains(&q);
                }
                true
            })
            .collect()
    }

    pub fn on_clipboard_change(&mut self, hwnd: HWND) -> Option<String> {
        unsafe {
            if OpenClipboard(hwnd).is_err() {
                return None;
            }

            // 1. Password manager exclusion checks
            if self.exclude_format != 0 && IsClipboardFormatAvailable(self.exclude_format).is_ok() {
                let _ = CloseClipboard();
                return None;
            }

            // Check for CF_HDROP (format 15) - Files/Images copied from Explorer or apps
            let mut captured_file = None;
            if IsClipboardFormatAvailable(15).is_ok() {
                let handle = GetClipboardData(15);
                if let Ok(hglobal) = handle {
                    let hdrop = windows::Win32::UI::Shell::HDROP(hglobal.0);
                    let count = DragQueryFileW(hdrop, 0xFFFFFFFF, None);
                    if count > 0 {
                        let mut buf = [0u16; 512];
                        let len = DragQueryFileW(hdrop, 0, Some(&mut buf));
                        if len > 0 {
                            let path_str = String::from_utf16_lossy(&buf[..len as usize]);
                            captured_file = Some((path_str, count));
                        }
                    }
                }
            }

            // Check if CF_UNICODETEXT is available (format 13)
            let mut captured_text = None;
            if captured_file.is_none() && IsClipboardFormatAvailable(13).is_ok() {
                let handle = GetClipboardData(13);
                if let Ok(hglobal) = handle {
                    let ptr = GlobalLock(HGLOBAL(hglobal.0));
                    if !ptr.is_null() {
                        let wide_slice = std::slice::from_raw_parts(ptr as *const u16, 100_000);
                        let mut len = 0;
                        while len < wide_slice.len() && wide_slice[len] != 0 {
                            len += 1;
                        }
                        let text = String::from_utf16_lossy(&wide_slice[..len]);
                        let _ = GlobalUnlock(HGLOBAL(hglobal.0));
                        if !text.trim().is_empty() {
                            captured_text = Some(text);
                        }
                    }
                }
            }

            // Check if CF_DIB (format 8) is available - Direct screenshot / copied bitmap
            let mut captured_bitmap = false;
            if captured_file.is_none() && captured_text.is_none() && IsClipboardFormatAvailable(8).is_ok() {
                captured_bitmap = true;
            }

            let _ = CloseClipboard();

            if let Some((filepath, count)) = captured_file {
                let path = std::path::Path::new(&filepath);
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                let is_img = matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "bmp" | "webp" | "gif" | "ico");
                let item_type = if is_img { ClipboardItemType::Image } else { ClipboardItemType::Text };
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or(&filepath);
                
                let preview = if count > 1 {
                    format!("{} (+{} other files)", file_name, count - 1)
                } else {
                    file_name.to_string()
                };

                let extra_info = if is_img {
                    format!("Image · {}", ext.to_uppercase())
                } else {
                    format!("{} files · Path", count)
                };

                if let Some(first) = self.items.first() {
                    if first.text == filepath {
                        return None;
                    }
                }

                let item = ClipboardItem {
                    id: self.next_id,
                    item_type,
                    text: filepath.clone(),
                    preview,
                    is_pinned: false,
                    timestamp_label: "Just now".to_string(),
                    date_group: "Today".to_string(),
                    extra_info,
                };
                self.next_id += 1;
                self.selected_item_id = Some(item.id);
                self.items.insert(0, item);
                if self.items.len() > self.max_items {
                    if let Some(idx) = self.items.iter().rposition(|it| !it.is_pinned) {
                        self.items.remove(idx);
                    }
                }
                self.save_history();
                return Some(filepath);
            }

            if captured_bitmap {
                let text = format!("[Copied Bitmap Image #{}]", self.next_id);
                let item = ClipboardItem {
                    id: self.next_id,
                    item_type: ClipboardItemType::Image,
                    text: text.clone(),
                    preview: "Screenshot / Copied Bitmap".to_string(),
                    is_pinned: false,
                    timestamp_label: "Just now".to_string(),
                    date_group: "Today".to_string(),
                    extra_info: "Bitmap Image".to_string(),
                };
                self.next_id += 1;
                self.selected_item_id = Some(item.id);
                self.items.insert(0, item);
                if self.items.len() > self.max_items {
                    if let Some(idx) = self.items.iter().rposition(|it| !it.is_pinned) {
                        self.items.remove(idx);
                    }
                }
                self.save_history();
                return Some(text);
            }

            if let Some(text) = captured_text {
                // Avoid consecutive duplicates
                if let Some(first) = self.items.first() {
                    if first.text == text {
                        return None;
                    }
                }

                let trimmed = text.trim();
                let item_type = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
                    ClipboardItemType::Link
                } else if is_hex_color(trimmed) {
                    ClipboardItemType::Color
                } else {
                    ClipboardItemType::Text
                };

                let extra_info = match item_type {
                    ClipboardItemType::Link => {
                        trimmed.split("://").nth(1).unwrap_or(trimmed).split('/').next().unwrap_or(trimmed).to_string()
                    }
                    ClipboardItemType::Color => trimmed.to_string(),
                    ClipboardItemType::Text => format!("{} chars", text.len()),
                    ClipboardItemType::Image => "Bitmap".to_string(),
                };

                let preview = if text.len() > 60 {
                    format!("{}...", &text[..text.char_indices().map(|(i, _)| i).nth(60).unwrap_or(text.len())])
                } else {
                    text.clone()
                };

                let item = ClipboardItem {
                    id: self.next_id,
                    item_type,
                    text: text.clone(),
                    preview: preview.replace('\n', " ").replace('\r', ""),
                    is_pinned: false,
                    timestamp_label: "Just now".to_string(),
                    date_group: "Today".to_string(),
                    extra_info,
                };
                self.next_id += 1;

                self.selected_item_id = Some(item.id);
                self.items.insert(0, item);
                if self.items.len() > self.max_items {
                    if let Some(idx) = self.items.iter().rposition(|it| !it.is_pinned) {
                        self.items.remove(idx);
                    }
                }
                self.save_history();
                return Some(text);
            }
        }
        None
    }

    pub fn copy_to_clipboard(&self, text: &str) -> bool {
        unsafe {
            if OpenClipboard(HWND(std::ptr::null_mut())).is_err() {
                return false;
            }
            let _ = EmptyClipboard();

            let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
            let bytes = wide.len() * 2;
            let hglobal = GlobalAlloc(GMEM_MOVEABLE, bytes);
            if let Ok(hg) = hglobal {
                let ptr = GlobalLock(hg);
                if !ptr.is_null() {
                    std::ptr::copy_nonoverlapping(wide.as_ptr() as *const u8, ptr as *mut u8, bytes);
                    let _ = GlobalUnlock(hg);
                    let _ = SetClipboardData(13, HANDLE(hg.0));
                }
            }
            let _ = CloseClipboard();
            true
        }
    }

    pub fn toggle_pin(&mut self, id: usize) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.is_pinned = !item.is_pinned;
            self.save_history();
        }
    }

    pub fn delete_item(&mut self, id: usize) {
        self.items.retain(|i| i.id != id);
        if self.selected_item_id == Some(id) {
            self.selected_item_id = self.items.first().map(|i| i.id);
        }
        self.save_history();
    }

    pub fn clear_all(&mut self) {
        self.items.retain(|i| i.is_pinned);
        self.selected_item_id = self.items.first().map(|i| i.id);
        self.save_history();
    }

    pub fn save_to_file(&self, id: usize) -> Option<PathBuf> {
        let item = self.items.iter().find(|i| i.id == id)?;
        let appdata = std::env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string());
        let desktop = PathBuf::from(appdata).join("Desktop");
        let filename = format!("NotchBar_Clip_{}.txt", item.id);
        let out_path = desktop.join(filename);
        let _ = fs::write(&out_path, &item.text);
        Some(out_path)
    }

    fn storage_path() -> PathBuf {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(appdata).join("NotchBar").join("clipboard.dpapi")
    }

    fn load_history(&mut self) {
        let path = Self::storage_path();
        if let Ok(encrypted_data) = fs::read(&path) {
            if let Some(decrypted_bytes) = dpapi_unprotect(&encrypted_data) {
                if let Ok(items) = serde_json::from_slice::<Vec<ClipboardItem>>(&decrypted_bytes) {
                    self.items = items;
                    if let Some(max_id) = self.items.iter().map(|i| i.id).max() {
                        self.next_id = max_id + 1;
                    }
                }
            }
        }
    }

    fn save_history(&self) {
        if let Ok(json_bytes) = serde_json::to_vec(&self.items) {
            if let Some(encrypted) = dpapi_protect(&json_bytes) {
                let path = Self::storage_path();
                let _ = fs::write(path, encrypted);
            }
        }
    }
}

fn is_hex_color(s: &str) -> bool {
    let s = s.trim();
    if !(s.starts_with('#') && (s.len() == 7 || s.len() == 9 || s.len() == 4)) {
        return false;
    }
    s[1..].chars().all(|c| c.is_ascii_hexdigit())
}

// Windows DPAPI Encryption at rest
fn dpapi_protect(data: &[u8]) -> Option<Vec<u8>> {
    unsafe {
        let data_in = CRYPT_INTEGER_BLOB {
            cbData: data.len() as u32,
            pbData: data.as_ptr() as *mut u8,
        };
        let mut data_out = CRYPT_INTEGER_BLOB::default();

        if CryptProtectData(
            &data_in,
            PCWSTR::null(),
            None,
            None,
            None,
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut data_out,
        ).is_ok() {
            let slice = std::slice::from_raw_parts(data_out.pbData, data_out.cbData as usize);
            let result = slice.to_vec();
            let _ = LocalFree(HLOCAL(data_out.pbData as *mut _));
            Some(result)
        } else {
            None
        }
    }
}

fn dpapi_unprotect(data: &[u8]) -> Option<Vec<u8>> {
    unsafe {
        let data_in = CRYPT_INTEGER_BLOB {
            cbData: data.len() as u32,
            pbData: data.as_ptr() as *mut u8,
        };
        let mut data_out = CRYPT_INTEGER_BLOB::default();

        if CryptUnprotectData(
            &data_in,
            None,
            None,
            None,
            None,
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut data_out,
        ).is_ok() {
            let slice = std::slice::from_raw_parts(data_out.pbData, data_out.cbData as usize);
            let result = slice.to_vec();
            let _ = LocalFree(HLOCAL(data_out.pbData as *mut _));
            Some(result)
        } else {
            None
        }
    }
}
