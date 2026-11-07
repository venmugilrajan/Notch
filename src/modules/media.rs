use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::time::Duration;
use windows::core::*;
use windows::Foundation::AsyncStatus;
use windows::Media::Control::*;
use windows::Win32::Foundation::*;
use windows::Win32::Media::Audio::*;
use windows::Win32::System::Com::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;

use windows::Storage::Streams::*;

#[derive(Clone, Debug, PartialEq)]
pub struct MediaInfo {
    pub title: String,
    pub artist: String,
    pub is_playing: bool,
    pub has_media: bool,
    pub thumbnail_bytes: Option<Vec<u8>>,
}

impl Default for MediaInfo {
    fn default() -> Self {
        Self {
            title: "No Media Playing".to_string(),
            artist: "".to_string(),
            is_playing: false,
            has_media: false,
            thumbnail_bytes: None,
        }
    }
}

pub struct MediaWatcher {
    info_rx: Receiver<MediaInfo>,
    current: MediaInfo,
}

impl MediaWatcher {
    pub fn new() -> Self {
        let (tx, rx) = channel();

        thread::spawn(move || {
            unsafe {
                let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            }

            loop {
                let info = query_media();
                let _ = tx.send(info);
                thread::sleep(Duration::from_millis(500));
            }
        });

        Self {
            info_rx: rx,
            current: MediaInfo::default(),
        }
    }

    pub fn get_info(&mut self) -> MediaInfo {
        while let Ok(updated) = self.info_rx.try_recv() {
            self.current = updated;
        }
        self.current.clone()
    }

    pub fn toggle_play_pause() {
        simulate_media_key(VK_MEDIA_PLAY_PAUSE);
    }

    pub fn next_track() {
        simulate_media_key(VK_MEDIA_NEXT_TRACK);
    }

    pub fn previous_track() {
        simulate_media_key(VK_MEDIA_PREV_TRACK);
    }
}

fn simulate_media_key(vk: VIRTUAL_KEY) {
    unsafe {
        let mut inputs = [
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: vk,
                        wScan: 0,
                        dwFlags: KEYBD_EVENT_FLAGS(0), // Key down
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            },
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: vk,
                        wScan: 0,
                        dwFlags: KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            },
        ];
        let _ = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
    }
}

fn query_media() -> MediaInfo {
    // 1. Try Windows SMTC (Global System Media Transport Controls)
    if let Some(info) = query_smtc() {
        return info;
    }

    // 2. Fallback to WASAPI active audio stream detection
    query_wasapi_audio()
}

fn wait_for_async<T: RuntimeType + 'static>(
    op: windows::Foundation::IAsyncOperation<T>,
) -> Option<T> {
    for _ in 0..20 {
        if let Ok(status) = op.Status() {
            if status == AsyncStatus::Completed {
                return op.GetResults().ok();
            } else if status == AsyncStatus::Error || status == AsyncStatus::Canceled {
                return None;
            }
        }
        thread::sleep(Duration::from_millis(50));
    }
    None
}

fn wait_for_async_load(
    op: windows::Storage::Streams::DataReaderLoadOperation,
) -> Option<u32> {
    for _ in 0..20 {
        if let Ok(status) = op.Status() {
            if status == AsyncStatus::Completed {
                return op.GetResults().ok();
            } else if status == AsyncStatus::Error || status == AsyncStatus::Canceled {
                return None;
            }
        }
        thread::sleep(Duration::from_millis(50));
    }
    None
}

