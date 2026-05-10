use windows::Win32::Graphics::Gdi::{
    EnumDisplayDevicesW, DISPLAY_DEVICEW, DISPLAY_DEVICE_ACTIVE,
};
use windows::core::PCWSTR;

use super::{DisplayErrorInner, MonitorInfo};

/// 枚举所有显示器
pub fn list_monitors_impl() -> Result<Vec<MonitorInfo>, DisplayErrorInner> {
    let mut monitors = Vec::new();
    let mut i: u32 = 0;

    loop {
        let mut display_device = DISPLAY_DEVICEW {
            cb: std::mem::size_of::<DISPLAY_DEVICEW>() as u32,
            ..Default::default()
        };

        let result = unsafe {
            EnumDisplayDevicesW(
                PCWSTR::null(),
                i,
                &mut display_device,
                DISPLAY_DEVICE_ACTIVE,
            )
        };

        if !result.as_bool() {
            break;
        }

        let device_name = String::from_utf16_lossy(
            &display_device.DeviceName[..display_device.DeviceName.iter().position(|&c| c == 0).unwrap_or(display_device.DeviceName.len())]
        );
        let friendly_name = String::from_utf16_lossy(
            &display_device.DeviceString[..display_device.DeviceString.iter().position(|&c| c == 0).unwrap_or(display_device.DeviceString.len())]
        );

        // DISPLAY_DEVICE_PRIMARY_DEVICE = 4
        let is_primary = (display_device.StateFlags & 4) != 0;

        monitors.push(MonitorInfo::new(
            device_name,
            friendly_name,
            is_primary,
        ));

        i += 1;
    }

    Ok(monitors)
}