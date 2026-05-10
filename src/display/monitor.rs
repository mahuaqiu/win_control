use pyo3::prelude::*;
use std::cell::RefCell;
use windows::core::PCWSTR;
use windows::Win32::Graphics::Gdi::{
    EnumDisplayDevicesW, EnumDisplaySettingsExW, DEVMODEW,
    DISPLAY_DEVICEW, DISPLAY_DEVICE_ACTIVE, DISPLAY_DEVICE_PRIMARY_DEVICE,
    ENUM_DISPLAY_SETTINGS_MODE, ENUM_DISPLAY_SETTINGS_FLAGS,
};
use windows::Win32::Devices::Display::{
    GetDisplayConfigBufferSizes, QueryDisplayConfig, SetDisplayConfig,
    DISPLAYCONFIG_PATH_INFO, DISPLAYCONFIG_MODE_INFO,
    DISPLAYCONFIG_SCALING,
    QUERY_DISPLAY_CONFIG_FLAGS, SET_DISPLAY_CONFIG_FLAGS,
    DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE, DISPLAYCONFIG_TOPOLOGY_ID,
    QDC_ONLY_ACTIVE_PATHS, QDC_ALL_PATHS, QDC_DATABASE_CURRENT,
    SDC_APPLY, SDC_ALLOW_CHANGES, SDC_SAVE_TO_DATABASE, SDC_FORCE_MODE_ENUMERATION,
    SDC_USE_SUPPLIED_DISPLAY_CONFIG,
};
use windows::Win32::Foundation::LUID;

use super::DisplayErrorInner;

// 线程本地存储，用于保存原始分辨率以便恢复
thread_local! {
    static ORIGINAL_RESOLUTION: RefCell<Option<(String, Resolution)>> = RefCell::new(None);
}

/// 显示器信息
#[pyclass]
#[derive(Clone)]
pub struct MonitorInfo {
    #[pyo3(get)]
    id: String,
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    is_primary: bool,
}

impl MonitorInfo {
    pub fn new(id: String, name: String, is_primary: bool) -> Self {
        Self { id, name, is_primary }
    }
}

/// 分辨率信息
#[pyclass]
#[derive(Clone)]
pub struct Resolution {
    #[pyo3(get)]
    width: u32,
    #[pyo3(get)]
    height: u32,
    #[pyo3(get)]
    refresh_rate: u32,
}

impl Resolution {
    pub fn new(width: u32, height: u32, refresh_rate: u32) -> Self {
        Self { width, height, refresh_rate }
    }
}

/// 枚举所有显示器的内部实现
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

        if get_current_resolution_impl(Some(device_name.clone())).is_ok() {
            let is_primary = (display_device.StateFlags & DISPLAY_DEVICE_PRIMARY_DEVICE) != 0;

            monitors.push(MonitorInfo::new(
                device_name,
                friendly_name,
                is_primary,
            ));
        }

        i += 1;
    }

    Ok(monitors)
}

/// 枚举所有显示器
#[pyfunction]
pub fn list_monitors() -> PyResult<Vec<MonitorInfo>> {
    list_monitors_impl().map_err(|e: DisplayErrorInner| super::DisplayError::new_err(e.to_string()))
}

/// 获取当前显示设置的模式标识
const ENUM_CURRENT_SETTINGS: u32 = 0xFFFFFFFF;

/// 获取当前分辨率的内部实现
pub fn get_current_resolution_impl(monitor_id: Option<String>) -> Result<Resolution, DisplayErrorInner> {
    let wide: Option<Vec<u16>> = monitor_id.map(|s| {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    });
    let device_name = wide.as_ref().map(|w| PCWSTR::from_raw(w.as_ptr()));

    let mut dev_mode = DEVMODEW {
        dmSize: std::mem::size_of::<DEVMODEW>() as u16,
        ..Default::default()
    };

    let result = unsafe {
        EnumDisplaySettingsExW(
            device_name.unwrap_or(PCWSTR::null()),
            ENUM_DISPLAY_SETTINGS_MODE(ENUM_CURRENT_SETTINGS),
            &mut dev_mode,
            ENUM_DISPLAY_SETTINGS_FLAGS(0),
        )
    };

    if !result.as_bool() {
        return Err(DisplayErrorInner::EnumerationFailed("Failed to get current resolution".into()));
    }

    Ok(Resolution::new(
        dev_mode.dmPelsWidth,
        dev_mode.dmPelsHeight,
        dev_mode.dmDisplayFrequency,
    ))
}

