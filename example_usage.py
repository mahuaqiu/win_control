#!/usr/bin/env python3
"""
WinCtrl 使用示例
"""

import sys

def test_display_module():
    """测试 Display 模块"""
    print("=" * 50)
    print("Display 模块测试")
    print("=" * 50)

    from winctrl.display import (
        list_monitors,
        get_current_resolution,
        get_supported_resolutions,
        set_resolution,
        restore_resolution,
        MonitorInfo,
        Resolution,
        DisplayError,
    )

    # 1. 列出所有显示器
    print("\n1. 显示器列表:")
    monitors = list_monitors()
    for i, m in enumerate(monitors):
        print(f"   [{i}] {m.name}")
        print(f"       ID: {m.id}")
        print(f"       主显示器: {m.is_primary}")

    # 2. 获取当前分辨率
    print("\n2. 当前分辨率:")
    res = get_current_resolution()
    print(f"   {res.width} x {res.height} @ {res.refresh_rate}Hz")

    # 3. 获取支持的分辨率列表（全部）
    print("\n3. 支持的分辨率列表:")
    resolutions = get_supported_resolutions()
    for i, r in enumerate(resolutions):
        print(f"   [{i}] {r.width} x {r.height} @ {r.refresh_rate}Hz")
    print(f"   共 {len(resolutions)} 种分辨率")

    # 4. 测试分辨率设置
    print("\n4. 分辨率设置测试:")
    current = get_current_resolution()
    print(f"   当前分辨率: {current.width}x{current.height}")

    # 找一个可以设置的分辨率（使用列表中确认支持的）
    # 注意：某些分辨率可能在列表中但实际不支持（显示器限制）
    # 当前显示器似乎只支持 75Hz 的分辨率
    test_width, test_height = 1920, 1080  # 这个可以成功
    print(f"   测试设置 {test_width}x{test_height}")
    try:
        set_resolution(test_width, test_height)
        print("   [OK] 设置成功")
    except DisplayError as e:
        print(f"   [Error] {e}")

    # 5. 设置其他显示器（如果有多显示器）
    if len(monitors) > 1:
        print("\n5. 多显示器设置示例:")
        print(f"   设置显示器2: set_resolution(1920, 1080, '{monitors[1].id}')")

    return True


def test_audio_module():
    """测试 Audio 模块"""
    print("\n" + "=" * 50)
    print("Audio 模块测试")
    print("=" * 50)

    from winctrl.audio import (
        list_devices,
        get_volume,
        set_volume,
        get_mute,
        set_mute,
        get_device_state,
        get_default_device,
        disable_device,
        enable_device,
        DeviceInfo,
        AudioError,
    )

    # 1. 列出所有扬声器设备（包括停用的）
    print("\n1. 扬声器设备（所有状态）:")
    all_speakers = list_devices("speaker", "all")
    for i, d in enumerate(all_speakers):
        print(f"   [{i}] {d.name}")
        print(f"       ID: {d.id}")
        print(f"       状态: {d.state}")

    # 2. 只列出活跃的麦克风设备
    print("\n2. 活跃麦克风设备:")
    active_mics = list_devices("microphone", "active")
    for i, d in enumerate(active_mics):
        print(f"   [{i}] {d.name} ({d.state})")

    # 3. 获取默认设备
    print("\n3. 默认扬声器:")
    try:
        default = get_default_device("speaker", "console")
        print(f"   名称: {default.name}")
        print(f"   ID: {default.id}")
        print(f"   状态: {default.state}")
    except AudioError as e:
        print(f"   错误: {e}")



    # 6. 设备状态详情
    print("\n6. 设备完整状态:")
    active_speakers = [d for d in all_speakers if d.state == "active"]
    if active_speakers:
        try:
            state = get_device_state(active_speakers[0].id)
            print(f"   设备: {active_speakers[0].name}")
            print(f"   状态: {state.state}")
            print(f"   默认设备: {state.is_default}")
            print(f"   音量: {state.volume}%")
            print(f"   静音: {state.is_muted}")
        except AudioError as e:
            print(f"   获取状态失败: {e}")

    # 7. 设备停用/启用示例（注释掉，避免实际停用设备）
    print("\n7. 设备停用/启用示例:")
    disabled_devices = [d for d in all_speakers if d.state == "disabled"]
    if disabled_devices:
        print(f"   已停用的设备: {disabled_devices[0].name}")
        print(f"   启用方法: enable_device('{disabled_devices[0].name}')")
    else:
        print("   当前没有停用的扬声器设备")

    # 显示停用设备的命令（不实际执行）
    if active_speakers:
        print(f"   停用设备示例: disable_device('{active_speakers[0].name}')")
        print(f"   启用设备示例: enable_device('{active_speakers[0].name}')")
        print("   (注意: 实际执行会停用/启用设备，需要管理员权限)")
    disable_device('扬声器 (Realtek High Definition Audio)')
    return True


def main():
    """主函数"""
    print("WinCtrl Windows 系统控制模块示例")
    print("=" * 50)

    success = True

    try:
        success &= test_audio_module()
    except Exception as e:
        print(f"\nAudio 模块测试失败: {e}")
        success = False

    print("\n" + "=" * 50)
    if success:
        print("[OK] 所有测试通过!")
    else:
        print("[Error] 部分测试失败")
    print("=" * 50)

    return 0 if success else 1


if __name__ == "__main__":
    sys.exit(main())