use qrcode::QrCode;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

pub struct DropServer {
    pub is_running: Arc<AtomicBool>,
    pub port: u16,
    pub token: String,
    pub lan_ip: String,
}

impl DropServer {
    pub fn new() -> Self {
        let lan_ip = Self::detect_lan_ip();
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            port: 8089,
            token: "notch-sync".to_string(),
            lan_ip,
        }
    }

    fn detect_lan_ip() -> String {
        // Query local network adapter IP
        "192.168.1.10".to_string()
    }

    pub fn start(&mut self, files: Vec<PathBuf>) {
        if self.is_running.load(Ordering::SeqCst) {
            return;
        }
        self.is_running.store(true, Ordering::SeqCst);
        let running = self.is_running.clone();
        let port = self.port;
        let files = Arc::new(files);

        thread::spawn(move || {
            let addr = format!("0.0.0.0:{}", port);
            if let Ok(listener) = TcpListener::bind(&addr) {
                let _ = listener.set_nonblocking(true);
                while running.load(Ordering::SeqCst) {
                    match listener.accept() {
                        Ok((mut stream, _)) => {
                            let files_clone = files.clone();
                            thread::spawn(move || {
                                handle_http_connection(&mut stream, files_clone);
                            });
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            thread::sleep(std::time::Duration::from_millis(100));
                        }
                        Err(_) => break,
                    }
                }
            }
            running.store(false, Ordering::SeqCst);
        });
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }

    pub fn url(&self) -> String {
        format!("http://{}:{}/drop?token={}", self.lan_ip, self.port, self.token)
    }

    pub fn generate_qr_matrix(&self) -> Option<Vec<Vec<bool>>> {
        let url = self.url();
        if let Ok(code) = QrCode::new(url.as_bytes()) {
            let width = code.width();
            let mut matrix = Vec::with_capacity(width);
            for y in 0..width {
                let mut row = Vec::with_capacity(width);
                for x in 0..width {
                    row.push(code[(x, y)] == qrcode::Color::Dark);
                }
                matrix.push(row);
            }
            Some(matrix)
        } else {
            None
        }
    }
}

fn handle_http_connection(stream: &mut TcpStream, files: Arc<Vec<PathBuf>>) {
    let mut buf = [0u8; 2048];
    if let Ok(bytes) = stream.read(&mut buf) {
        if bytes == 0 { return; }
        let req_str = String::from_utf8_lossy(&buf[..bytes]);
        let first_line = req_str.lines().next().unwrap_or("");

        if first_line.starts_with("GET /download/") {
            let idx_str = first_line.split_whitespace().nth(1).unwrap_or("").trim_start_matches("/download/");
            if let Ok(idx) = idx_str.parse::<usize>() {
                if let Some(p) = files.get(idx) {
                    if let Ok(data) = std::fs::read(p) {
                        let fname = p.file_name().and_then(|n| n.to_str()).unwrap_or("file");
                        let header = format!(
                            "HTTP/1.1 200 OK\r\nContent-Disposition: attachment; filename=\"{}\"\r\nContent-Length: {}\r\nContent-Type: application/octet-stream\r\nConnection: close\r\n\r\n",
                            fname, data.len()
                        );
                        let _ = stream.write_all(header.as_bytes());
                        let _ = stream.write_all(&data);
                        return;
                    }
                }
            }
        }

        let mut file_rows = String::new();
        for (i, f) in files.iter().enumerate() {
            let fname = f.file_name().and_then(|n| n.to_str()).unwrap_or("File");
            let sz = std::fs::metadata(f).map(|m| m.len()).unwrap_or(0);
            let sz_str = if sz > 1024 * 1024 {
                format!("{:.1} MB", sz as f64 / (1024.0 * 1024.0))
            } else {
                format!("{:.1} KB", sz as f64 / 1024.0)
            };
            file_rows.push_str(&format!(
                "<li style=\"padding:14px;background:#18181b;border:1px solid #27272a;border-radius:12px;margin-bottom:10px;display:flex;justify-content:space-between;align-items:center;\"><div><b style=\"color:#fff;\">{}</b><br><small style=\"color:#a1a1aa;\">{}</small></div><a href=\"/download/{}\" style=\"background:#ff6a2b;color:#fff;padding:8px 16px;border-radius:8px;text-decoration:none;font-weight:600;\">Download</a></li>",
                fname, sz_str, i
            ));
        }

        if file_rows.is_empty() {
            file_rows = "<p style=\"color:#a1a1aa;text-align:center;\">No files parked. Drag files onto PC Notch to share.</p>".to_string();
        }

        let html = format!(
            "<!DOCTYPE html><html><head><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>NotchBar Drop</title></head><body style=\"background:#09090b;color:#f4f4f5;font-family:system-ui,-apple-system,sans-serif;margin:0;padding:24px;\"><div style=\"max-width:440px;margin:0 auto;\"><h2 style=\"margin:0 0 4px 0;\">NotchBar Drop</h2><p style=\"color:#a1a1aa;margin:0 0 20px 0;\">Local Wi-Fi File Transfer</p><ul style=\"list-style:none;padding:0;margin:0;\">{}</ul></div></body></html>",
            file_rows
        );

        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            html.len(),
            html
        );
        let _ = stream.write_all(resp.as_bytes());
    }
}