/// 获取当前分辨率
#[pyfunction]
#[pyo3(signature = (monitor_id=None))]
pub fn get_current_resolution(_py: Python<'_>, monitor_id: Option<String>) -> PyResult<Resolution> {
    get_current_resolution_impl(monitor_id).map_err(|e| super::DisplayError::new_err(e.to_string()))
}

/// 获取所有分辨率模式的内部实现（完整列表）
pub fn get_all_resolutions_impl(monitor_id: Option<String>) -> Result<Vec<Resolution>, DisplayErrorInner> {
    let wide: Option<Vec<u16>> = monitor_id.map(|s| {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    });
    let device_name = wide.as_ref().map(|w| PCWSTR::from_raw(w.as_ptr()));

    let mut resolutions = Vec::new();
    let mut i: u32 = 0;

    loop {
        let mut dev_mode = DEVMODEW {
            dmSize: std::mem::size_of::<DEVMODEW>() as u16,
            ..Default::default()
        };

        let result = unsafe {
            EnumDisplaySettingsExW(
                device_name.unwrap_or(PCWSTR::null()),
                ENUM_DISPLAY_SETTINGS_MODE(i),
                &mut dev_mode,
                ENUM_DISPLAY_SETTINGS_FLAGS(0),
            )
        };

        if !result.as_bool() {
            break;
        }

        resolutions.push(Resolution::new(
            dev_mode.dmPelsWidth,
            dev_mode.dmPelsHeight,
            dev_mode.dmDisplayFrequency,
        ));

        i += 1;
    }

    Ok(resolutions)
}

/// 获取所有分辨率模式（完整列表）
#[pyfunction]
#[pyo3(signature = (monitor_id=None))]
pub fn get_all_resolutions(_py: Python<'_>, monitor_id: Option<String>) -> PyResult<Vec<Resolution>> {
    get_all_resolutions_impl(monitor_id).map_err(|e| super::DisplayError::new_err(e.to_string()))
}

/// 获取支持的分辨率列表的内部实现（去重版）
pub fn get_supported_resolutions_impl(monitor_id: Option<String>) -> Result<Vec<Resolution>, DisplayErrorInner> {
    let raw = get_all_resolutions_impl(monitor_id)?;

    let mut resolution_map: std::collections::HashMap<(u32, u32), u32> = std::collections::HashMap::new();

    for r in raw {
        if r.width >= 800 && r.height >= 600 {
            let key = (r.width, r.height);
            resolution_map
                .entry(key)
                .and_modify(|refresh| *refresh = std::cmp::max(*refresh, r.refresh_rate))
                .or_insert(r.refresh_rate);
        }
    }

    let mut resolutions: Vec<Resolution> = resolution_map
        .into_iter()
        .map(|((w, h), r)| Resolution::new(w, h, r))
        .collect();

    resolutions.sort_by(|a, b| {
        (b.width * b.height).cmp(&(a.width * a.height))
    });

    Ok(resolutions)
}

/// 获取支持的分辨率列表
#[pyfunction]
#[pyo3(signature = (monitor_id=None))]
pub fn get_supported_resolutions(_py: Python<'_>, monitor_id: Option<String>) -> PyResult<Vec<Resolution>> {
    get_supported_resolutions_impl(monitor_id).map_err(|e| super::DisplayError::new_err(e.to_string()))
}

