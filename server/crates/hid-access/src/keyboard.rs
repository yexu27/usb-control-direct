//! S02 键盘准入状态机。
//!
//! 实现架构 06 §2 定义的键盘准入流程：
//! - S01 检测到键盘设备后，S02 尝试 HID 接管（grab）。
//! - 接管成功后进入等待状态，要求用户按序输入 "1234" 挑战码。
//! - 修饰键（Ctrl/Alt/Shift/Fn 等）不干扰序列匹配，也不重置缓冲区。
//! - 错误按键清空缓冲区，继续等待，不超时、不自动拒绝。
//! - 接管失败直接进入 KbRejected，不进入挑战流程。
//! - 任意状态下拔出设备均进入 KbRemoved。

use common::types::KeyboardState;
use tracing::{debug, info, warn};

use crate::error::HidAccessError;

/// 键盘挑战码：用户必须按序输入 "1234"。
const CHALLENGE_CODE: &[u8] = b"1234";

/// 键盘准入状态机。
///
/// 封装键盘设备从检测到映射（或拒绝/移除）的完整状态迁移逻辑。
/// 调用方负责将 HID 事件转换为 `KeyboardEvent` 后驱动本状态机。
pub struct KeyboardChallenge {
    /// 当前状态。
    state: KeyboardState,
    /// 挑战码输入缓冲区，记录已正确输入的前缀长度。
    input_buffer: Vec<u8>,
}

/// 键盘状态机输入事件。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyboardEvent {
    /// HID 接管成功，可以开始接收按键。
    GrabSuccess,
    /// HID 接管失败，键盘将被拒绝。
    GrabFailed,
    /// 普通按键（非修饰键），携带 HID 用途码字节。
    KeyPress(u8),
    /// 修饰键按下（Ctrl/Alt/Shift/Fn 等），不影响挑战序列。
    ModifierKey,
    /// 键盘物理拔出。
    Unplug,
}

/// 键盘状态机迁移结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyboardTransitionResult {
    /// 状态已迁移，携带新状态。
    Transitioned(KeyboardState),
    /// 状态未变化（例如挑战码输入中途或错误按键）。
    Unchanged,
}

impl KeyboardChallenge {
    /// 创建新的键盘挑战实例，初始状态为 KbDetected。
    pub fn new() -> Self {
        Self {
            state: KeyboardState::KbDetected,
            input_buffer: Vec::new(),
        }
    }

    /// 返回当前状态。
    pub fn state(&self) -> KeyboardState {
        self.state
    }

    /// 驱动状态机处理一个键盘事件。
    ///
    /// 参数:
    /// - `event`: 键盘事件，由 S01 HID 事件流转换而来。
    ///
    /// 返回:
    /// - `Ok(Transitioned(new_state))`: 状态迁移成功。
    /// - `Ok(Unchanged)`: 状态未变化（挑战码输入中途或修饰键）。
    /// - `Err(InvalidTransition)`: 当前状态不允许该事件。
    pub fn transition(
        &mut self,
        event: KeyboardEvent,
    ) -> Result<KeyboardTransitionResult, HidAccessError> {
        debug!(state = ?self.state, event = ?event, "键盘状态机处理事件");

        // 任意状态下拔出设备，直接迁移到 KbRemoved。
        if event == KeyboardEvent::Unplug {
            info!(state = ?self.state, "键盘已拔出，迁移到 KbRemoved");
            self.input_buffer.clear();
            self.state = KeyboardState::KbRemoved;
            return Ok(KeyboardTransitionResult::Transitioned(
                KeyboardState::KbRemoved,
            ));
        }

        match self.state {
            KeyboardState::KbDetected => self.handle_detected(event),
            KeyboardState::KbWaiting => self.handle_waiting(event),
            KeyboardState::KbMapped
            | KeyboardState::KbRejected
            | KeyboardState::KbRemoved => {
                // 终态不接受除 Unplug 外的任何事件（Unplug 已在上方处理）。
                Err(HidAccessError::InvalidTransition {
                    from: format!("{:?}", self.state),
                    event: format!("{:?}", event),
                })
            }
        }
    }

