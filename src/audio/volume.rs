use pyo3::prelude::*;
use windows::core::PCWSTR;
use windows::Win32::Media::Audio::{
    IMMDeviceEnumerator, MMDeviceEnumerator, eRender, eCapture, eConsole,
};
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED};
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;

use super::AudioError;
use super::error::AudioErrorInner;

/// 根据设备ID获取音频端点
fn get_audio_endpoint(device_id: Option<&str>, data_flow: windows::Win32::Media::Audio::EDataFlow) -> Result<IAudioEndpointVolume, AudioErrorInner> {
    // 初始化 COM（S_OK 和 S_FALSE 都是成功状态）
    unsafe {
        let hr = CoInitializeEx(None, COINIT_MULTITHREADED);
        if hr.is_err() {
            return Err(AudioErrorInner::ComError(format!("COM 初始化失败: {}", hr)));
        }
    }

    let enumerator: IMMDeviceEnumerator = unsafe {
        CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
            .map_err(|e| AudioErrorInner::ComError(format!("无法创建设备枚举器: {}", e)))?
    };

    let device = if let Some(id) = device_id {
        // 使用指定的设备ID
        let wide: Vec<u16> = id.encode_utf16().chain(std::iter::once(0)).collect();
        let device_id_ptr = PCWSTR::from_raw(wide.as_ptr());

        unsafe {
            enumerator.GetDevice(device_id_ptr)
                .map_err(|e| AudioErrorInner::DeviceNotFound(format!("设备未找到: {}", e)))?
        }
    } else {
        // 使用默认设备
        unsafe {
            enumerator.GetDefaultAudioEndpoint(data_flow, eConsole)
                .map_err(|e| AudioErrorInner::DeviceNotFound(format!("无法获取默认设备: {}", e)))?
        }
    };

    unsafe {
        let endpoint: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None)
            .map_err(|e| AudioErrorInner::VolumeError(format!("无法获取音频端点: {}", e)))?;
        Ok(endpoint)
    }
}

/// 获取音量的内部实现（返回百分比 0-100）
pub fn get_volume_impl(device_id: Option<String>) -> Result<u32, AudioErrorInner> {
    let endpoint = get_audio_endpoint(device_id.as_deref(), eRender)?;

    unsafe {
        let volume = endpoint.GetMasterVolumeLevelScalar()
            .map_err(|e| AudioErrorInner::VolumeError(format!("无法获取音量: {}", e)))?;
        // 将 0.0-1.0 转换为 0-100
        Ok((volume * 100.0).round() as u32)
    }
}

/// 获取音量（百分比形式，0-100）
#[pyfunction]
#[pyo3(signature = (device_id=None))]
pub fn get_volume(_py: Python<'_>, device_id: Option<String>) -> PyResult<u32> {
    get_volume_impl(device_id).map_err(|e| AudioError::new_err(e.to_string()))
}

/// 设置音量的内部实现（参数为百分比 0-100）
pub fn set_volume_impl(volume: u32, device_id: Option<String>) -> Result<(), AudioErrorInner> {
    // 验证音量范围
    if volume > 100 {
        return Err(AudioErrorInner::VolumeError("音量必须在0到100之间".to_string()));
    }

    let endpoint = get_audio_endpoint(device_id.as_deref(), eRender)?;

    // 将 0-100 转换为 0.0-1.0
    let scalar_volume = volume as f32 / 100.0;

    unsafe {
        endpoint.SetMasterVolumeLevelScalar(scalar_volume, std::ptr::null())
            .map_err(|e| AudioErrorInner::VolumeError(format!("无法设置音量: {}", e)))?;
    }

    Ok(())
}

/// 设置音量（百分比形式，0-100）
#[pyfunction]
#[pyo3(signature = (volume, device_id=None))]
pub fn set_volume(_py: Python<'_>, volume: u32, device_id: Option<String>) -> PyResult<()> {
    set_volume_impl(volume, device_id).map_err(|e| AudioError::new_err(e.to_string()))
}

/// 获取静音状态的内部实现
pub fn get_mute_impl(device_id: Option<String>) -> Result<bool, AudioErrorInner> {
    let endpoint = get_audio_endpoint(device_id.as_deref(), eRender)?;

    unsafe {
        let is_muted = endpoint.GetMute()
            .map_err(|e| AudioErrorInner::VolumeError(format!("无法获取静音状态: {}", e)))?
            .as_bool();
        Ok(is_muted)
    }
}

/// 获取静音状态
#[pyfunction]
#[pyo3(signature = (device_id=None))]
pub fn get_mute(_py: Python<'_>, device_id: Option<String>) -> PyResult<bool> {
    get_mute_impl(device_id).map_err(|e| AudioError::new_err(e.to_string()))
}

