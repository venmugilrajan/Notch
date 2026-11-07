use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Storage::FileSystem::*;
use windows::Win32::System::SystemInformation::*;

#[link(name = "kernel32")]
extern "system" {
    pub fn GetSystemTimes(
        lpIdleTime: *mut FILETIME,
        lpKernelTime: *mut FILETIME,
        lpUserTime: *mut FILETIME,
    ) -> BOOL;
}

#[derive(Clone, Debug, Default)]
pub struct SystemStatsData {
    pub cpu_usage: f32,
    pub ram_used_mb: u64,
    pub ram_total_mb: u64,
    pub ram_percent: f32,
    pub disk_used_gb: f32,
    pub disk_total_gb: f32,
    pub disk_percent: f32,
    pub uptime_seconds: u64,
}

pub struct SystemStatsSampler {
    rx: Receiver<SystemStatsData>,
    current: SystemStatsData,
    is_active: Arc<AtomicBool>,
}

impl SystemStatsSampler {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        let is_active = Arc::new(AtomicBool::new(false));
        let active_flag = is_active.clone();

        thread::spawn(move || {
            let mut prev_idle = 0u64;
            let mut prev_kernel = 0u64;
            let mut prev_user = 0u64;

            loop {
                if active_flag.load(Ordering::Relaxed) {
                    let data = sample_system_metrics(&mut prev_idle, &mut prev_kernel, &mut prev_user);
                    let _ = tx.send(data);
                    thread::sleep(Duration::from_millis(1000));
                } else {
                    thread::sleep(Duration::from_millis(500));
                }
            }
        });

        Self {
            rx,
            current: SystemStatsData::default(),
            is_active,
        }
    }

    pub fn set_active(&self, active: bool) {
        self.is_active.store(active, Ordering::Relaxed);
    }

    pub fn get_metrics(&mut self) -> SystemStatsData {
        while let Ok(data) = self.rx.try_recv() {
            self.current = data;
        }
        self.current.clone()
    }
}

fn filetime_to_u64(ft: &FILETIME) -> u64 {
    ((ft.dwHighDateTime as u64) << 32) | (ft.dwLowDateTime as u64)
}

fn sample_system_metrics(prev_idle: &mut u64, prev_kernel: &mut u64, prev_user: &mut u64) -> SystemStatsData {
    unsafe {
        // 1. RAM
        let mut mem = MEMORYSTATUSEX::default();
        mem.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
        let _ = GlobalMemoryStatusEx(&mut mem);

        let total_mb = mem.ullTotalPhys / (1024 * 1024);
        let avail_mb = mem.ullAvailPhys / (1024 * 1024);
        let used_mb = total_mb.saturating_sub(avail_mb);
        let ram_pct = mem.dwMemoryLoad as f32;

        // 2. Real CPU Usage via GetSystemTimes
        let mut idle_ft = FILETIME::default();
        let mut kernel_ft = FILETIME::default();
        let mut user_ft = FILETIME::default();
        let mut cpu_usage = 0.0f32;

        if GetSystemTimes(&mut idle_ft, &mut kernel_ft, &mut user_ft).as_bool() {
            let cur_idle = filetime_to_u64(&idle_ft);
            let cur_kernel = filetime_to_u64(&kernel_ft);
            let cur_user = filetime_to_u64(&user_ft);

            if *prev_kernel > 0 || *prev_user > 0 {
                let diff_idle = cur_idle.saturating_sub(*prev_idle);
                let diff_kernel = cur_kernel.saturating_sub(*prev_kernel);
                let diff_user = cur_user.saturating_sub(*prev_user);
                let total_sys = diff_kernel + diff_user;

                if total_sys > 0 {
                    let busy = total_sys.saturating_sub(diff_idle);
                    cpu_usage = (busy as f32 / total_sys as f32) * 100.0;
                }
            }

            *prev_idle = cur_idle;
            *prev_kernel = cur_kernel;
            *prev_user = cur_user;
        }

        // 3. Real Disk Space for C:\
        let mut free_bytes_avail = 0u64;
        let mut total_bytes = 0u64;
        let mut total_free = 0u64;
        let mut disk_used_gb = 0.0f32;
        let mut disk_total_gb = 0.0f32;
        let mut disk_pct = 0.0f32;

        if GetDiskFreeSpaceExW(
            w!("C:\\"),
            Some(&mut free_bytes_avail),
            Some(&mut total_bytes),
            Some(&mut total_free),
        ).is_ok() {
            let total_gb = total_bytes as f32 / (1024.0 * 1024.0 * 1024.0);
            let free_gb = total_free as f32 / (1024.0 * 1024.0 * 1024.0);
            let used_gb = total_gb - free_gb;
            disk_total_gb = total_gb;
            disk_used_gb = used_gb;
            if total_gb > 0.0 {
                disk_pct = (used_gb / total_gb) * 100.0;
            }
        }

        // 4. Real System Uptime
        let uptime_ms = GetTickCount64();
        let uptime_seconds = uptime_ms / 1000;

        SystemStatsData {
            cpu_usage: cpu_usage.clamp(0.0, 100.0),
            ram_used_mb: used_mb,
            ram_total_mb: total_mb,
            ram_percent: ram_pct,
            disk_used_gb,
            disk_total_gb,
            disk_percent: disk_pct,
            uptime_seconds,
        }
    }
}
