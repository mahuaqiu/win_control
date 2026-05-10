use windows::core::{GUID, PCWSTR, Interface, IUnknown, IUnknown_Vtbl};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL, COINIT_MULTITHREADED, CoInitializeEx};
use windows::core::HRESULT;

/// IPolicyConfig 接口的 IID
/// {f8679f50-850a-41cf-9c72-430f290290c8}
pub const IID_IPolicyConfig: GUID = GUID::from_u128(0xf8679f50_850a_41cf_9c72_430f290290c8);

/// CPolicyConfigClient 的 CLSID
/// {870af99c-171d-4f9e-af0d-e63df40c2bc9}
pub const CLSID_CPolicyConfigClient: GUID = GUID::from_u128(0x870af99c_171d_4f9e_af0d_e63df40c2bc9);

/// ERole 枚举值 - 定义音频设备的角色
/// 在 Windows API 中，ERole 是一个整数枚举
pub const EROLE_CONSOLE: i32 = 0;         // eConsole - 控制台设备
pub const EROLE_MULTIMEDIA: i32 = 1;      // eMultimedia - 多媒体设备
pub const EROLE_COMMUNICATIONS: i32 = 2;  // eCommunications - 通信设备

/// IPolicyConfig vtable
#[repr(C)]
pub struct IPolicyConfig_Vtbl {
    pub base: IUnknown_Vtbl,
    pub GetMixFormat: unsafe extern "system" fn(this: *mut core::ffi::c_void, device_id: PCWSTR, format: *mut *mut windows::Win32::Media::Audio::WAVEFORMATEX) -> HRESULT,
    pub GetDeviceFormat: unsafe extern "system" fn(this: *mut core::ffi::c_void, device_id: PCWSTR, _default: i32, format: *mut *mut windows::Win32::Media::Audio::WAVEFORMATEX) -> HRESULT,
    pub ResetDeviceFormat: unsafe extern "system" fn(this: *mut core::ffi::c_void, device_id: PCWSTR) -> HRESULT,
    pub SetDeviceFormat: unsafe extern "system" fn(this: *mut core::ffi::c_void, device_id: PCWSTR, format: *const windows::Win32::Media::Audio::WAVEFORMATEX, closest_match: *const windows::Win32::Media::Audio::WAVEFORMATEX) -> HRESULT,
    pub GetProcessingPeriod: unsafe extern "system" fn(this: *mut core::ffi::c_void, device_id: PCWSTR, _default: i32, period: *mut i64, min_period: *mut i64) -> HRESULT,
    pub SetProcessingPeriod: unsafe extern "system" fn(this: *mut core::ffi::c_void, device_id: PCWSTR, period: *const i64) -> HRESULT,
    pub GetShareMode: unsafe extern "system" fn(this: *mut core::ffi::c_void, device_id: PCWSTR, mode: *mut core::ffi::c_void) -> HRESULT,
    pub SetShareMode: unsafe extern "system" fn(this: *mut core::ffi::c_void, device_id: PCWSTR, mode: *const core::ffi::c_void) -> HRESULT,
    pub GetPropertyValue: unsafe extern "system" fn(this: *mut core::ffi::c_void, device_id: PCWSTR, key: *const core::ffi::c_void, value: *mut core::ffi::c_void) -> HRESULT,
    pub SetPropertyValue: unsafe extern "system" fn(this: *mut core::ffi::c_void, device_id: PCWSTR, key: *const core::ffi::c_void, value: *mut core::ffi::c_void) -> HRESULT,
    pub SetDefaultEndpoint: unsafe extern "system" fn(this: *mut core::ffi::c_void, device_id: PCWSTR, role: i32) -> HRESULT,
    pub SetEndpointVisibility: unsafe extern "system" fn(this: *mut core::ffi::c_void, device_id: PCWSTR, visibility: i32) -> HRESULT,
}

/// IPolicyConfig COM 接口
/// 这是一个非公开的 Windows COM 接口，用于设置默认音频端点和设备可见性
#[repr(transparent)]
#[derive(Clone)]
pub struct IPolicyConfig(IUnknown);

unsafe impl Interface for IPolicyConfig {
    const IID: GUID = IID_IPolicyConfig;
    type Vtable = IPolicyConfig_Vtbl;
}