/// 设置静音状态的内部实现
pub fn set_mute_impl(mute: bool, device_id: Option<String>) -> Result<(), AudioErrorInner> {
    let endpoint = get_audio_endpoint(device_id.as_deref(), eRender)?;

    unsafe {
        endpoint.SetMute(mute, std::ptr::null())
            .map_err(|e| AudioErrorInner::VolumeError(format!("无法设置静音状态: {}", e)))?;
    }

    Ok(())
}

/// 设置静音状态
#[pyfunction]
#[pyo3(signature = (mute, device_id=None))]
pub fn set_mute(_py: Python<'_>, mute: bool, device_id: Option<String>) -> PyResult<()> {
    set_mute_impl(mute, device_id).map_err(|e| AudioError::new_err(e.to_string()))
}

/// 获取输入设备音量的内部实现（返回百分比 0-100）
pub fn get_input_volume_impl(device_id: Option<String>) -> Result<u32, AudioErrorInner> {
    let endpoint = get_audio_endpoint(device_id.as_deref(), eCapture)?;

    unsafe {
        let volume = endpoint.GetMasterVolumeLevelScalar()
            .map_err(|e| AudioErrorInner::VolumeError(format!("无法获取输入音量: {}", e)))?;
        // 将 0.0-1.0 转换为 0-100
        Ok((volume * 100.0).round() as u32)
    }
}

/// 获取输入设备音量（百分比形式，0-100）
#[pyfunction]
#[pyo3(signature = (device_id=None))]
pub fn get_input_volume(_py: Python<'_>, device_id: Option<String>) -> PyResult<u32> {
    get_input_volume_impl(device_id).map_err(|e| AudioError::new_err(e.to_string()))
}

/// 设置输入设备音量的内部实现（参数为百分比 0-100）
pub fn set_input_volume_impl(volume: u32, device_id: Option<String>) -> Result<(), AudioErrorInner> {
    // 验证音量范围
    if volume > 100 {
        return Err(AudioErrorInner::VolumeError("音量必须在0到100之间".to_string()));
    }

    let endpoint = get_audio_endpoint(device_id.as_deref(), eCapture)?;

    // 将 0-100 转换为 0.0-1.0
    let scalar_volume = volume as f32 / 100.0;

    unsafe {
        endpoint.SetMasterVolumeLevelScalar(scalar_volume, std::ptr::null())
            .map_err(|e| AudioErrorInner::VolumeError(format!("无法设置输入音量: {}", e)))?;
    }

    Ok(())
}

/// 设置输入设备音量（百分比形式，0-100）
#[pyfunction]
#[pyo3(signature = (volume, device_id=None))]
pub fn set_input_volume(_py: Python<'_>, volume: u32, device_id: Option<String>) -> PyResult<()> {
    set_input_volume_impl(volume, device_id).map_err(|e| AudioError::new_err(e.to_string()))
}

/// 获取输入设备静音状态的内部实现
pub fn get_input_mute_impl(device_id: Option<String>) -> Result<bool, AudioErrorInner> {
    let endpoint = get_audio_endpoint(device_id.as_deref(), eCapture)?;

    unsafe {
        let is_muted = endpoint.GetMute()
            .map_err(|e| AudioErrorInner::VolumeError(format!("无法获取输入静音状态: {}", e)))?
            .as_bool();
        Ok(is_muted)
    }
}

/// 获取输入设备静音状态
#[pyfunction]
#[pyo3(signature = (device_id=None))]
pub fn get_input_mute(_py: Python<'_>, device_id: Option<String>) -> PyResult<bool> {
    get_input_mute_impl(device_id).map_err(|e| AudioError::new_err(e.to_string()))
}

/// 设置输入设备静音状态的内部实现
pub fn set_input_mute_impl(mute: bool, device_id: Option<String>) -> Result<(), AudioErrorInner> {
    let endpoint = get_audio_endpoint(device_id.as_deref(), eCapture)?;

    unsafe {
        endpoint.SetMute(mute, std::ptr::null())
            .map_err(|e| AudioErrorInner::VolumeError(format!("无法设置输入静音状态: {}", e)))?;
    }

    Ok(())
}

/// 设置输入设备静音状态
#[pyfunction]
#[pyo3(signature = (mute, device_id=None))]
pub fn set_input_mute(_py: Python<'_>, mute: bool, device_id: Option<String>) -> PyResult<()> {
    set_input_mute_impl(mute, device_id).map_err(|e| AudioError::new_err(e.to_string()))
}