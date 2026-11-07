use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};
use windows::core::*;
use windows::Win32::Networking::WinHttp::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WeatherData {
    pub temperature: f32,
    pub condition: String,
    pub high: f32,
    pub low: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CurrencyRates {
    pub eur: f32,
    pub gbp: f32,
    pub jpy: f32,
    pub inr: f32,
    pub cad: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: usize,
    pub title: String,
    pub time_str: String,
    pub is_reminder: bool,
}

#[derive(Clone, Debug)]
pub struct WorldClock {
    pub name: &'static str,
    pub offset_hours: i32,
    pub offset_mins: i32,
}

pub struct UtilityManager {
    pub weather: Option<WeatherData>,
    pub currency: CurrencyRates,
    pub events: Vec<CalendarEvent>,
    pub world_clocks: Vec<WorldClock>,
    last_weather_fetch: Option<Instant>,
    weather_rx: Receiver<WeatherData>,
    weather_tx: Sender<WeatherData>,
    is_fetching_weather: bool,
}

impl UtilityManager {
    pub fn new() -> Self {
        let (tx, rx) = channel();

        let mut mgr = Self {
            weather: Some(WeatherData {
                temperature: 21.5,
                condition: "Partly Cloudy".to_string(),
                high: 25.0,
                low: 17.0,
            }),
            currency: CurrencyRates {
                eur: 0.92,
                gbp: 0.79,
                jpy: 154.5,
                inr: 84.3,
                cad: 1.38,
            },
            events: vec![
                CalendarEvent {
                    id: 1,
                    title: "Architecture Review".to_string(),
                    time_str: "10:00 AM".to_string(),
                    is_reminder: false,
                },
                CalendarEvent {
                    id: 2,
                    title: "Team Standup".to_string(),
                    time_str: "02:30 PM".to_string(),
                    is_reminder: true,
                },
                CalendarEvent {
                    id: 3,
                    title: "Release Deployment".to_string(),
                    time_str: "05:00 PM".to_string(),
                    is_reminder: false,
                },
            ],
            world_clocks: vec![
                WorldClock { name: "New York (EST)", offset_hours: -5, offset_mins: 0 },
                WorldClock { name: "London (GMT)", offset_hours: 0, offset_mins: 0 },
                WorldClock { name: "Tokyo (JST)", offset_hours: 9, offset_mins: 0 },
                WorldClock { name: "New Delhi (IST)", offset_hours: 5, offset_mins: 30 },
            ],
            last_weather_fetch: None,
            weather_rx: rx,
            weather_tx: tx,
            is_fetching_weather: false,
        };
        mgr.load_cached_weather();
        mgr
    }

    pub fn check_async_updates(&mut self) -> bool {
        let mut updated = false;
        while let Ok(data) = self.weather_rx.try_recv() {
            self.weather = Some(data.clone());
            self.last_weather_fetch = Some(Instant::now());
            self.is_fetching_weather = false;
            self.save_cached_weather(&data);
            updated = true;
        }
        updated
    }

    pub fn refresh_weather_if_needed(&mut self) {
        let needs_refresh = match self.last_weather_fetch {
            Some(t) => t.elapsed() > Duration::from_secs(1800), // 30 min cache
            None => true,
        };

        if needs_refresh && !self.is_fetching_weather {
            self.is_fetching_weather = true;
            let tx = self.weather_tx.clone();

            thread::spawn(move || {
                if let Some(w) = fetch_open_meteo_weather() {
                    let _ = tx.send(w);
                }
            });
        }
    }

    pub fn convert_length_m_to_ft(meters: f32) -> f32 {
        meters * 3.28084
    }

    pub fn convert_weight_kg_to_lbs(kg: f32) -> f32 {
        kg * 2.20462
    }

    pub fn convert_temperature_c_to_f(celsius: f32) -> f32 {
        (celsius * 9.0 / 5.0) + 32.0
    }

    pub fn convert_usd_to(&self, usd: f32, target: &str) -> f32 {
        match target {
            "EUR" => usd * self.currency.eur,
            "GBP" => usd * self.currency.gbp,
            "JPY" => usd * self.currency.jpy,
            "INR" => usd * self.currency.inr,
            "CAD" => usd * self.currency.cad,
            _ => usd,
        }
    }

    fn storage_path() -> PathBuf {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(appdata).join("NotchBar").join("weather.json")
    }

    fn load_cached_weather(&mut self) {
        let path = Self::storage_path();
        if let Ok(data) = fs::read_to_string(path) {
            if let Ok(w) = serde_json::from_str::<WeatherData>(&data) {
                self.weather = Some(w);
            }
        }
    }

    fn save_cached_weather(&self, data: &WeatherData) {
        let path = Self::storage_path();
        let _ = fs::write(path, serde_json::to_string(data).unwrap_or_default());
    }
}

// Native WinHttp asynchronous fetch for Open-Meteo
fn fetch_open_meteo_weather() -> Option<WeatherData> {
    unsafe {
        let session = WinHttpOpen(
            w!("NotchBar/1.0"),
            WINHTTP_ACCESS_TYPE_DEFAULT_PROXY,
            PCWSTR::null(),
            PCWSTR::null(),
            0,
        );
        if session.is_null() {
            return None;
        }

        let connect = WinHttpConnect(
            session,
            w!("api.open-meteo.com"),
            INTERNET_DEFAULT_HTTPS_PORT,
            0,
        );
        if connect.is_null() {
            let _ = WinHttpCloseHandle(session);
            return None;
        }

        let request = WinHttpOpenRequest(
            connect,
            w!("GET"),
            w!("/v1/forecast?latitude=40.7128&longitude=-74.0060&current=temperature_2m,weather_code&daily=temperature_2m_max,temperature_2m_min&timezone=auto"),
            PCWSTR::null(),
            PCWSTR::null(),
            std::ptr::null(),
            WINHTTP_OPEN_REQUEST_FLAGS(WINHTTP_FLAG_SECURE.0),
        );

        if request.is_null() {
            let _ = WinHttpCloseHandle(connect);
            let _ = WinHttpCloseHandle(session);
            return None;
        }

        let sent = WinHttpSendRequest(
            request,
            None,
            None,
            0,
            0,
            0,
        );

        if sent.is_ok() {
            if WinHttpReceiveResponse(request, std::ptr::null_mut()).is_ok() {
                let mut buffer = Vec::new();
                let mut bytes_to_read = 0u32;

                while WinHttpQueryDataAvailable(request, &mut bytes_to_read).is_ok() && bytes_to_read > 0 {
                    let mut chunk = vec![0u8; bytes_to_read as usize];
                    let mut bytes_read = 0u32;
                    if WinHttpReadData(request, chunk.as_mut_ptr() as *mut _, bytes_to_read, &mut bytes_read).is_ok() {
                        chunk.truncate(bytes_read as usize);
                        buffer.extend_from_slice(&chunk);
                    } else {
                        break;
                    }
                }

                let _ = WinHttpCloseHandle(request);
                let _ = WinHttpCloseHandle(connect);
                let _ = WinHttpCloseHandle(session);

                if let Ok(json_str) = String::from_utf8(buffer) {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_str) {
                        let temp = parsed["current"]["temperature_2m"].as_f64().unwrap_or(21.0) as f32;
                        let max_t = parsed["daily"]["temperature_2m_max"][0].as_f64().unwrap_or(25.0) as f32;
                        let min_t = parsed["daily"]["temperature_2m_min"][0].as_f64().unwrap_or(16.0) as f32;
                        let w_code = parsed["current"]["weather_code"].as_i64().unwrap_or(0);

                        let cond = match w_code {
                            0 => "Clear Sky",
                            1..=3 => "Partly Cloudy",
                            45 | 48 => "Foggy",
                            51..=67 => "Rain",
                            71..=77 => "Snow",
                            80..=82 => "Showers",
                            _ => "Overcast",
                        };

                        return Some(WeatherData {
                            temperature: temp,
                            condition: cond.to_string(),
                            high: max_t,
                            low: min_t,
                        });
                    }
                }
            }
        }

        let _ = WinHttpCloseHandle(request);
        let _ = WinHttpCloseHandle(connect);
        let _ = WinHttpCloseHandle(session);
        None
    }
}
