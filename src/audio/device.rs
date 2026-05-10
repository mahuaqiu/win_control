use pyo3::prelude::*;
use windows::core::PCWSTR;
use windows::Win32::Media::Audio::{
    IMMDeviceEnumerator, MMDeviceEnumerator, eRender, eCapture, eConsole, eMultimedia, eCommunications,
    DEVICE_STATE, DEVICE_STATE_ACTIVE, DEVICE_STATE_DISABLED, DEVICE_STATE_UNPLUGGED,
    DEVICE_STATE_NOTPRESENT, DEVICE_STATEMASK_ALL,
};
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_ALL, STGM_READ, COINIT_MULTITHREADED};
use windows::Win32::UI::Shell::PropertiesSystem::{IPropertyStore, PROPERTYKEY};
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::core::BSTR;

use super::{DeviceInfo, DeviceState, AudioError};
use super::error::AudioErrorInner;
use super::policy_config::{create_policy_config, EROLE_CONSOLE, EROLE_MULTIMEDIA, EROLE_COMMUNICATIONS};

/// 设备数据流方向
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeviceDataFlow {
    Render,  // 输出设备（扬声器）
    Capture, // 输入设备（麦克风）
    All,     // 所有设备
}

impl DeviceDataFlow {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "render" | "output" | "speaker" | "speakers" => DeviceDataFlow::Render,
            "capture" | "input" | "microphone" | "mic" => DeviceDataFlow::Capture,
            _ => DeviceDataFlow::All,
        }
    }
}

/// 枚举音频设备的内部实现
pub fn list_devices_impl(device_type: &str) -> Result<Vec<DeviceInfo>, AudioErrorInner> {
    let data_flow = DeviceDataFlow::from_str(device_type);

    // 初始化 COM（使用 MTA 模式）
    unsafe {
        let hr = CoInitializeEx(None, COINIT_MULTITHREADED);
        if hr.is_err() {
            return Err(AudioErrorInner::ComError(format!("COM 初始化失败: {}", hr)));
        }
    }

    // 创建设备枚举器
    let enumerator: IMMDeviceEnumerator = unsafe {
        CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
            .map_err(|e| AudioErrorInner::ComError(format!("无法创建设备枚举器: {}", e)))?
    };

    let mut devices = Vec::new();

    // 根据设备类型枚举
    let flows = match data_flow {
        DeviceDataFlow::Render => vec![eRender],
        DeviceDataFlow::Capture => vec![eCapture],
        DeviceDataFlow::All => vec![eRender, eCapture],
    };

    for flow in flows {
        // 只获取 active 状态的设备（排除 not_present, disabled, unplugged）
        let collection = unsafe {
            enumerator.EnumAudioEndpoints(flow, DEVICE_STATE_ACTIVE)
                .map_err(|e| AudioErrorInner::EnumerationFailed(format!("无法枚举设备: {}", e)))?
        };

        let count = unsafe {
            collection.GetCount()
                .map_err(|e| AudioErrorInner::EnumerationFailed(format!("无法获取设备数量: {}", e)))?
        };

        for i in 0..count {
            let device = unsafe {
                collection.Item(i)
                    .map_err(|e| AudioErrorInner::EnumerationFailed(format!("无法获取设备: {}", e)))?
            };

            // 获取设备ID
            let id = unsafe {
                let id_ptr = device.GetId()
                    .map_err(|e| AudioErrorInner::EnumerationFailed(format!("无法获取设备ID: {}", e)))?;
                let id_slice = id_ptr.as_wide();
                String::from_utf16_lossy(id_slice)
            };

            // 获取设备状态
            let state = unsafe {
                device.GetState()
                    .map_err(|e| AudioErrorInner::EnumerationFailed(format!("无法获取设备状态: {}", e)))?
            };

            let state_str = device_state_to_string(state.0);

            // 获取设备名称（根据 flow 确定类型）
            let name = get_device_name(&device)?;
            let device_type_str = if flow == eRender { "render" } else { "capture" };

            devices.push(DeviceInfo::new(
                id,
                name,
                device_type_str.to_string(),
                state_str,
            ));
        }
    }

    Ok(devices)
}