impl IPolicyConfig {
    /// 设置默认音频端点
    ///
    /// # 参数
    /// * `device_id` - 设备ID（宽字符串）
    /// * `role` - 设备角色（EROLE_CONSOLE, EROLE_MULTIMEDIA, EROLE_COMMUNICATIONS）
    ///
    /// # 返回值
    /// 成功返回 Ok(())，失败返回错误
    pub fn set_default_endpoint(&self, device_id: PCWSTR, role: i32) -> windows::core::Result<()> {
        unsafe {
            let vtable = self.vtable();
            ((*vtable).SetDefaultEndpoint)(core::mem::transmute_copy(&self.0), device_id, role)
                .ok()
        }
    }

    /// 设置设备可见性（启用/禁用设备）
    ///
    /// # 参数
    /// * `device_id` - 设备ID（宽字符串）
    /// * `visibility` - 可见性（1 = 启用，0 = 禁用）
    ///
    /// # 返回值
    /// 成功返回 Ok(())，失败返回错误
    pub fn set_endpoint_visibility(&self, device_id: PCWSTR, visibility: i32) -> windows::core::Result<()> {
        unsafe {
            let vtable = self.vtable();
            ((*vtable).SetEndpointVisibility)(core::mem::transmute_copy(&self.0), device_id, visibility)
                .ok()
        }
    }

    /// 获取设备格式
    ///
    /// # 参数
    /// * `device_id` - 设备ID（宽字符串）
    /// * `_default` - 是否使用默认格式
    /// * `format` - 输出的 WAVEFORMATEX 结构指针
    pub fn get_device_format(&self, device_id: PCWSTR, _default: i32, format: *mut *mut windows::Win32::Media::Audio::WAVEFORMATEX) -> windows::core::Result<()> {
        unsafe {
            let vtable = self.vtable();
            ((*vtable).GetDeviceFormat)(core::mem::transmute_copy(&self.0), device_id, _default, format)
                .ok()
        }
    }

    /// 设置设备格式
    ///
    /// # 参数
    /// * `device_id` - 设备ID（宽字符串）
    /// * `format` - 要设置的 WAVEFORMATEX 结构
    /// * `closest_match` - 最接近的匹配格式
    pub fn set_device_format(&self, device_id: PCWSTR, format: *const windows::Win32::Media::Audio::WAVEFORMATEX, closest_match: *const windows::Win32::Media::Audio::WAVEFORMATEX) -> windows::core::Result<()> {
        unsafe {
            let vtable = self.vtable();
            ((*vtable).SetDeviceFormat)(core::mem::transmute_copy(&self.0), device_id, format, closest_match)
                .ok()
        }
    }

    /// 重置设备格式
    ///
    /// # 参数
    /// * `device_id` - 设备ID（宽字符串）
    pub fn reset_device_format(&self, device_id: PCWSTR) -> windows::core::Result<()> {
        unsafe {
            let vtable = self.vtable();
            ((*vtable).ResetDeviceFormat)(core::mem::transmute_copy(&self.0), device_id)
                .ok()
        }
    }
}

/// 创建 IPolicyConfig 实例
///
/// # 返回值
/// 成功返回 IPolicyConfig 接口，失败返回错误
///
/// # 示例
/// ```rust,ignore
/// let policy_config = create_policy_config()?;
/// policy_config.set_default_endpoint(device_id, EROLE_CONSOLE)?;
/// ```
pub fn create_policy_config() -> windows::core::Result<IPolicyConfig> {
    unsafe {
        // 初始化 COM（MTA 模式）
        // 注意：如果 COM 已经初始化，CoInitializeEx 会返回 S_FALSE，这在 windows-rs 中被视为成功
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

        // 创建 CPolicyConfigClient 实例
        let instance: IUnknown = CoCreateInstance(&CLSID_CPolicyConfigClient, None, CLSCTX_ALL)?;

        // 查询 IPolicyConfig 接口
        instance.cast::<IPolicyConfig>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guid_values() {
        // 验证 GUID 值正确
        assert_eq!(
            IID_IPolicyConfig,
            GUID::from_u128(0xf8679f50_850a_41cf_9c72_430f290290c8)
        );
        assert_eq!(
            CLSID_CPolicyConfigClient,
            GUID::from_u128(0x870af99c_171d_4f9e_af0d_e63df40c2bc9)
        );
    }

    #[test]
    fn test_interface_iid() {
        // 验证 Interface trait 的 IID 实现
        assert_eq!(IPolicyConfig::IID, IID_IPolicyConfig);
    }

    #[test]
    fn test_erole_constants() {
        // 验证 ERole 常量值
        assert_eq!(EROLE_CONSOLE, 0);
        assert_eq!(EROLE_MULTIMEDIA, 1);
        assert_eq!(EROLE_COMMUNICATIONS, 2);
    }
}