    /// 处理 KbDetected 状态下的事件。
    fn handle_detected(
        &mut self,
        event: KeyboardEvent,
    ) -> Result<KeyboardTransitionResult, HidAccessError> {
        match event {
            KeyboardEvent::GrabSuccess => {
                info!("HID 接管成功，进入挑战等待状态 KbWaiting");
                self.input_buffer.clear();
                self.state = KeyboardState::KbWaiting;
                Ok(KeyboardTransitionResult::Transitioned(
                    KeyboardState::KbWaiting,
                ))
            }
            KeyboardEvent::GrabFailed => {
                warn!("HID 接管失败，键盘被拒绝，迁移到 KbRejected");
                self.state = KeyboardState::KbRejected;
                Ok(KeyboardTransitionResult::Transitioned(
                    KeyboardState::KbRejected,
                ))
            }
            other => Err(HidAccessError::InvalidTransition {
                from: format!("{:?}", self.state),
                event: format!("{:?}", other),
            }),
        }
    }

    /// 处理 KbWaiting 状态下的事件。
    fn handle_waiting(
        &mut self,
        event: KeyboardEvent,
    ) -> Result<KeyboardTransitionResult, HidAccessError> {
        match event {
            KeyboardEvent::ModifierKey => {
                // 修饰键不影响挑战序列，直接忽略。
                debug!("忽略修饰键，挑战序列不受影响");
                Ok(KeyboardTransitionResult::Unchanged)
            }
            KeyboardEvent::KeyPress(key) => self.handle_key_press(key),
            other => Err(HidAccessError::InvalidTransition {
                from: format!("{:?}", self.state),
                event: format!("{:?}", other),
            }),
        }
    }

    /// 处理挑战序列中的一次按键。
    ///
    /// - 按键与挑战码当前期望位置匹配：追加到缓冲区。
    ///   - 若序列完整（4位全部匹配）：迁移到 KbMapped。
    ///   - 否则：继续等待，返回 Unchanged。
    /// - 按键不匹配：清空缓冲区，返回 Unchanged（继续等待）。
    fn handle_key_press(
        &mut self,
        key: u8,
    ) -> Result<KeyboardTransitionResult, HidAccessError> {
        let expected_pos = self.input_buffer.len();

        // 防御：缓冲区长度不应超过挑战码长度（正常流程下不会发生）。
        if expected_pos >= CHALLENGE_CODE.len() {
            warn!("挑战缓冲区异常溢出，清空并重置");
            self.input_buffer.clear();
            return Ok(KeyboardTransitionResult::Unchanged);
        }

        let expected_key = CHALLENGE_CODE[expected_pos];

        if key == expected_key {
            self.input_buffer.push(key);
            debug!(
                progress = self.input_buffer.len(),
                total = CHALLENGE_CODE.len(),
                "挑战码输入正确"
            );

            if self.input_buffer.len() == CHALLENGE_CODE.len() {
                info!("挑战码验证通过，键盘映射成功，迁移到 KbMapped");
                self.input_buffer.clear();
                self.state = KeyboardState::KbMapped;
                return Ok(KeyboardTransitionResult::Transitioned(
                    KeyboardState::KbMapped,
                ));
            }

            // 序列未完成，继续等待。
            Ok(KeyboardTransitionResult::Unchanged)
        } else {
            debug!(
                key,
                expected = expected_key,
                "挑战码输入错误，清空缓冲区重新等待"
            );
            self.input_buffer.clear();
            Ok(KeyboardTransitionResult::Unchanged)
        }
    }
}