/// 使用 CCD API 设置分辨率（支持 GPU 缩放）
fn set_resolution_ccd(width: u32, height: u32, monitor_id: Option<String>, skip_save: bool) -> Result<(), DisplayErrorInner> {
    // 保存原始分辨率
    if !skip_save {
        let current = get_current_resolution_impl(monitor_id.clone())?;
        let monitor_name = monitor_id.clone().unwrap_or_else(|| {
            list_monitors_impl()
                .ok()
                .and_then(|m| m.into_iter().find(|m| m.is_primary).map(|m| m.id))
                .unwrap_or_default()
        });
        ORIGINAL_RESOLUTION.with(|r| {
            *r.borrow_mut() = Some((monitor_name, current));
        });
    }

    // 获取 buffer 大小
    let mut path_count: u32 = 0;
    let mut mode_count: u32 = 0;

    unsafe {
        let result = GetDisplayConfigBufferSizes(
            QDC_DATABASE_CURRENT,
            &mut path_count,
            &mut mode_count,
        );
        if result.0 != 0 {
            return Err(DisplayErrorInner::EnumerationFailed(format!("GetDisplayConfigBufferSizes 失败: 错误码 {}", result.0)));
        }
    }

    // 确保 buffer 大小有效
    if path_count == 0 || mode_count == 0 {
        return Err(DisplayErrorInner::EnumerationFailed("无法获取显示配置 buffer 大小".into()));
    }

    // 分配 buffer - 使用 zeroed 初始化
    let mut paths: Vec<DISPLAYCONFIG_PATH_INFO> = unsafe {
        let mut v: Vec<DISPLAYCONFIG_PATH_INFO> = Vec::with_capacity(path_count as usize);
        v.set_len(path_count as usize);
        for i in 0..path_count as usize {
            v[i] = core::mem::zeroed();
        }
        v
    };
    let mut modes: Vec<DISPLAYCONFIG_MODE_INFO> = unsafe {
        let mut v: Vec<DISPLAYCONFIG_MODE_INFO> = Vec::with_capacity(mode_count as usize);
        v.set_len(mode_count as usize);
        for i in 0..mode_count as usize {
            v[i] = core::mem::zeroed();
        }
        v
    };
    let mut topology_id: DISPLAYCONFIG_TOPOLOGY_ID = DISPLAYCONFIG_TOPOLOGY_ID(0);

    // 获取当前配置
    unsafe {
        let result = QueryDisplayConfig(
            QDC_DATABASE_CURRENT,
            &mut path_count,
            paths.as_mut_ptr(),
            &mut mode_count,
            modes.as_mut_ptr(),
            Some(&mut topology_id),
        );
        if result.0 != 0 {
            return Err(DisplayErrorInner::EnumerationFailed(format!("QueryDisplayConfig 失败: 错误码 {}", result.0)));
        }
    }

    // 调整 buffer 到实际大小
    paths.truncate(path_count as usize);
    modes.truncate(mode_count as usize);

    // 找到主显示器的活跃路径
    // 主显示器的 Source Mode position 是 (0, 0)
    let mut active_path_index = 0;
    let mut primary_found = false;

    for (i, path) in paths.iter().enumerate() {
        if path.flags & 1 != 0 { // DISPLAYCONFIG_PATH_ACTIVE
            let source_mode_idx = unsafe { path.sourceInfo.Anonymous.modeInfoIdx };
            if source_mode_idx < mode_count as u32 {
                let mode = &modes[source_mode_idx as usize];
                if mode.infoType == DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE {
                    let pos = unsafe { mode.Anonymous.sourceMode.position };
                    // 主显示器的 position 是 (0, 0)
                    if pos.x == 0 && pos.y == 0 {
                        active_path_index = i;
                        primary_found = true;
                        break;
                    }
                }
            }
        }
    }

    // 如果没找到 position (0,0)，使用第一个活跃路径
    if !primary_found {
        active_path_index = paths.iter().position(|path| {
            path.flags & 1 != 0
        }).unwrap_or(0);
    }

    // 如果指定了显示器 ID，尝试匹配
    if let Some(monitor_id) = &monitor_id {
        for (i, path) in paths.iter().enumerate() {
            if path.flags & 1 != 0 {
                let expected_id = if monitor_id.starts_with("\\\\.\\DISPLAY") {
                    monitor_id.replace("\\\\.\\DISPLAY", "").parse::<u32>().unwrap_or(1) - 1
                } else {
                    path.sourceInfo.id
                };
                if path.sourceInfo.id == expected_id {
                    active_path_index = i;
                    break;
                }
            }
        }
    }

    // 设置缩放模式以支持非原生分辨率
    paths[active_path_index].targetInfo.scaling = DISPLAYCONFIG_SCALING(4);

    // 获取活跃路径的 Source Mode 索引
    let source_mode_idx = unsafe { paths[active_path_index].sourceInfo.Anonymous.modeInfoIdx };

    // 修改对应的 Source Mode（桌面逻辑分辨率）
    if source_mode_idx < mode_count as u32 {
        let mode = &mut modes[source_mode_idx as usize];
        if mode.infoType == DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE {
            unsafe {
                mode.Anonymous.sourceMode.width = width;
                mode.Anonymous.sourceMode.height = height;
            }
        }
    }

    // 应用配置
    let flags = SDC_USE_SUPPLIED_DISPLAY_CONFIG | SDC_APPLY | SDC_ALLOW_CHANGES;

    unsafe {
        let result = SetDisplayConfig(
            Some(&paths),
            Some(&modes),
            flags,
        );
        if result != 0 {
            return Err(DisplayErrorInner::EnumerationFailed(format!("SetDisplayConfig 失败: 错误码 {}", result)));
        }
    }

    Ok(())
}

