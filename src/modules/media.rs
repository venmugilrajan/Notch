use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Duration;
use windows::core::*;
use windows::Foundation::{AsyncStatus, TypedEventHandler};
use windows::Media::Control::*;
use windows::Storage::Streams::*;
use windows::Win32::Foundation::*;
use windows::Win32::Media::Audio::*;
use windows::Win32::System::Com::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;

#[derive(Clone, Debug, PartialEq)]
pub struct MediaInfo {
    pub title: String,
    pub artist: String,
    pub is_playing: bool,
    pub has_media: bool,
    pub thumbnail_bytes: Option<Vec<u8>>,
    pub position_secs: u32,
    pub duration_secs: u32,
    pub source_app: String, // e.g. "Spotify", "Chrome", "Apple Music", "Edge"
}

impl Default for MediaInfo {
    fn default() -> Self {
        Self {
            title: "No Media Playing".to_string(),
            artist: "".to_string(),
            is_playing: false,
            has_media: false,
            thumbnail_bytes: None,
            position_secs: 0,
            duration_secs: 0,
            source_app: String::new(),
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

            run_smtc_event_loop(tx);
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
        thread::spawn(|| {
            unsafe { let _ = CoInitializeEx(None, COINIT_MULTITHREADED); }
            if !control_via_smtc(MediaCommand::TogglePlayPause) {
                simulate_media_key(VK_MEDIA_PLAY_PAUSE);
            }
        });
    }

    pub fn next_track() {
        thread::spawn(|| {
            unsafe { let _ = CoInitializeEx(None, COINIT_MULTITHREADED); }
            if !control_via_smtc(MediaCommand::Next) {
                simulate_media_key(VK_MEDIA_NEXT_TRACK);
            }
        });
    }

    pub fn previous_track() {
        thread::spawn(|| {
            unsafe { let _ = CoInitializeEx(None, COINIT_MULTITHREADED); }
            if !control_via_smtc(MediaCommand::Previous) {
                simulate_media_key(VK_MEDIA_PREV_TRACK);
            }
        });
    }

    pub fn seek_to_ratio(ratio: f32) {
        let r = ratio.clamp(0.0, 1.0);
        thread::spawn(move || {
            unsafe { let _ = CoInitializeEx(None, COINIT_MULTITHREADED); }
            let Ok(req) = GlobalSystemMediaTransportControlsSessionManager::RequestAsync() else { return; };
            let Some(mgr) = wait_for_async(req) else { return; };

            let session = get_best_session(&mgr);
            if let Some(s) = session {
                if let Ok(timeline) = s.GetTimelineProperties() {
                    if let Ok(end) = timeline.EndTime() {
                        let target_ticks = (end.Duration as f64 * r as f64) as i64;
                        if let Ok(op) = s.TryChangePlaybackPositionAsync(target_ticks) {
                            let _ = wait_for_async(op);
                        }
                    }
                }
            }
        });
    }
}

fn run_smtc_event_loop(tx: Sender<MediaInfo>) {
    let (wake_tx, wake_rx) = channel::<()>();

    let Ok(req) = GlobalSystemMediaTransportControlsSessionManager::RequestAsync() else {
        loop {
            let wasapi_info = query_wasapi_audio();
            let _ = tx.send(wasapi_info);
            thread::sleep(Duration::from_millis(1000));
        }
    };
    let Some(mgr) = wait_for_async(req) else {
        loop {
            let wasapi_info = query_wasapi_audio();
            let _ = tx.send(wasapi_info);
            thread::sleep(Duration::from_millis(1000));
        }
    };

    // Manager-level event subscriptions
    let w1 = wake_tx.clone();
    let _ = mgr.CurrentSessionChanged(&TypedEventHandler::new(move |_sender, _args| {
        let _ = w1.send(());
        Ok(())
    }));

    let w2 = wake_tx.clone();
    let _ = mgr.SessionsChanged(&TypedEventHandler::new(move |_sender, _args| {
        let _ = w2.send(());
        Ok(())
    }));

    let mut current_session_token: Option<String> = None;
    let mut cache: HashMap<String, Option<Vec<u8>>> = HashMap::new();
    let mut last_playing;

    loop {
        let session = get_best_session(&mgr);

        // If active session changed, register listeners on new session
        let session_id = session.as_ref().and_then(|s| s.SourceAppUserModelId().ok().map(|id| id.to_string()));
        if session_id != current_session_token {
            current_session_token = session_id;
            if let Some(ref s) = session {
                let w_prop = wake_tx.clone();
                let _ = s.MediaPropertiesChanged(&TypedEventHandler::new(move |_s, _a| {
                    let _ = w_prop.send(());
                    Ok(())
                }));

                let w_pb = wake_tx.clone();
                let _ = s.PlaybackInfoChanged(&TypedEventHandler::new(move |_s, _a| {
                    let _ = w_pb.send(());
                    Ok(())
                }));

                let w_tl = wake_tx.clone();
                let _ = s.TimelinePropertiesChanged(&TypedEventHandler::new(move |_s, _a| {
                    let _ = w_tl.send(());
                    Ok(())
                }));
            }
        }

        if let Some(ref s) = session {
            if let Some(info) = extract_media_info(s, &mut cache) {
                last_playing = info.is_playing;
                let _ = tx.send(info);
            } else {
                let wasapi_info = query_wasapi_audio();
                last_playing = wasapi_info.is_playing;
                let _ = tx.send(wasapi_info);
            }
        } else {
            let wasapi_info = query_wasapi_audio();
            last_playing = wasapi_info.is_playing;
            let _ = tx.send(wasapi_info);
        }

        // Wait for next event; tick every 1000ms if playing to advance scrubber smoothly
        if last_playing {
            let _ = wake_rx.recv_timeout(Duration::from_millis(1000));
        } else {
            let _ = wake_rx.recv();
        }
    }
}

fn get_best_session(mgr: &GlobalSystemMediaTransportControlsSessionManager) -> Option<GlobalSystemMediaTransportControlsSession> {
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
                    let is_music_app = app_id.contains("spotify")
                        || app_id.contains("music")
                        || app_id.contains("apple")
                        || app_id.contains("chrome")
                        || app_id.contains("edge")
                        || app_id.contains("firefox")
                        || app_id.contains("vlc");

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
                return Some(s);
            }
            if let Some(s) = paused_candidates.into_iter().next() {
                return Some(s);
            }
        }
    }

    mgr.GetCurrentSession().ok()
}