impl Default for KeyboardChallenge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 辅助函数：模拟输入一组按键序列。
    fn press_sequence(
        challenge: &mut KeyboardChallenge,
        keys: &[u8],
    ) -> Result<KeyboardTransitionResult, HidAccessError> {
        let mut last = Ok(KeyboardTransitionResult::Unchanged);
        for &key in keys {
            last = challenge.transition(KeyboardEvent::KeyPress(key));
            if last.is_err() {
                return last;
            }
        }
        last
    }

    #[test]
    fn 初始状态为_kb_detected() {
        let ch = KeyboardChallenge::new();
        assert_eq!(ch.state(), KeyboardState::KbDetected);
    }

    #[test]
    fn 接管成功后进入_kb_waiting() {
        let mut ch = KeyboardChallenge::new();
        let result = ch.transition(KeyboardEvent::GrabSuccess).unwrap();
        assert_eq!(
            result,
            KeyboardTransitionResult::Transitioned(KeyboardState::KbWaiting)
        );
        assert_eq!(ch.state(), KeyboardState::KbWaiting);
    }

    #[test]
    fn 接管失败后进入_kb_rejected() {
        let mut ch = KeyboardChallenge::new();
        let result = ch.transition(KeyboardEvent::GrabFailed).unwrap();
        assert_eq!(
            result,
            KeyboardTransitionResult::Transitioned(KeyboardState::KbRejected)
        );
        assert_eq!(ch.state(), KeyboardState::KbRejected);
    }

    #[test]
    fn 正确输入1234后进入_kb_mapped() {
        let mut ch = KeyboardChallenge::new();
        ch.transition(KeyboardEvent::GrabSuccess).unwrap();

        let result = press_sequence(&mut ch, b"1234").unwrap();
        assert_eq!(
            result,
            KeyboardTransitionResult::Transitioned(KeyboardState::KbMapped)
        );
        assert_eq!(ch.state(), KeyboardState::KbMapped);
    }

    #[test]
    fn 错误按键清空缓冲区后仍可继续输入() {
        let mut ch = KeyboardChallenge::new();
        ch.transition(KeyboardEvent::GrabSuccess).unwrap();

        // 先输入部分正确序列
        press_sequence(&mut ch, b"12").unwrap();
        // 输入错误按键
        let result = ch.transition(KeyboardEvent::KeyPress(b'9')).unwrap();
        assert_eq!(result, KeyboardTransitionResult::Unchanged);
        assert_eq!(ch.state(), KeyboardState::KbWaiting);

        // 重新输入完整正确序列应能成功
        let result = press_sequence(&mut ch, b"1234").unwrap();
        assert_eq!(
            result,
            KeyboardTransitionResult::Transitioned(KeyboardState::KbMapped)
        );
    }

    #[test]
    fn 修饰键不影响挑战序列() {
        let mut ch = KeyboardChallenge::new();
        ch.transition(KeyboardEvent::GrabSuccess).unwrap();

        // 输入部分序列后夹杂修饰键
        ch.transition(KeyboardEvent::KeyPress(b'1')).unwrap();
        let modifier_result = ch.transition(KeyboardEvent::ModifierKey).unwrap();
        assert_eq!(modifier_result, KeyboardTransitionResult::Unchanged);

        // 修饰键后继续完成序列
        let result = press_sequence(&mut ch, b"234").unwrap();
        assert_eq!(
            result,
            KeyboardTransitionResult::Transitioned(KeyboardState::KbMapped)
        );
    }

    #[test]
    fn 任意状态下拔出设备进入_kb_removed() {
        // KbDetected 状态拔出
        let mut ch = KeyboardChallenge::new();
        let result = ch.transition(KeyboardEvent::Unplug).unwrap();
        assert_eq!(
            result,
            KeyboardTransitionResult::Transitioned(KeyboardState::KbRemoved)
        );

        // KbWaiting 状态拔出
        let mut ch = KeyboardChallenge::new();
        ch.transition(KeyboardEvent::GrabSuccess).unwrap();
        let result = ch.transition(KeyboardEvent::Unplug).unwrap();
        assert_eq!(
            result,
            KeyboardTransitionResult::Transitioned(KeyboardState::KbRemoved)
        );

        // KbRejected 状态拔出
        let mut ch = KeyboardChallenge::new();
        ch.transition(KeyboardEvent::GrabFailed).unwrap();
        let result = ch.transition(KeyboardEvent::Unplug).unwrap();
        assert_eq!(
            result,
            KeyboardTransitionResult::Transitioned(KeyboardState::KbRemoved)
        );
    }

    #[test]
    fn kb_detected状态不接受按键事件() {
        let mut ch = KeyboardChallenge::new();
        let result = ch.transition(KeyboardEvent::KeyPress(b'1'));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, HidAccessError::InvalidTransition { .. }));
    }

    #[test]
    fn 终态kb_mapped不接受普通事件() {
        let mut ch = KeyboardChallenge::new();
        ch.transition(KeyboardEvent::GrabSuccess).unwrap();
        press_sequence(&mut ch, b"1234").unwrap();
        assert_eq!(ch.state(), KeyboardState::KbMapped);

        let result = ch.transition(KeyboardEvent::GrabSuccess);
        assert!(result.is_err());
    }

    #[test]
    fn 终态kb_rejected不接受普通事件() {
        let mut ch = KeyboardChallenge::new();
        ch.transition(KeyboardEvent::GrabFailed).unwrap();
        assert_eq!(ch.state(), KeyboardState::KbRejected);

        let result = ch.transition(KeyboardEvent::KeyPress(b'1'));
        assert!(result.is_err());
    }

    #[test]
    fn 逐步输入123验证中间状态不变() {
        let mut ch = KeyboardChallenge::new();
        ch.transition(KeyboardEvent::GrabSuccess).unwrap();

        for key in b"123" {
            let result = ch.transition(KeyboardEvent::KeyPress(*key)).unwrap();
            assert_eq!(result, KeyboardTransitionResult::Unchanged);
            assert_eq!(ch.state(), KeyboardState::KbWaiting);
        }
    }
}
