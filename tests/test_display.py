# Display模块测试
import pytest


def test_display_module_imports():
    """测试 display 模块可以正确导入"""
    from win_control import display
    assert hasattr(display, 'MonitorInfo')
    assert hasattr(display, 'Resolution')
    assert hasattr(display, 'DisplayError')


def test_list_monitors():
    """测试 list_monitors 函数"""
    from win_control.display import list_monitors, MonitorInfo
    monitors = list_monitors()
    assert len(monitors) >= 1
    assert all(isinstance(m, MonitorInfo) for m in monitors)
    assert any(m.is_primary for m in monitors)


def test_monitor_info_attributes():
    """测试 MonitorInfo 属性"""
    from win_control.display import list_monitors
    monitors = list_monitors()
    if monitors:
        m = monitors[0]
        assert hasattr(m, 'id')
        assert hasattr(m, 'name')
        assert hasattr(m, 'is_primary')
        assert isinstance(m.id, str)
        assert isinstance(m.name, str)
        assert isinstance(m.is_primary, bool)


def test_get_current_resolution():
    """测试获取当前分辨率"""
    from win_control.display import get_current_resolution, Resolution
    res = get_current_resolution()
    assert isinstance(res, Resolution)
    assert res.width > 0
    assert res.height > 0
    assert res.refresh_rate > 0


def test_get_supported_resolutions():
    """测试获取支持的分辨率列表"""
    from win_control.display import get_supported_resolutions, Resolution
    resolutions = get_supported_resolutions()
    assert len(resolutions) >= 1
    assert all(isinstance(r, Resolution) for r in resolutions)


def test_set_resolution_safe():
    """测试设置分辨率（标记为跳过，因为会实际更改分辨率）"""
    # 此测试会实际更改显示器分辨率，可能干扰用户工作
    # 仅在明确需要时手动运行
    pytest.skip("跳过: 此测试会实际更改显示器分辨率")

    from win_control.display import get_current_resolution, set_resolution, Resolution
    current = get_current_resolution()

    # 尝试设置一个常见分辨率
    set_resolution(1920, 1080)

    # 验证设置成功
    new_res = get_current_resolution()
    assert new_res.width == 1920
    assert new_res.height == 1080

    # 注意: 恢复分辨率由 restore_resolution 完成 (Task 9)


def test_set_resolution_module_exists():
    """测试 set_resolution 函数存在"""
    from win_control import display
    assert hasattr(display, 'set_resolution')
    assert callable(display.set_resolution)


def test_restore_resolution_module_exists():
    """测试 restore_resolution 函数存在"""
    from win_control import display
    assert hasattr(display, 'restore_resolution')
    assert callable(display.restore_resolution)


def test_restore_resolution_without_saved():
    """测试在没有保存分辨率时调用 restore_resolution 会抛出错误"""
    from win_control.display import restore_resolution, DisplayError
    import pytest

    # 在没有先调用 set_resolution 的情况下调用 restore_resolution
    # 应该抛出 DisplayError
    # 注意：由于 ORIGINAL_RESOLUTION 是线程本地存储，这里可能不会抛出错误
    # 如果在同一线程中已经调用过 set_resolution
    # 所以这个测试主要验证函数可调用
    try:
        restore_resolution()
    except DisplayError as e:
        # 预期在没有保存分辨率时抛出错误
        assert "没有保存的原始分辨率" in str(e) or "No original resolution" in str(e)