//! USB 启动装配。
//!
//! 本模块只做 app bootstrap 编排，不承载单个 USB 设备业务流程。

use std::path::PathBuf;

use file_access::gadget::GadgetRuntime;
use file_access::nbd::NbdDeviceManager;
use file_access::raw_mount::RealMountOps;
use file_access::startup_recovery::{
    run_startup_recovery, StartupRecoveryConfig, StartupRecoveryReport,
};
use hid_access::hid_gadget::{discover_hidg_nodes_for_functions, HidFunctionNames, HidgNodes};

use crate::config::AppConfig;

/// USB runtime 启动结果。
pub struct UsbBootstrapResult {
    pub gadget_runtime: GadgetRuntime,
    pub hidg_nodes: HidgNodes,
    pub recovery_report: StartupRecoveryReport,
}

/// 准备 USB gadget、HID 节点和 storage 启动恢复。
pub fn prepare_usb_runtime(
    config: &AppConfig,
    device_description: String,
) -> Result<UsbBootstrapResult, String> {
    let bootstrap_config = file_access::gadget_bootstrap::GadgetBootstrapConfig {
        configfs_root: PathBuf::from("/sys/kernel/config/usb_gadget"),
        udc_root: PathBuf::from("/sys/class/udc"),
        gadget_name: config.gadget.name.clone(),
        config_name: config.gadget.config.clone(),
        udc: config.gadget.udc.clone(),
        keep_adb: config.gadget.keep_adb,
        storage_function: config.gadget.storage.function.clone(),
        storage_lun: config.gadget.storage.lun,
        keyboard_function: config.gadget.keyboard.function.clone(),
        mouse_function: config.gadget.mouse.function.clone(),
        device_description,
    };

    let gadget_runtime = file_access::gadget_bootstrap::GadgetBootstrap::prepare(bootstrap_config)
        .map_err(|e| format!("USB gadget bootstrap 失败: {e}"))?;

    let names = HidFunctionNames {
        keyboard: config.gadget.keyboard.function.clone(),
        mouse: config.gadget.mouse.function.clone(),
    };
    let hidg_nodes = discover_hidg_nodes_for_functions(gadget_runtime.gadget_dir(), &names)
        .map_err(|e| format!("hidg 节点发现失败: {e}"))?;

    let recovery_config = StartupRecoveryConfig::production(gadget_runtime.lun_dir().join("file"));
    let recovery_report = run_startup_recovery(
        &recovery_config,
        &RealMountOps,
        &NbdDeviceManager::default(),
    )
    .map_err(|e| format!("USB storage 启动恢复失败: {e}"))?;

    Ok(UsbBootstrapResult {
        gadget_runtime,
        hidg_nodes,
        recovery_report,
    })
}