/// 设置分辨率的内部实现
pub fn set_resolution_impl(width: u32, height: u32, monitor_id: Option<String>) -> Result<(), DisplayErrorInner> {
    set_resolution_ccd(width, height, monitor_id, false)
}

/// 设置分辨率（自动选择最佳刷新率）
#[pyfunction]
#[pyo3(signature = (width, height, monitor_id=None))]
pub fn set_resolution(_py: Python<'_>, width: u32, height: u32, monitor_id: Option<String>) -> PyResult<()> {
    set_resolution_impl(width, height, monitor_id).map_err(|e| super::DisplayError::new_err(e.to_string()))
}

/// 设置分辨率（指定刷新率）- CCD API 不直接支持指定刷新率，此函数保留但使用 CCD
#[pyfunction]
#[pyo3(signature = (width, height, refresh_rate, monitor_id=None))]
pub fn set_resolution_with_refresh(_py: Python<'_>, width: u32, height: u32, refresh_rate: u32, monitor_id: Option<String>) -> PyResult<()> {
    // CCD API 的刷新率由 Target Mode 控制，这里简化处理
    set_resolution_ccd(width, height, monitor_id, false)
        .map_err(|e| super::DisplayError::new_err(e.to_string()))
}

/// 恢复原始分辨率的内部实现
pub fn restore_resolution_impl(monitor_id: Option<String>) -> Result<(), DisplayErrorInner> {
    ORIGINAL_RESOLUTION.with(|r| {
        let stored = r.borrow().clone();
        if let Some((monitor_name, resolution)) = stored {
            let target_id = monitor_id.or(Some(monitor_name));
            set_resolution_ccd(resolution.width, resolution.height, target_id, true)?;
        } else {
            return Err(DisplayErrorInner::DeviceNotFound("没有保存的原始分辨率".into()));
        }
        Ok(())
    })
}

/// 恢复原始分辨率
#[pyfunction]
#[pyo3(signature = (monitor_id=None))]
pub fn restore_resolution(_py: Python<'_>, monitor_id: Option<String>) -> PyResult<()> {
    restore_resolution_impl(monitor_id).map_err(|e| super::DisplayError::new_err(e.to_string()))
}