fn query_smtc() -> Option<MediaInfo> {
    let req = GlobalSystemMediaTransportControlsSessionManager::RequestAsync().ok()?;
    let mgr = wait_for_async(req)?;

    // Gather candidate sessions: prioritize sessions that are currently Playing, especially Spotify
    let mut candidate = None;

    if let Ok(sessions) = mgr.GetSessions() {
        if let Ok(size) = sessions.Size() {
            let mut playing_candidates = Vec::new();
            let mut paused_candidates = Vec::new();

            for i in 0..size {
                if let Ok(s) = sessions.GetAt(i) {
                    let is_playing = s
                        .GetPlaybackInfo()
                        .ok()
                        .and_then(|info| info.PlaybackStatus().ok())
                        .map(|status| status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing)
                        .unwrap_or(false);

                    let app_id = s.SourceAppUserModelId().map(|id| id.to_string()).unwrap_or_default().to_lowercase();
                    let is_music_app = app_id.contains("spotify") || app_id.contains("music") || app_id.contains("apple");

                    if is_playing {
                        if is_music_app {
                            playing_candidates.insert(0, s);
                        } else {
                            playing_candidates.push(s);
                        }
                    } else {
                        if is_music_app {
                            paused_candidates.insert(0, s);
                        } else {
                            paused_candidates.push(s);
                        }
                    }
                }
            }

            if let Some(s) = playing_candidates.into_iter().next() {
                candidate = Some((s, true));
            } else if let Some(s) = paused_candidates.into_iter().next() {
                candidate = Some((s, false));
            }
        }
    }

    // Fallback to GetCurrentSession() if candidate is none
    if candidate.is_none() {
        if let Ok(s) = mgr.GetCurrentSession() {
            let is_playing = s
                .GetPlaybackInfo()
                .ok()
                .and_then(|info| info.PlaybackStatus().ok())
                .map(|status| status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing)
                .unwrap_or(false);
            candidate = Some((s, is_playing));
        }
    }

    let (session, is_playing) = candidate?;
    let props_op = session.TryGetMediaPropertiesAsync().ok()?;
    let props = wait_for_async(props_op)?;

    let title = props.Title().map(|s| s.to_string()).unwrap_or_default();
    let artist = props.Artist().map(|s| s.to_string()).unwrap_or_default();

    if title.is_empty() && artist.is_empty() {
        return None;
    }

    let thumbnail_bytes = if let Ok(thumb_ref) = props.Thumbnail() {
        if let Ok(stream_op) = thumb_ref.OpenReadAsync() {
            if let Some(stream) = wait_for_async(stream_op) {
                if let Ok(size) = stream.Size() {
                    let len = size as usize;
                    if len > 0 && len < 10 * 1024 * 1024 {
                        let mut buf = vec![0u8; len];
                        let reader = DataReader::CreateDataReader(&stream);
                        if let Ok(reader) = reader {
                            if let Ok(load_op) = reader.LoadAsync(size as u32) {
                                if let Some(_) = wait_for_async_load(load_op) {
                                    if reader.ReadBytes(&mut buf).is_ok() {
                                        Some(buf)
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    Some(MediaInfo {
        title,
        artist,
        is_playing,
        has_media: true,
        thumbnail_bytes,
    })
}

fn query_wasapi_audio() -> MediaInfo {
    unsafe {
        let enumerator: Result<IMMDeviceEnumerator> =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL);
        if let Ok(enum_device) = enumerator {
            if let Ok(default_device) =
                enum_device.GetDefaultAudioEndpoint(eRender, eMultimedia)
            {
                let session_mgr: Result<IAudioSessionManager2> =
                    default_device.Activate(CLSCTX_ALL, None);
                if let Ok(mgr) = session_mgr {
                    if let Ok(session_enum) = mgr.GetSessionEnumerator() {
                        if let Ok(count) = session_enum.GetCount() {
                            for i in 0..count {
                                if let Ok(ctrl) = session_enum.GetSession(i) {
                                    if let Ok(ctrl2) = ctrl.cast::<IAudioSessionControl2>() {
                                        if let Ok(state) = ctrl.GetState() {
                                            if state == AudioSessionStateActive {
                                                if let Ok(pid) = ctrl2.GetProcessId() {
                                                    if pid > 0 {
                                                        let proc_name = get_process_name(pid);
                                                        return MediaInfo {
                                                            title: format!("Playing: {}", proc_name),
                                                            artist: "Active Audio Stream".to_string(),
                                                            is_playing: true,
                                                            has_media: true,
                                                            thumbnail_bytes: None,
                                                        };
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    MediaInfo {
        title: "No Media Playing".to_string(),
        artist: "Ready".to_string(),
        is_playing: false,
        has_media: false,
        thumbnail_bytes: None,
    }
}

fn get_process_name(pid: u32) -> String {
    use windows::Win32::System::ProcessStatus::*;
    use windows::Win32::System::Threading::*;

    unsafe {
        if let Ok(process) = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid) {
            let mut name_buf = [0u16; 260];
            let len = GetModuleBaseNameW(process, HMODULE(std::ptr::null_mut()), &mut name_buf);
            let _ = CloseHandle(process);
            if len > 0 {
                return String::from_utf16_lossy(&name_buf[..len as usize]);
            }
        }
    }
    format!("App PID {}", pid)
}