/// 将设备状态转换为字符串
fn device_state_to_string(state: u32) -> String {
    if state & DEVICE_STATE_ACTIVE.0 != 0 {
        "active".to_string()
    } else if state & DEVICE_STATE_DISABLED.0 != 0 {
        "disabled".to_string()
    } else if state & DEVICE_STATE_UNPLUGGED.0 != 0 {
        "unplugged".to_string()
    } else if state & DEVICE_STATE_NOTPRESENT.0 != 0 {
        "not_present".to_string()
    } else {
        "unknown".to_string()
    }
}

/// 获取设备名称
fn get_device_name(device: &windows::Win32::Media::Audio::IMMDevice) -> Result<String, AudioErrorInner> {
    unsafe {
        // 尝试打开属性存储，失败时返回默认值
        let store_result = device.OpenPropertyStore(STGM_READ);
        if store_result.is_err() {
            return Ok("未知设备".to_string());
        }
        let store: IPropertyStore = store_result.unwrap();

        // PKEY_Device_FriendlyName
        let friendly_name_key = PROPERTYKEY {
            fmtid: windows::core::GUID::from_u128(0xa45c254e_df1c_4efd_8020_67d146a850e0),
            pid: 14,
        };

        let prop_result = store.GetValue(&friendly_name_key);
        if prop_result.is_err() {
            return Ok("未知设备".to_string());
        }

        let prop = prop_result.unwrap();
        Ok(BSTR::try_from(&prop)
            .map(|bstr| bstr.to_string())
            .unwrap_or_else(|_| "未知设备".to_string()))
    }
}

/// 枚举所有音频设备
#[pyfunction]
#[pyo3(signature = (device_type="all".into()))]
pub fn list_devices(_py: Python<'_>, device_type: String) -> PyResult<Vec<DeviceInfo>> {
    list_devices_impl(&device_type).map_err(|e| AudioError::new_err(e.to_string()))
}

/// 获取设备状态的内部实现
pub fn get_device_state_impl(device_id: String) -> Result<DeviceState, AudioErrorInner> {
    // 初始化 COM（使用 MTA 模式）
    unsafe {
        let hr = CoInitializeEx(None, COINIT_MULTITHREADED);
        if hr.is_err() {
            return Err(AudioErrorInner::ComError(format!("COM 初始化失败: {}", hr)));
        }
    }

    // 创建设备枚举器
    let enumerator: IMMDeviceEnumerator = unsafe {
        CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
            .map_err(|e| AudioErrorInner::ComError(format!("无法创建设备枚举器: {}", e)))?
    };

    // 将设备ID转换为宽字符串
    let wide: Vec<u16> = device_id.encode_utf16().chain(std::iter::once(0)).collect();
    let device_id_ptr = PCWSTR::from_raw(wide.as_ptr());

    // 获取设备
    let device = unsafe {
        enumerator.GetDevice(device_id_ptr)
            .map_err(|e| AudioErrorInner::DeviceNotFound(format!("设备未找到: {}", e)))?
    };

    // 获取设备状态
    let state = unsafe {
        device.GetState()
            .map_err(|e| AudioErrorInner::EnumerationFailed(format!("无法获取设备状态: {}", e)))?
    };

    let state_str = device_state_to_string(state.0);

    // 检查是否为默认设备
    let is_default = is_default_device_impl_inner(&enumerator, &device_id)?;

    // 获取音量和静音状态
    let (volume, is_muted) = get_volume_and_mute(&device)?;

    Ok(DeviceState::new(state_str, is_default, volume, is_muted))
}

