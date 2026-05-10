# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

win_control 是一个 Windows 系统控制模块，使用 Rust + PyO3 构建 Python 绑定。提供显示器控制（分辨率管理）和音频设备控制（设备枚举、音量调节、设备启用/禁用）功能。

## 构建和开发命令

```bash
# 构建并安装（开发模式）
maturin develop

# 构建发布版本 wheel
maturin build --release

# 运行测试（需要先 maturin develop）
pytest tests/

# 运行单个测试文件
pytest tests/test_audio.py -v
pytest tests/test_display.py -v

# 运行示例脚本
python example_usage.py
```

## 架构

```
src/
├── lib.rs           # PyO3 模块入口，注册 display/audio 子模块到 Python
├── error.rs         # 顶层异常 WinCtrlError
├── display/
│   ├── mod.rs       # 模块注册，导出 DisplayError、MonitorInfo、Resolution 类和函数
│   ├── monitor.rs   # 核心实现：list_monitors、get_current_resolution、set_resolution 等
│   └── error.rs     # DisplayError、MonitorNotFoundError、ResolutionNotSupportedError
└── audio/
│   ├── mod.rs       # 模块注册，导出 AudioError、DeviceInfo、DeviceState 类和函数
│   ├── device.rs    # 设备枚举、状态获取、启用/禁用、默认设备管理
│   ├── volume.rs    # 音量和静音控制（输出和输入设备）
│   ├── policy_config.rs  # IPolicyConfig COM 接口（用于设备启用/禁用）
│   └── error.rs     # AudioError、DeviceNotFoundError、DeviceDisabledError
```

## API 模块

### win_control.display
- `list_monitors()` → `MonitorInfo` 列表（每个 MonitorInfo 包含 `index` 属性）
- `get_current_resolution(monitor_index=None)` → `Resolution`
- `get_supported_resolutions(monitor_index=None)` → `Resolution` 列表
- `set_resolution(width, height, monitor_index=None)` → 设置分辨率
- `restore_resolution(monitor_index=None)` → 恢复原始分辨率

**monitor_index 参数说明：**
- `None` 或 `0` = 主显示器
- `1` = 第二个显示器（list_monitors 返回列表的索引 1）
- `2` = 第三个显示器...

### win_control.audio
- `list_devices(device_type="all", state_filter="all")` → `DeviceInfo` 列表
  - device_type: "speaker"/"render", "microphone"/"capture", "all"
  - state_filter: "active", "disabled", "unplugged", "all"
- `get_device_state(device_name_or_id)` → `DeviceState`（支持设备名称或ID）
- `get_volume()`, `set_volume(level)` → 输出设备音量 (0.0-1.0)
- `get_mute()`, `set_mute(bool)` → 输出设备静音
- `get_input_volume()`, `set_input_volume(level)` → 输入设备音量
- `get_input_mute()`, `set_input_mute(bool)` → 输入设备静音
- `enable_device(device_name_or_id)`, `disable_device(device_name_or_id)` → 设备启用/禁用
- `get_default_device(device_type="speaker", role="console")`, `set_default_device(device_id, role="console")` → 默认设备管理

## 依赖

- Rust: pyo3 (extension-module), windows crate (Win32 API)
- Python: maturin (构建后端), pytest (测试)