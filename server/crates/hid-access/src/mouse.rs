//! S02 鼠标准入状态机。
//!
//! 实现架构 06 §3 定义的鼠标准入流程：
//! - S01 检测到鼠标设备后，S02 直接尝试映射（无需挑战码）。
//! - 映射成功进入 MouseMapped，映射失败进入 MouseRejected。
//! - 任意状态下拔出设备均进入 MouseRemoved。

use common::types::MouseState;
use tracing::{debug, info, warn};

use crate::error::HidAccessError;

/// 鼠标准入状态机。
///
/// 封装鼠标设备从检测到映射（或拒绝/移除）的完整状态迁移逻辑。
/// 鼠标不需要挑战码验证，检测到设备后直接尝试映射。
pub struct MouseAdmission {
    /// 当前状态。
    state: MouseState,
}

/// 鼠标状态机输入事件。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MouseEvent {
    /// 鼠标映射成功。
    MapSuccess,
    /// 鼠标映射失败。
    MapFailed,
    /// 鼠标物理拔出。
    Unplug,
}

/// 鼠标状态机迁移结果。
///
/// 鼠标状态机每次有效迁移均产生状态变化，无 Unchanged 情况。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MouseTransitionResult {
    /// 状态已迁移，携带新状态。
    Transitioned(MouseState),
}

impl MouseAdmission {
    /// 创建新的鼠标准入实例，初始状态为 MouseDetected。
    pub fn new() -> Self {
        Self {
            state: MouseState::MouseDetected,
        }
    }

    /// 返回当前状态。
    pub fn state(&self) -> MouseState {
        self.state
    }

    /// 驱动状态机处理一个鼠标事件。
    ///
    /// 参数:
    /// - `event`: 鼠标事件，由 S01 设备管理层传入。
    ///
    /// 返回:
    /// - `Ok(Transitioned(new_state))`: 状态迁移成功。
    /// - `Err(InvalidTransition)`: 当前状态不允许该事件。
    pub fn transition(
        &mut self,
        event: MouseEvent,
    ) -> Result<MouseTransitionResult, HidAccessError> {
        debug!("鼠标状态机处理事件");
        // 任意状态下拔出设备，直接迁移到 MouseRemoved。
        if event == MouseEvent::Unplug {
            info!(state = ?self.state, "鼠标已拔出，迁移到 MouseRemoved");
            self.state = MouseState::MouseRemoved;
            return Ok(MouseTransitionResult::Transitioned(
                MouseState::MouseRemoved,
            ));
        }

        match self.state {
            MouseState::MouseDetected => self.handle_detected(event),
            MouseState::MouseMapped
            | MouseState::MouseRejected
            | MouseState::MouseRemoved => {
                // 终态不接受除 Unplug 外的任何事件（Unplug 已在上方处理）。
                Err(HidAccessError::InvalidTransition {
                    from: format!("{:?}", self.state),
                    event: format!("{:?}", event),
                })
            }
        }
    }

    /// 处理 MouseDetected 状态下的事件。
    fn handle_detected(
        &mut self,
        event: MouseEvent,
    ) -> Result<MouseTransitionResult, HidAccessError> {
        let old_state = self.state;
        match event {
            MouseEvent::MapSuccess => {
                self.state = MouseState::MouseMapped;
                debug!(from = ?old_state, to = ?self.state, "鼠标状态转换");
                info!("鼠标映射成功，迁移到 MouseMapped");
                Ok(MouseTransitionResult::Transitioned(MouseState::MouseMapped))
            }
            MouseEvent::MapFailed => {
                self.state = MouseState::MouseRejected;
                debug!(from = ?old_state, to = ?self.state, "鼠标状态转换");
                warn!("鼠标映射失败，迁移到 MouseRejected");
                Ok(MouseTransitionResult::Transitioned(
                    MouseState::MouseRejected,
                ))
            }
            // Unplug 已在 transition() 入口处理，此处不会到达。
            MouseEvent::Unplug => unreachable!("Unplug 已在入口处理"),
        }
    }
}

impl Default for MouseAdmission {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 初始状态为_mouse_detected() {
        let ma = MouseAdmission::new();
        assert_eq!(ma.state(), MouseState::MouseDetected);
    }

    #[test]
    fn 映射成功后进入_mouse_mapped() {
        let mut ma = MouseAdmission::new();
        let result = ma.transition(MouseEvent::MapSuccess).unwrap();
        assert_eq!(
            result,
            MouseTransitionResult::Transitioned(MouseState::MouseMapped)
        );
        assert_eq!(ma.state(), MouseState::MouseMapped);
    }

    #[test]
    fn 映射失败后进入_mouse_rejected() {
        let mut ma = MouseAdmission::new();
        let result = ma.transition(MouseEvent::MapFailed).unwrap();
        assert_eq!(
            result,
            MouseTransitionResult::Transitioned(MouseState::MouseRejected)
        );
        assert_eq!(ma.state(), MouseState::MouseRejected);
    }

    #[test]
    fn 任意状态下拔出进入_mouse_removed() {
        // MouseDetected 状态拔出
        let mut ma = MouseAdmission::new();
        let result = ma.transition(MouseEvent::Unplug).unwrap();
        assert_eq!(
            result,
            MouseTransitionResult::Transitioned(MouseState::MouseRemoved)
        );
        assert_eq!(ma.state(), MouseState::MouseRemoved);

        // MouseMapped 状态拔出
        let mut ma = MouseAdmission::new();
        ma.transition(MouseEvent::MapSuccess).unwrap();
        let result = ma.transition(MouseEvent::Unplug).unwrap();
        assert_eq!(
            result,
            MouseTransitionResult::Transitioned(MouseState::MouseRemoved)
        );

        // MouseRejected 状态拔出
        let mut ma = MouseAdmission::new();
        ma.transition(MouseEvent::MapFailed).unwrap();
        let result = ma.transition(MouseEvent::Unplug).unwrap();
        assert_eq!(
            result,
            MouseTransitionResult::Transitioned(MouseState::MouseRemoved)
        );
    }

    #[test]
    fn 终态_mouse_mapped_不接受普通事件() {
        let mut ma = MouseAdmission::new();
        ma.transition(MouseEvent::MapSuccess).unwrap();
        assert_eq!(ma.state(), MouseState::MouseMapped);

        let result = ma.transition(MouseEvent::MapSuccess);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HidAccessError::InvalidTransition { .. }
        ));
    }

    #[test]
    fn 终态_mouse_rejected_不接受普通事件() {
        let mut ma = MouseAdmission::new();
        ma.transition(MouseEvent::MapFailed).unwrap();
        assert_eq!(ma.state(), MouseState::MouseRejected);

        let result = ma.transition(MouseEvent::MapFailed);
        assert!(result.is_err());
    }

    #[test]
    fn 终态_mouse_removed_不接受普通事件() {
        let mut ma = MouseAdmission::new();
        ma.transition(MouseEvent::Unplug).unwrap();
        assert_eq!(ma.state(), MouseState::MouseRemoved);

        let result = ma.transition(MouseEvent::MapSuccess);
        assert!(result.is_err());
    }
}