/// 检查设备是否为默认设备
fn is_default_device_impl_inner(enumerator: &IMMDeviceEnumerator, device_id: &str) -> Result<bool, AudioErrorInner> {
    unsafe {
        // 检查是否为默认输出设备
        if let Ok(default_render) = enumerator.GetDefaultAudioEndpoint(eRender, eConsole) {
            if let Ok(default_id) = default_render.GetId() {
                let default_id_str = String::from_utf16_lossy(default_id.as_wide());
                if default_id_str == device_id {
                    return Ok(true);
                }
            }
        }

        // 检查是否为默认输入设备
        if let Ok(default_capture) = enumerator.GetDefaultAudioEndpoint(eCapture, eConsole) {
            if let Ok(default_id) = default_capture.GetId() {
                let default_id_str = String::from_utf16_lossy(default_id.as_wide());
                if default_id_str == device_id {
                    return Ok(true);
                }
            }
        }
    }

    Ok(false)
}

/// 从设备获取音量和静音状态
fn get_volume_and_mute(device: &windows::Win32::Media::Audio::IMMDevice) -> Result<(f32, bool), AudioErrorInner> {
    unsafe {
        let endpoint: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None)
            .map_err(|e| AudioErrorInner::VolumeError(format!("无法获取音频端点: {}", e)))?;

        let volume = endpoint.GetMasterVolumeLevelScalar()
            .map_err(|e| AudioErrorInner::VolumeError(format!("无法获取音量: {}", e)))?;

        let is_muted = endpoint.GetMute()
            .map_err(|e| AudioErrorInner::VolumeError(format!("无法获取静音状态: {}", e)))?
            .as_bool();

        Ok((volume, is_muted))
    }
}

/// 获取设备状态
#[pyfunction]
pub fn get_device_state(_py: Python<'_>, device_id: String) -> PyResult<DeviceState> {
    get_device_state_impl(device_id).map_err(|e| AudioError::new_err(e.to_string()))
}

// ============================================================
// Task 15: 设备禁用/启用
// ============================================================

/// 禁用音频设备的内部实现
pub fn disable_device_impl(device_id: String) -> Result<(), AudioErrorInner> {
    let policy = create_policy_config()
        .map_err(|e| AudioErrorInner::ComError(format!("无法创建 PolicyConfig: {}", e)))?;
    let wide: Vec<u16> = device_id.encode_utf16().chain(std::iter::once(0)).collect();
    let device_id_ptr = PCWSTR::from_raw(wide.as_ptr());
    policy.set_endpoint_visibility(device_id_ptr, 0)
        .map_err(|e| AudioErrorInner::PermissionDenied(format!("禁用设备失败: {}", e)))?;
    Ok(())
}

/// 禁用音频设备
#[pyfunction]
pub fn disable_device(_py: Python<'_>, device_id: String) -> PyResult<()> {
    disable_device_impl(device_id).map_err(|e| AudioError::new_err(e.to_string()))
}

/// 启用音频设备的内部实现
pub fn enable_device_impl(device_id: String) -> Result<(), AudioErrorInner> {
    let policy = create_policy_config()
        .map_err(|e| AudioErrorInner::ComError(format!("无法创建 PolicyConfig: {}", e)))?;
    let wide: Vec<u16> = device_id.encode_utf16().chain(std::iter::once(0)).collect();
    let device_id_ptr = PCWSTR::from_raw(wide.as_ptr());
    policy.set_endpoint_visibility(device_id_ptr, 1)
        .map_err(|e| AudioErrorInner::PermissionDenied(format!("启用设备失败: {}", e)))?;
    Ok(())
}

/// 启用音频设备
#[pyfunction]
pub fn enable_device(_py: Python<'_>, device_id: String) -> PyResult<()> {
    enable_device_impl(device_id).map_err(|e| AudioError::new_err(e.to_string()))
}

// ============================================================
// Task 16: 默认设备管理
// ============================================================

