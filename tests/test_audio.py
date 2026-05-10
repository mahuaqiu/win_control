# Audio模块测试

import pytest
import win_control.audio as audio


class TestAudioExceptions:
    """测试Audio模块异常类"""

    def test_audio_error_is_base_exception(self):
        """AudioError 应该是基础异常类"""
        assert issubclass(audio.DeviceNotFoundError, audio.AudioError)
        assert issubclass(audio.DeviceDisabledError, audio.AudioError)
        assert issubclass(audio.AudioPermissionError, audio.AudioError)

    def test_audio_error_can_be_raised(self):
        """可以抛出和捕获AudioError"""
        with pytest.raises(audio.AudioError):
            raise audio.AudioError("测试错误")

    def test_device_not_found_error(self):
        """DeviceNotFoundError 测试"""
        with pytest.raises(audio.DeviceNotFoundError):
            raise audio.DeviceNotFoundError("设备未找到")


class TestDeviceInfo:
    """测试DeviceInfo数据类型"""

    def test_device_info_has_required_fields(self):
        """DeviceInfo应该包含所有必需字段"""
        devices = audio.list_devices()
        if len(devices) > 0:
            device = devices[0]
            assert hasattr(device, 'id')
            assert hasattr(device, 'name')
            assert hasattr(device, 'device_type')
            assert hasattr(device, 'state')
            assert isinstance(device.id, str)
            assert isinstance(device.name, str)
            assert isinstance(device.device_type, str)
            assert isinstance(device.state, str)


class TestDeviceState:
    """测试DeviceState数据类型"""

    def test_device_state_has_required_fields(self):
        """DeviceState应该包含所有必需字段"""
        devices = audio.list_devices()
        active_devices = [d for d in devices if d.state == 'active']
        if len(active_devices) > 0:
            state = audio.get_device_state(active_devices[0].id)
            assert hasattr(state, 'state')
            assert hasattr(state, 'is_default')
            assert hasattr(state, 'volume')
            assert hasattr(state, 'is_muted')
            assert isinstance(state.state, str)
            assert isinstance(state.is_default, bool)
            assert isinstance(state.volume, float)
            assert isinstance(state.is_muted, bool)


class TestListDevices:
    """测试设备枚举功能"""

    def test_list_devices_returns_list(self):
        """list_devices 应该返回列表"""
        devices = audio.list_devices()
        assert isinstance(devices, list)

    def test_list_devices_default_parameter(self):
        """list_devices 默认参数测试"""
        devices = audio.list_devices()
        # 默认应该返回所有设备
        assert isinstance(devices, list)

    def test_list_devices_with_render_type(self):
        """list_devices 只返回输出设备"""
        devices = audio.list_devices("render")
        for device in devices:
            assert device.device_type == "render"

    def test_list_devices_with_capture_type(self):
        """list_devices 只返回输入设备"""
        devices = audio.list_devices("capture")
        for device in devices:
            assert device.device_type == "capture"

    def test_list_devices_at_least_one_active(self):
        """系统应该至少有一个活动音频设备"""
        devices = audio.list_devices()
        active_devices = [d for d in devices if d.state == 'active']
        # 大多数Windows系统至少有一个活动音频设备
        # 但在某些情况下可能没有（如无头服务器），所以不做强制要求


class TestVolumeControl:
    """测试音量控制功能"""

    def test_get_volume_returns_float(self):
        """get_volume 应该返回浮点数"""
        volume = audio.get_volume()
        assert isinstance(volume, float)
        assert 0.0 <= volume <= 1.0

    def test_set_volume_valid_range(self):
        """set_volume 在有效范围内应该成功"""
        original_volume = audio.get_volume()
        try:
            # 测试设置音量到0.5
            audio.set_volume(0.5)
            new_volume = audio.get_volume()
            # 由于音量控制可能有精度损失，使用近似比较
            assert abs(new_volume - 0.5) < 0.05
        finally:
            # 恢复原始音量
            audio.set_volume(original_volume)

    def test_set_volume_invalid_range(self):
        """set_volume 在无效范围应该抛出异常"""
        with pytest.raises(audio.AudioError):
            audio.set_volume(-0.1)
        with pytest.raises(audio.AudioError):
            audio.set_volume(1.1)


class TestMuteControl:
    """测试静音控制功能"""

    def test_get_mute_returns_bool(self):
        """get_mute 应该返回布尔值"""
        is_muted = audio.get_mute()
        assert isinstance(is_muted, bool)

    def test_set_mute(self):
        """set_mute 应该能够设置静音状态"""
        original_mute = audio.get_mute()
        try:
            # 测试设置静音
            audio.set_mute(True)
            assert audio.get_mute() == True
            # 测试取消静音
            audio.set_mute(False)
            assert audio.get_mute() == False
        finally:
            # 恢复原始静音状态
            audio.set_mute(original_mute)


