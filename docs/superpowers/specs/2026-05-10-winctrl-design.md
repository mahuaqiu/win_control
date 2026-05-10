---
name: winctrl-design
description: Rust Python 模块用于 Windows 系统控制（分辨率、音量、音频设备管理）
type: project
---

# WinCtrl - Windows 系统控制 Python 模块设计文档

## 概述

WinCtrl 是一个使用 Rust 实现的 Python 模块，用于控制 Windows 系统功能。当前版本包含以下功能域：

- **Display 模块**：显示器分辨率管理
- **Audio 模块**：音频设备与音量管理

**技术路线**：基于 `windows` crate（Microsoft 官方 Win32 API 绑定）全部手动实现，通过 PyO3 提供 Python 接口。

---

## 项目结构

```
winctrl/
├── Cargo.toml
├── pyproject.toml
├── src/
│   ├── lib.rs                 # PyO3 模块入口
│   ├── display/
│   │   ├── mod.rs             # display 子模块导出
│   │   ├── monitor.rs         # 显示器枚举与分辨率控制
│   │   └── error.rs           # 显示器相关异常
│   ├── audio/
│   │   ├── mod.rs             # audio 子模块导出
│   │   ├── device.rs          # 音频设备枚举与管理
│   │   ├── volume.rs          # 音量控制
│   │   └── error.rs           # 音频相关异常
│   └── error.rs               # 公共异常基类
└── tests/
    └── test_winctrl.py        # Python 测试
```

---

## Display 模块

### Python API

```python
from winctrl.display import (
    # 异常类
    DisplayError,           # 基类
    MonitorNotFoundError,   # 指定显示器不存在
    ResolutionNotSupportedError,  # 分辨率不支持
    DisplayPermissionError, # 权限不足

    # 函数
    list_monitors,          # -> List[MonitorInfo]
    get_current_resolution, # (monitor_id: Optional[str] = None) -> Resolution
    get_supported_resolutions, # (monitor_id: Optional[str] = None) -> List[Resolution]
    set_resolution,         # (width: int, height: int, monitor_id: Optional[str] = None) -> None
    restore_resolution,     # (monitor_id: Optional[str] = None) -> None
                            # monitor_id=None 时恢复主显示器分辨率
)

# 数据类型
@dataclass
class MonitorInfo:
    id: str                 # 显示器设备 ID
    name: str               # 显示器名称
    is_primary: bool        # 是否主显示器

@dataclass
class Resolution:
    width: int
    height: int
    refresh_rate: int       # 刷新率 (Hz)
```

### 底层实现

| 功能 | Win32 API |
|------|-----------|
| 枚举显示器 | `EnumDisplayDevicesW` |
| 获取支持的分辨率 | `EnumDisplaySettingsExW` |
| 设置分辨率 | `ChangeDisplaySettingsExW` |

**实现要点**：
- 使用 `DISPLAY_DEVICE` 结构获取显示器信息
- 使用 `DEVMODEW` 结构存储分辨率设置
- 内部维护原始分辨率状态，用于 `restore_resolution`

---

## Audio 模块

### Python API

```python
from winctrl.audio import (
    # 异常类
    AudioError,             # 基类
    DeviceNotFoundError,    # 指定设备不存在
    DeviceDisabledError,    # 设备已禁用
    AudioPermissionError,   # 权限不足

    # 函数
    list_devices,           # (device_type: str = "all") -> List[DeviceInfo]
                            # device_type 可选值: "all", "speaker", "microphone"
    get_device_state,       # (device_id: str) -> DeviceState
    get_default_device,     # (role: str = "console") -> DeviceInfo
    set_default_device,     # (device_id: str, role: str = "console") -> None
    get_volume,             # (device_id: Optional[str] = None) -> float  # 0.0-1.0
    set_volume,             # (volume: float, device_id: Optional[str] = None) -> None
    get_mute,               # (device_id: Optional[str] = None) -> bool
    set_mute,               # (mute: bool, device_id: Optional[str] = None) -> None
    enable_device,          # (device_id: str) -> None
    disable_device,         # (device_id: str) -> None
)

# 数据类型
@dataclass
class DeviceInfo:
    id: str                 # 音频设备 ID
    name: str               # 设备名称
    type: str               # "speaker" / "microphone"
    state: str              # "active" / "disabled" / "unplugged"

@dataclass
class DeviceState:
    state: str              # 设备状态
    is_default: bool        # 是否默认设备
    volume: float           # 当前音量 (0.0-1.0)
    is_muted: bool          # 是否静音
```

