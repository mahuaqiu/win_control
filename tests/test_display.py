# Display模块测试
import pytest


def test_display_module_imports():
    """测试 display 模块可以正确导入"""
    from winctrl import display
    assert hasattr(display, 'MonitorInfo')
    assert hasattr(display, 'Resolution')
    assert hasattr(display, 'DisplayError')


def test_list_monitors():
    """测试 list_monitors 函数"""
    from winctrl.display import list_monitors, MonitorInfo
    monitors = list_monitors()
    assert len(monitors) >= 1
    assert all(isinstance(m, MonitorInfo) for m in monitors)
    assert any(m.is_primary for m in monitors)


def test_monitor_info_attributes():
    """测试 MonitorInfo 属性"""
    from winctrl.display import list_monitors
    monitors = list_monitors()
    if monitors:
        m = monitors[0]
        assert hasattr(m, 'id')
        assert hasattr(m, 'name')
        assert hasattr(m, 'is_primary')
        assert isinstance(m.id, str)
        assert isinstance(m.name, str)
        assert isinstance(m.is_primary, bool)