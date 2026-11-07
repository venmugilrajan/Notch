use windows::Win32::Foundation::*;
use windows::Win32::System::Power::*;

#[link(name = "user32")]
extern "system" {
    pub fn RegisterPowerSettingNotification(
        hRecipient: HANDLE,
        PowerSettingGuid: *const windows::core::GUID,
        Flags: u32,
    ) -> HPOWERNOTIFY;

    pub fn UnregisterPowerSettingNotification(
        Handle: HPOWERNOTIFY,
    ) -> BOOL;
}

// GUIDs for power setting notifications
pub const GUID_BATTERY_PERCENTAGE_REMAINING: windows::core::GUID = windows::core::GUID::from_u128(0xA7AD8041_B45A_4CAE_87A3_EECBB468A9E1);
pub const GUID_ACDC_POWER_SOURCE: windows::core::GUID = windows::core::GUID::from_u128(0x5D3E4A24_E925_4894_A613_14A4E9E6509F);

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PowerStatusInfo {
    pub ac_online: bool,
    pub has_battery: bool,
    pub battery_percent: u8,
    pub is_charging: bool,
    pub battery_flag: u8,
    pub battery_life_time_sec: u32,
    pub raw_ac_line_status: u8,
}

impl PowerStatusInfo {
    pub fn read_current() -> Self {
        unsafe {
            let mut status = SYSTEM_POWER_STATUS::default();
            if GetSystemPowerStatus(&mut status).is_ok() {
                // BatteryFlag:
                // 1 = High, 2 = Low, 4 = Critical, 8 = Charging
                // 128 = No system battery, 255 = Unknown status
                let has_battery = status.BatteryFlag != 128 && status.BatteryFlag != 255 && status.BatteryLifePercent != 255;
                let ac_online = status.ACLineStatus == 1;
                let is_charging = (status.BatteryFlag & 8) != 0;
                let battery_percent = if has_battery { status.BatteryLifePercent.min(100) } else { 0 };

                Self {
                    ac_online,
                    has_battery,
                    battery_percent,
                    is_charging,
                    battery_flag: status.BatteryFlag,
                    battery_life_time_sec: status.BatteryLifeTime,
                    raw_ac_line_status: status.ACLineStatus,
                }
            } else {
                Self::default()
            }
        }
    }
}

pub struct PowerNotifier {
    reg_battery: HPOWERNOTIFY,
    reg_ac: HPOWERNOTIFY,
}

impl PowerNotifier {
    pub fn register(hwnd: HWND) -> Self {
        unsafe {
            let reg_battery = RegisterPowerSettingNotification(
                HANDLE(hwnd.0),
                &GUID_BATTERY_PERCENTAGE_REMAINING,
                0, // DEVICE_NOTIFY_WINDOW_HANDLE = 0
            );

            let reg_ac = RegisterPowerSettingNotification(
                HANDLE(hwnd.0),
                &GUID_ACDC_POWER_SOURCE,
                0, // DEVICE_NOTIFY_WINDOW_HANDLE = 0
            );

            Self {
                reg_battery,
                reg_ac,
            }
        }
    }

    pub fn unregister(&mut self) {
        unsafe {
            if self.reg_battery.0 != 0 {
                let _ = UnregisterPowerSettingNotification(self.reg_battery);
                self.reg_battery = HPOWERNOTIFY(0);
            }
            if self.reg_ac.0 != 0 {
                let _ = UnregisterPowerSettingNotification(self.reg_ac);
                self.reg_ac = HPOWERNOTIFY(0);
            }
        }
    }
}

impl Drop for PowerNotifier {
    fn drop(&mut self) {
        self.unregister();
    }
}