### 底层实现

| 功能 | Windows API |
|------|-------------|
| 枚举音频设备 | Core Audio: `IMMDeviceEnumerator::EnumAudioEndpoints` |
| 获取设备状态 | `IMMDevice::GetState` |
| 音量控制 | `IAudioEndpointVolume` (GetMasterVolumeLevelScalar, SetMasterVolumeLevelScalar) |
| 静音控制 | `IAudioEndpointVolume::GetMute`, `SetMute` |
| 设置默认设备 | `IPolicyConfig::SetDefaultEndpoint` |
| 禁用/启用设备 | `IPolicyConfig::SetEndpointVisibility` |

**COM 接口定义**：

`IPolicyConfig` 是非公开的 COM 接口，需要手动定义。参考 audioswitch 项目实现：

```rust
// IPolicyConfig GUID
// Interface: {f8679f50-850a-41cf-9c72-430f290290c8}
// Class: {870af99c-171d-4f9e-af0d-e63df40c2bc9}
```

**实现要点**：
- 初始化 COM: `CoCreateInstance` 创建 `MMDeviceEnumerator` 和 `PolicyConfigClient`
- 设备类型过滤: `eRender` (扬声器) / `eCapture` (麦克风)
- Role 参数: `eConsole` / `eMultimedia` / `eCommunications`

---

## 依赖清单

```toml
[package]
name = "winctrl"
version = "0.1.0"
edition = "2021"

[lib]
name = "winctrl"
crate-type = ["cdylib"]

[dependencies]
pyo3 = { version = "0.22", features = ["extension-module"] }
windows = { version = "0.58", features = [
    "Win32_Foundation",
    "Win32_UI_HiDpi",
    "Win32_Devices_Display",
    "Win32_Media_Audio",
    "Win32_Media_Audio_Endpoints",
    "Win32_System_Com",
    "Win32_System_Com_StructuredStorage",
    "Win32_System_Ole",
]}
```

---

## 错误处理策略

所有错误通过 Python 异常体系处理：

| 异常类型 | 触发场景 |
|----------|----------|
| `DisplayError` | Display 模块基础异常 |
| `MonitorNotFoundError` | 指定 monitor_id 不存在 |
| `ResolutionNotSupportedError` | 请求的分辨率不在支持列表中 |
| `DisplayPermissionError` | 权限不足或 API 调用失败 |
| `AudioError` | Audio 模块基础异常 |
| `DeviceNotFoundError` | 指定 device_id 不存在 |
| `DeviceDisabledError` | 操作需要设备启用但设备已禁用 |
| `AudioPermissionError` | COM 初始化失败或 API 调用失败 |

---

## 构建与使用

### 开发构建

```bash
# 安装 maturin
pip install maturin

# 开发模式构建（自动安装到当前 Python 环境）
maturin develop
```

### 发布构建

```bash
maturin build --release
```

### 使用示例

```python
# Display 模块
from winctrl.display import list_monitors, set_resolution, get_supported_resolutions

monitors = list_monitors()
print(f"主显示器: {monitors[0].name}")

resolutions = get_supported_resolutions()
for r in resolutions:
    print(f"{r.width}x{r.height} @ {r.refresh_rate}Hz")

set_resolution(1920, 1080)

# Audio 模块
from winctrl.audio import list_devices, set_volume, disable_device, get_default_device

devices = list_devices("speaker")
for d in devices:
    print(f"{d.name}: {d.state}")

default = get_default_device()
print(f"默认扬声器: {default.name}")

set_volume(0.5)  # 50% 音量
disable_device("设备ID")
```

---

## 系统兼容性

- **操作系统**: Windows 10 / Windows 11
- **Python 版本**: 3.8+
- **架构**: x64 (暂不支持 x86)

---

## 参考

- audioswitch C++ 实现: `D:\code\audioswitch-master\audioswitch-master\IPolicyConfig.h`
- Windows-rs 文档: https://microsoft.github.io/windows-rs/
- PyO3 文档: https://pyo3.rs/