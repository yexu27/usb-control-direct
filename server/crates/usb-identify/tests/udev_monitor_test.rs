use usb_identify::udev_monitor::should_forward_usb_event;

#[test]
fn forwards_usb_interface_add_and_remove_only() {
    assert!(should_forward_usb_event("add", "usb_interface"));
    assert!(should_forward_usb_event("remove", "usb_interface"));

    assert!(!should_forward_usb_event("add", "usb_device"));
    assert!(!should_forward_usb_event("remove", "usb_device"));
    assert!(!should_forward_usb_event("bind", "usb_interface"));
    assert!(!should_forward_usb_event("unbind", "usb_interface"));
    assert!(!should_forward_usb_event("change", "usb_interface"));
    assert!(!should_forward_usb_event("remove", ""));
    assert!(!should_forward_usb_event("add", ""));
}
