//! U 盘状态机。
//!
//! 严格按架构 06 §1 定义的状态转换表实现。
//! 所有状态转换集中在 transition 函数中，其他模块只能投递事件。

use common::types::UsbDeviceState;
use tracing::{info, warn};

use crate::error::UsbIdentifyError;

/// U 盘状态转换事件。
#[derive(Debug, Clone)]
pub enum UsbDeviceEvent {
    /// USB 插入检测。
    Inserted,
    /// 设备分类完成：存储设备。
    ClassifiedAsStorage,
    /// 设备分类完成：非存储/未知/伪装。
    ClassifiedAsRejected { reason: String },
    /// 白名单查询：不在白名单。
    WhitelistDenied,
    /// 白名单查询：在白名单。
    WhitelistApproved,
    /// mount 失败。
    MountFailed { reason: String },
    /// 扫描完成：未发现病毒。
    ScanClean,
    /// 扫描完成：发现病毒。
    ScanInfected { infected_files: Vec<String> },
    /// 扫描失败。
    ScanFailed { reason: String },
    /// 映射成功。
    MapSuccess,
    /// 映射失败。
    MapFailed { reason: String },
    /// USB 拔出。
    Unplug,
}

/// U 盘状态转换结果。
#[derive(Debug, Clone, PartialEq)]
pub struct TransitionResult {
    pub new_state: UsbDeviceState,
}

/// U 盘状态机。
pub struct UsbStateMachine {
    state: UsbDeviceState,
}

impl UsbStateMachine {
    /// 创建状态机，初始状态为 Detected。
    pub fn new() -> Self {
        UsbStateMachine {
            state: UsbDeviceState::Detected,
        }
    }

    /// 当前状态。
    pub fn state(&self) -> UsbDeviceState {
        self.state
    }

    /// 集中式状态转换函数。
    ///
    /// 严格按架构 06 §1 定义的状态转换表执行。
    /// 禁止跳过状态、禁止虚构架构未定义的状态名。
    pub fn transition(
        &mut self,
        event: UsbDeviceEvent,
    ) -> Result<TransitionResult, UsbIdentifyError> {
        match (&self.state, &event) {
            // DETECTED + Inserted → CLASSIFYING
            (UsbDeviceState::Detected, UsbDeviceEvent::Inserted) => {
                self.state = UsbDeviceState::Classifying;
                info!("U 盘进入分类阶段");
                Ok(TransitionResult {
                    new_state: UsbDeviceState::Classifying,
                })
            }

            // CLASSIFYING + ClassifiedAsRejected → REJECTED
            (UsbDeviceState::Classifying, UsbDeviceEvent::ClassifiedAsRejected { reason }) => {
                self.state = UsbDeviceState::Rejected;
                warn!(reason = %reason, "U 盘分类拒绝");
                Ok(TransitionResult {
                    new_state: UsbDeviceState::Rejected,
                })
            }

            // CLASSIFYING + WhitelistDenied → BLOCKED
            (UsbDeviceState::Classifying, UsbDeviceEvent::WhitelistDenied) => {
                self.state = UsbDeviceState::Blocked;
                info!("U 盘不在白名单，已阻止");
                Ok(TransitionResult {
                    new_state: UsbDeviceState::Blocked,
                })
            }

            // CLASSIFYING + WhitelistApproved → SCANNING (经过 mount)
            (UsbDeviceState::Classifying, UsbDeviceEvent::WhitelistApproved) => {
                self.state = UsbDeviceState::Scanning;
                info!("U 盘在白名单，mount 成功，进入扫描阶段");
                Ok(TransitionResult {
                    new_state: UsbDeviceState::Scanning,
                })
            }

            // CLASSIFYING + MountFailed → SCAN_FAILED
            (UsbDeviceState::Classifying, UsbDeviceEvent::MountFailed { reason }) => {
                self.state = UsbDeviceState::ScanFailed;
                warn!(reason = %reason, "U 盘挂载失败");
                Ok(TransitionResult {
                    new_state: UsbDeviceState::ScanFailed,
                })
            }

            // SCANNING + ScanClean → CLEAN
            (UsbDeviceState::Scanning, UsbDeviceEvent::ScanClean) => {
                self.state = UsbDeviceState::Clean;
                info!("U 盘扫描完成，未发现病毒");
                Ok(TransitionResult {
                    new_state: UsbDeviceState::Clean,
                })
            }

            // SCANNING + ScanInfected → INFECTED
            (UsbDeviceState::Scanning, UsbDeviceEvent::ScanInfected { .. }) => {
                self.state = UsbDeviceState::Infected;
                warn!("U 盘扫描完成，发现病毒");
                Ok(TransitionResult {
                    new_state: UsbDeviceState::Infected,
                })
            }

            // SCANNING + ScanFailed → SCAN_FAILED
            (UsbDeviceState::Scanning, UsbDeviceEvent::ScanFailed { reason }) => {
                self.state = UsbDeviceState::ScanFailed;
                warn!(reason = %reason, "U 盘扫描失败");
                Ok(TransitionResult {
                    new_state: UsbDeviceState::ScanFailed,
                })
            }

            // CLEAN + MapSuccess → MAPPED
            (UsbDeviceState::Clean, UsbDeviceEvent::MapSuccess) => {
                self.state = UsbDeviceState::Mapped;
                info!("U 盘映射成功（无病毒）");
                Ok(TransitionResult {
                    new_state: UsbDeviceState::Mapped,
                })
            }

            // CLEAN + MapFailed → MAP_FAILED
            (UsbDeviceState::Clean, UsbDeviceEvent::MapFailed { reason }) => {
                self.state = UsbDeviceState::MapFailed;
                warn!(reason = %reason, "U 盘映射失败（无病毒）");
                Ok(TransitionResult {
                    new_state: UsbDeviceState::MapFailed,
                })
            }

            // INFECTED + MapSuccess → MAPPED
            (UsbDeviceState::Infected, UsbDeviceEvent::MapSuccess) => {
                self.state = UsbDeviceState::Mapped;
                info!("U 盘映射成功（含病毒文件）");
                Ok(TransitionResult {
                    new_state: UsbDeviceState::Mapped,
                })
            }

            // INFECTED + MapFailed → MAP_FAILED
            (UsbDeviceState::Infected, UsbDeviceEvent::MapFailed { reason }) => {
                self.state = UsbDeviceState::MapFailed;
                warn!(reason = %reason, "U 盘映射失败（含病毒文件）");
                Ok(TransitionResult {
                    new_state: UsbDeviceState::MapFailed,
                })
            }

            // 任意状态 + Unplug → REMOVED
            (_, UsbDeviceEvent::Unplug) => {
                let from = self.state;
                self.state = UsbDeviceState::Removed;
                info!(from = ?from, "U 盘拔出");
                Ok(TransitionResult {
                    new_state: UsbDeviceState::Removed,
                })
            }

            // 非法转换
            (state, event) => Err(UsbIdentifyError::InvalidTransition {
                from: format!("{:?}", state),
                event: format!("{:?}", event),
            }),
        }
    }
}