/// 将角色字符串转换为 ERole 常量
fn role_from_str(role: &str) -> i32 {
    match role.to_lowercase().as_str() {
        "console" | "default" => EROLE_CONSOLE,
        "multimedia" | "media" => EROLE_MULTIMEDIA,
        "communications" | "comm" => EROLE_COMMUNICATIONS,
        _ => EROLE_CONSOLE,
    }
}

/// 获取默认音频设备的内部实现
pub fn get_default_device_impl(device_type: &str, role: &str) -> Result<DeviceInfo, AudioErrorInner> {
    // 初始化 COM（使用 MTA 模式）
    unsafe {
        let hr = CoInitializeEx(None, COINIT_MULTITHREADED);
        if hr.is_err() {
            return Err(AudioErrorInner::ComError(format!("COM 初始化失败: {}", hr)));
        }
    }

    // 创建设备枚举器
    let enumerator: IMMDeviceEnumerator = unsafe {
        CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
            .map_err(|e| AudioErrorInner::ComError(format!("无法创建设备枚举器: {}", e)))?
    };

    // 确定数据流方向
    let data_flow = match device_type.to_lowercase().as_str() {
        "render" | "output" | "speaker" | "speakers" => eRender,
        "capture" | "input" | "microphone" | "mic" => eCapture,
        _ => eRender,
    };

    // 确定角色
    let role_enum = match role.to_lowercase().as_str() {
        "multimedia" | "media" => eMultimedia,
        "communications" | "comm" => eCommunications,
        _ => eConsole,
    };

    // 获取默认设备
    let device = unsafe {
        enumerator.GetDefaultAudioEndpoint(data_flow, role_enum)
            .map_err(|e| AudioErrorInner::DeviceNotFound(format!("无法获取默认设备: {}", e)))?
    };

    // 获取设备ID
    let id = unsafe {
        let id_ptr = device.GetId()
            .map_err(|e| AudioErrorInner::EnumerationFailed(format!("无法获取设备ID: {}", e)))?;
        let id_slice = id_ptr.as_wide();
        String::from_utf16_lossy(id_slice)
    };

    // 获取设备名称
    let name = get_device_name(&device)?;

    // 获取设备状态
    let state = unsafe {
        device.GetState()
            .map_err(|e| AudioErrorInner::EnumerationFailed(format!("无法获取设备状态: {}", e)))?
    };
    let state_str = device_state_to_string(state.0);

    let device_type_str = if data_flow == eRender { "render" } else { "capture" };

    Ok(DeviceInfo::new(id, name, device_type_str.to_string(), state_str))
}

/// 获取默认音频设备
#[pyfunction]
#[pyo3(signature = (device_type="speaker".into(), role="console".into()))]
pub fn get_default_device(_py: Python<'_>, device_type: String, role: String) -> PyResult<DeviceInfo> {
    get_default_device_impl(&device_type, &role).map_err(|e| AudioError::new_err(e.to_string()))
}

/// 设置默认音频设备的内部实现
pub fn set_default_device_impl(device_id: String, role: &str) -> Result<(), AudioErrorInner> {
    let policy = create_policy_config()
        .map_err(|e| AudioErrorInner::ComError(format!("无法创建 PolicyConfig: {}", e)))?;
    let wide: Vec<u16> = device_id.encode_utf16().chain(std::iter::once(0)).collect();
    let device_id_ptr = PCWSTR::from_raw(wide.as_ptr());
    let role_enum = role_from_str(role);
    policy.set_default_endpoint(device_id_ptr, role_enum)
        .map_err(|e| AudioErrorInner::PermissionDenied(format!("设置默认设备失败: {}", e)))?;
    Ok(())
}

/// 设置默认音频设备
#[pyfunction]
#[pyo3(signature = (device_id, role="console".into()))]
pub fn set_default_device(_py: Python<'_>, device_id: String, role: String) -> PyResult<()> {
    set_default_device_impl(device_id, &role).map_err(|e| AudioError::new_err(e.to_string()))
}