fn extract_media_info(
    session: &GlobalSystemMediaTransportControlsSession,
    cache: &mut HashMap<String, Option<Vec<u8>>>,
) -> Option<MediaInfo> {
    let props_op = session.TryGetMediaPropertiesAsync().ok()?;
    let props = wait_for_async(props_op)?;

    let title = props.Title().map(|s| s.to_string()).unwrap_or_default();
    let artist = props.Artist().map(|s| s.to_string()).unwrap_or_default();

    if title.is_empty() && artist.is_empty() {
        return None;
    }

    let is_playing = session
        .GetPlaybackInfo()
        .ok()
        .and_then(|info| info.PlaybackStatus().ok())
        .map(|status| status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing)
        .unwrap_or(false);

    let (mut position_secs, mut duration_secs) = (0u32, 0u32);
    if let Ok(timeline) = session.GetTimelineProperties() {
        if let Ok(pos) = timeline.Position() {
            position_secs = (pos.Duration / 10_000_000).max(0) as u32;
        }
        if let Ok(end) = timeline.EndTime() {
            duration_secs = (end.Duration / 10_000_000).max(0) as u32;
        }
    }

    let source_app = session
        .SourceAppUserModelId()
        .map(|id| {
            let s = id.to_string().to_lowercase();
            if s.contains("spotify") {
                "Spotify".to_string()
            } else if s.contains("chrome") {
                "Chrome".to_string()
            } else if s.contains("apple") {
                "Apple Music".to_string()
            } else if s.contains("edge") {
                "Edge".to_string()
            } else if s.contains("firefox") {
                "Firefox".to_string()
            } else if s.contains("vlc") {
                "VLC".to_string()
            } else {
                "Player".to_string()
            }
        })
        .unwrap_or_else(|_| "Player".to_string());

    let cache_key = format!("{}::{}", title, artist);
    let thumbnail_bytes = if let Some(cached) = cache.get(&cache_key) {
        cached.clone()
    } else {
        let fetched = fetch_thumbnail(&props);
        if cache.len() > 30 {
            cache.clear();
        }
        cache.insert(cache_key, fetched.clone());
        fetched
    };

    Some(MediaInfo {
        title,
        artist,
        is_playing,
        has_media: true,
        thumbnail_bytes,
        position_secs,
        duration_secs,
        source_app,
    })
}

