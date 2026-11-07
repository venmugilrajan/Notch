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

