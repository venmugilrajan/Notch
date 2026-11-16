use windows::core::*;
use windows::Foundation::AsyncStatus;
use windows::Media::Control::*;
use windows::Win32::System::Com::*;

fn main() {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    }
    println!("Requesting SMTC manager...");
    let req = match GlobalSystemMediaTransportControlsSessionManager::RequestAsync() {
        Ok(r) => r,
        Err(e) => {
            println!("RequestAsync err: {:?}", e);
            return;
        }
    };
    
    // Wait for manager
    let mut mgr = None;
    for i in 0..40 {
        match req.Status() {
            Ok(AsyncStatus::Completed) => {
                mgr = req.GetResults().ok();
                println!("Manager completed at i={}", i);
                break;
            }
            Ok(status) => {
                // print status
            }
            Err(e) => {
                println!("req.Status err: {:?}", e);
                break;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    if let Some(mgr) = mgr {
        println!("Got SMTC manager!");
        if let Ok(curr) = mgr.GetCurrentSession() {
            println!("Current session appId: {:?}", curr.SourceAppUserModelId());
            if let Ok(prop_op) = curr.TryGetMediaPropertiesAsync() {
                for _ in 0..40 {
                    if let Ok(AsyncStatus::Completed) = prop_op.Status() {
                        if let Ok(props) = prop_op.GetResults() {
                            println!("Title: {:?}, Artist: {:?}", props.Title(), props.Artist());
                        }
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
            }
        } else {
            println!("No current session.");
        }

        if let Ok(sessions) = mgr.GetSessions() {
            if let Ok(sz) = sessions.Size() {
                println!("Total sessions count: {}", sz);
                for i in 0..sz {
                    if let Ok(s) = sessions.GetAt(i) {
                        println!("Session [{}]: AppId: {:?}", i, s.SourceAppUserModelId());
                        if let Ok(prop_op) = s.TryGetMediaPropertiesAsync() {
                            for _ in 0..40 {
                                if let Ok(AsyncStatus::Completed) = prop_op.Status() {
                                    if let Ok(props) = prop_op.GetResults() {
                                        println!(" -> Title: {:?}, Artist: {:?}", props.Title(), props.Artist());
                                        if let Ok(thumb) = props.Thumbnail() {
                                            println!(" -> Got thumbnail reference: {:?}", thumb);
                                            if let Ok(stream_op) = thumb.OpenReadAsync() {
                                                println!(" -> OpenReadAsync returned stream_op");
                                                for _ in 0..40 {
                                                    if let Ok(AsyncStatus::Completed) = stream_op.Status() {
                                                        if let Ok(stream) = stream_op.GetResults() {
                                                            let sz = stream.Size().unwrap_or(0);
                                                            println!(" -> Got stream! Size: {:?}", sz);
                                                            let mut buf = vec![0u8; sz as usize];
                                                            let reader = windows::Storage::Streams::DataReader::CreateDataReader(&stream);
                                                            if let Ok(reader) = reader {
                                                                if let Ok(load_op) = reader.LoadAsync(sz as u32) {
                                                                    for _ in 0..40 {
                                                                        if let Ok(AsyncStatus::Completed) = load_op.Status() {
                                                                            let res = reader.ReadBytes(&mut buf);
                                                                            println!(" -> ReadBytes result: {:?}, first 16 bytes: {:?}", res, &buf[..16.min(buf.len())]);
                                                                            break;
                                                                        }
                                                                        std::thread::sleep(std::time::Duration::from_millis(50));
                                                                    }
                                                                }
                                                            }
                                                        } else {
                                                            println!(" -> stream_op.GetResults() failed");
                                                        }
                                                        break;
                                                    }
                                                    std::thread::sleep(std::time::Duration::from_millis(50));
                                                }
                                            } else {
                                                println!(" -> OpenReadAsync returned Err");
                                            }
                                        } else {
                                            println!(" -> props.Thumbnail() failed or returned None");
                                        }
                                    }
                                    break;
                                }
                                std::thread::sleep(std::time::Duration::from_millis(50));
                            }
                        }
                    }
                }
            }
        }
    } else {
        println!("Failed to get SMTC manager within timeout.");
    }
}