fn fetch_thumbnail(props: &GlobalSystemMediaTransportControlsSessionMediaProperties) -> Option<Vec<u8>> {
    let thumb_ref = props.Thumbnail().ok()?;
    let stream_op = thumb_ref.OpenReadAsync().ok()?;
    let stream = wait_for_async(stream_op)?;
    let size = stream.Size().ok()? as usize;
    if size == 0 || size > 10 * 1024 * 1024 {
        return None;
    }

    let mut buf = vec![0u8; size];
    let reader = DataReader::CreateDataReader(&stream).ok()?;
    let load_op = reader.LoadAsync(size as u32).ok()?;
    let _ = wait_for_async_load(load_op)?;
    if reader.ReadBytes(&mut buf).is_ok() {
        Some(buf)
    } else {
        None
    }
}

#[derive(Clone, Copy)]
enum MediaCommand {
    TogglePlayPause,
    Next,
    Previous,
}

fn control_via_smtc(cmd: MediaCommand) -> bool {
    let Ok(req) = GlobalSystemMediaTransportControlsSessionManager::RequestAsync() else { return false; };
    let Some(mgr) = wait_for_async(req) else { return false; };

    let session = get_best_session(&mgr);
    let Some(s) = session else { return false; };

    match cmd {
        MediaCommand::TogglePlayPause => {
            if let Ok(op) = s.TryTogglePlayPauseAsync() {
                if let Some(true) = wait_for_async(op) {
                    return true;
                }
            }
        }
        MediaCommand::Next => {
            if let Ok(op) = s.TrySkipNextAsync() {
                if let Some(true) = wait_for_async(op) {
                    return true;
                }
            }
        }
        MediaCommand::Previous => {
            if let Ok(op) = s.TrySkipPreviousAsync() {
                if let Some(true) = wait_for_async(op) {
                    return true;
                }
            }
        }
    }
    false
}

fn simulate_media_key(vk: VIRTUAL_KEY) {
    unsafe {
        let inputs = [
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: vk,
                        wScan: 0,
                        dwFlags: KEYBD_EVENT_FLAGS(0),
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

fn wait_for_async<T: RuntimeType + 'static>(
    op: windows::Foundation::IAsyncOperation<T>,
) -> Option<T> {
    for _ in 0..50 {
        if let Ok(status) = op.Status() {
            if status == AsyncStatus::Completed {
                return op.GetResults().ok();
            } else if status == AsyncStatus::Error || status == AsyncStatus::Canceled {
                return None;
            }
        }
        thread::sleep(Duration::from_millis(20));
    }
    None
}

fn wait_for_async_load(
    op: windows::Storage::Streams::DataReaderLoadOperation,
) -> Option<u32> {
    for _ in 0..50 {
        if let Ok(status) = op.Status() {
            if status == AsyncStatus::Completed {
                return op.GetResults().ok();
            } else if status == AsyncStatus::Error || status == AsyncStatus::Canceled {
                return None;
            }
        }
        thread::sleep(Duration::from_millis(20));
    }
    None
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
                                                    let proc_name = get_process_name_by_id(pid);
                                                    let is_music_app = proc_name.contains("spotify")
                                                        || proc_name.contains("chrome")
                                                        || proc_name.contains("msedge")
                                                        || proc_name.contains("music")
                                                        || proc_name.contains("vlc");
                                                    if is_music_app {
                                                        let display_name = if proc_name.contains("spotify") {
                                                            "Spotify".to_string()
                                                        } else if proc_name.contains("chrome") {
                                                            "Chrome".to_string()
                                                        } else if proc_name.contains("msedge") {
                                                            "Edge".to_string()
                                                        } else if proc_name.contains("vlc") {
                                                            "VLC".to_string()
                                                        } else {
                                                            "Media Audio".to_string()
                                                        };
                                                        return MediaInfo {
                                                            title: format!("Active Audio ({})", display_name),
                                                            artist: "".to_string(),
                                                            is_playing: true,
                                                            has_media: true,
                                                            thumbnail_bytes: None,
                                                            position_secs: 0,
                                                            duration_secs: 0,
                                                            source_app: display_name,
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

    MediaInfo::default()
}

fn get_process_name_by_id(pid: u32) -> String {
    use windows::Win32::System::ProcessStatus::*;
    use windows::Win32::System::Threading::*;

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid);
        if let Ok(h) = handle {
            let mut name_buf = [0u16; 260];
            let len = GetModuleBaseNameW(h, None, &mut name_buf);
            let _ = CloseHandle(h);
            if len > 0 {
                return String::from_utf16_lossy(&name_buf[..len as usize]).to_lowercase();
            }
        }
    }
    String::new()
}