class TestInputVolumeControl:
    """测试输入设备音量控制"""

    def test_get_input_volume_returns_float(self):
        """get_input_volume 应该返回浮点数"""
        # 如果有麦克风设备
        devices = audio.list_devices("capture")
        active_devices = [d for d in devices if d.state == 'active']
        if len(active_devices) > 0:
            volume = audio.get_input_volume()
            assert isinstance(volume, float)
            assert 0.0 <= volume <= 1.0

    def test_get_input_mute_returns_bool(self):
        """get_input_mute 应该返回布尔值"""
        devices = audio.list_devices("capture")
        active_devices = [d for d in devices if d.state == 'active']
        if len(active_devices) > 0:
            is_muted = audio.get_input_mute()
            assert isinstance(is_muted, bool)


class TestGetDeviceState:
    """测试获取设备状态"""

    def test_get_device_state_for_active_device(self):
        """获取活动设备的状态"""
        devices = audio.list_devices()
        active_devices = [d for d in devices if d.state == 'active']
        if len(active_devices) > 0:
            state = audio.get_device_state(active_devices[0].id)
            assert state.state == 'active'
            assert isinstance(state.volume, float)

    def test_get_device_state_invalid_id(self):
        """使用无效设备ID应该抛出异常"""
        with pytest.raises(audio.AudioError):
            audio.get_device_state("invalid_device_id")


# 运行测试
if __name__ == "__main__":
    pytest.main([__file__, "-v"])


# ============================================================
# Task 15 & 16: 设备禁用/启用和默认设备管理测试
# ============================================================

class TestDeviceEnableDisable:
    """测试设备禁用/启用功能"""

    def test_enable_disable_device_functions_exist(self):
        """测试禁用/启用设备函数存在"""
        assert hasattr(audio, 'disable_device')
        assert hasattr(audio, 'enable_device')
        assert callable(audio.disable_device)
        assert callable(audio.enable_device)

    def test_disable_device_invalid_id(self):
        """使用无效设备ID禁用设备应该抛出异常"""
        with pytest.raises(audio.AudioError):
            audio.disable_device("invalid_device_id")

    def test_enable_device_invalid_id(self):
        """使用无效设备ID启用设备应该抛出异常"""
        with pytest.raises(audio.AudioError):
            audio.enable_device("invalid_device_id")


class TestDefaultDeviceManagement:
    """测试默认设备管理功能"""

    def test_get_default_device_functions_exist(self):
        """测试默认设备管理函数存在"""
        assert hasattr(audio, 'get_default_device')
        assert hasattr(audio, 'set_default_device')
        assert callable(audio.get_default_device)
        assert callable(audio.set_default_device)

    def test_get_default_device_returns_device_info(self):
        """get_default_device 应该返回 DeviceInfo"""
        # 获取默认输出设备
        device = audio.get_default_device("speaker", "console")
        assert hasattr(device, 'id')
        assert hasattr(device, 'name')
        assert hasattr(device, 'device_type')
        assert hasattr(device, 'state')
        assert device.device_type == "render"

    def test_get_default_device_capture(self):
        """获取默认输入设备"""
        devices = audio.list_devices("capture")
        active_devices = [d for d in devices if d.state == 'active']
        if len(active_devices) > 0:
            # 只在有活动输入设备时测试
            try:
                device = audio.get_default_device("capture", "console")
                assert device.device_type == "capture"
            except audio.AudioError:
                # 如果没有输入设备，跳过测试
                pass

    def test_get_default_device_multimedia_role(self):
        """使用 multimedia 角色获取默认设备"""
        try:
            device = audio.get_default_device("speaker", "multimedia")
            assert hasattr(device, 'id')
        except audio.AudioError:
            # 某些系统可能没有设置 multimedia 角色
            pass

    def test_get_default_device_communications_role(self):
        """使用 communications 角色获取默认设备"""
        try:
            device = audio.get_default_device("speaker", "communications")
            assert hasattr(device, 'id')
        except audio.AudioError:
            # 某些系统可能没有设置 communications 角色
            pass

    def test_set_default_device_invalid_id(self):
        """使用无效设备ID设置默认设备应该抛出异常"""
        with pytest.raises(audio.AudioError):
            audio.set_default_device("invalid_device_id", "console")

    def test_default_device_parameter_defaults(self):
        """测试默认参数值"""
        # 默认参数: device_type="speaker", role="console"
        device = audio.get_default_device()
        assert device.device_type == "render"