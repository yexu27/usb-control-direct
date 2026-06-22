//! HID report 描述符与结构体。
//!
//! 定义标准 boot keyboard（8字节）和 boot mouse（4字节）的 HID report 格式，
//! 以及 Linux evdev keycode 到 HID usage code 的映射表。

/// 键盘 HID report 长度（字节）。
pub const KEYBOARD_REPORT_LEN: usize = 8;

/// 鼠标 HID report 长度（字节）。
pub const MOUSE_REPORT_LEN: usize = 4;

/// 标准 boot keyboard HID report 描述符（50 字节）。
pub const KEYBOARD_REPORT_DESC: &[u8] = &[
    0x05, 0x01, // Usage Page (Generic Desktop)
    0x09, 0x06, // Usage (Keyboard)
    0xA1, 0x01, // Collection (Application)
    0x05, 0x07, //   Usage Page (Key Codes)
    0x19, 0xE0, //   Usage Minimum (224)
    0x29, 0xE7, //   Usage Maximum (231)
    0x15, 0x00, //   Logical Minimum (0)
    0x25, 0x01, //   Logical Maximum (1)
    0x75, 0x01, //   Report Size (1)
    0x95, 0x08, //   Report Count (8)
    0x81, 0x02, //   Input (Data, Variable, Absolute) — modifier 位掩码
    0x95, 0x01, //   Report Count (1)
    0x75, 0x08, //   Report Size (8)
    0x81, 0x01, //   Input (Constant) — reserved byte
    0x95, 0x06, //   Report Count (6)
    0x75, 0x08, //   Report Size (8)
    0x15, 0x00, //   Logical Minimum (0)
    0x25, 0x65, //   Logical Maximum (101)
    0x05, 0x07, //   Usage Page (Key Codes)
    0x19, 0x00, //   Usage Minimum (0)
    0x29, 0x65, //   Usage Maximum (101)
    0x81, 0x00, //   Input (Data, Array) — key array[6]
    0xC0,       // End Collection
];

/// 标准 boot mouse HID report 描述符（39 字节）。
pub const MOUSE_REPORT_DESC: &[u8] = &[
    0x05, 0x01, // Usage Page (Generic Desktop)
    0x09, 0x02, // Usage (Mouse)
    0xA1, 0x01, // Collection (Application)
    0x09, 0x01, //   Usage (Pointer)
    0xA1, 0x00, //   Collection (Physical)
    0x05, 0x09, //     Usage Page (Button)
    0x19, 0x01, //     Usage Minimum (1)
    0x29, 0x03, //     Usage Maximum (3)
    0x15, 0x00, //     Logical Minimum (0)
    0x25, 0x01, //     Logical Maximum (1)
    0x95, 0x03, //     Report Count (3)
    0x75, 0x01, //     Report Size (1)
    0x81, 0x02, //     Input (Data, Variable, Absolute) — button bits
    0x95, 0x01, //     Report Count (1)
    0x75, 0x05, //     Report Size (5)
    0x81, 0x01, //     Input (Constant) — padding
    0x05, 0x01, //     Usage Page (Generic Desktop)
    0x09, 0x30, //     Usage (X)
    0x09, 0x31, //     Usage (Y)
    0x09, 0x38, //     Usage (Wheel)
    0x15, 0x81, //     Logical Minimum (-127)
    0x25, 0x7F, //     Logical Maximum (127)
    0x75, 0x08, //     Report Size (8)
    0x95, 0x03, //     Report Count (3)
    0x81, 0x06, //     Input (Data, Variable, Relative) — dx, dy, wheel
    0xC0,       //   End Collection
    0xC0,       // End Collection
];

/// 键盘 HID report（8 字节）。
///
/// 字节布局：
/// - byte 0: modifier 位掩码
/// - byte 1: reserved（恒为 0）
/// - bytes 2-7: 当前按下的键的 HID usage code（最多 6 键，未使用的位置填 0）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyboardReport {
    pub modifier: u8,
    pub keys: [u8; 6],
}

impl KeyboardReport {
    pub fn empty() -> Self {
        Self {
            modifier: 0,
            keys: [0; 6],
        }
    }

    pub fn to_bytes(&self) -> [u8; KEYBOARD_REPORT_LEN] {
        [
            self.modifier,
            0, // reserved
            self.keys[0],
            self.keys[1],
            self.keys[2],
            self.keys[3],
            self.keys[4],
            self.keys[5],
        ]
    }
}

/// 鼠标 HID report（4 字节）。
///
/// 字节布局：
/// - byte 0: button 位掩码（bit0=左, bit1=右, bit2=中）
/// - byte 1: dx（有符号相对位移）
/// - byte 2: dy（有符号相对位移）
/// - byte 3: wheel（有符号滚轮增量）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseReport {
    pub buttons: u8,
    pub dx: i8,
    pub dy: i8,
    pub wheel: i8,
}

impl MouseReport {
    pub fn empty() -> Self {
        Self {
            buttons: 0,
            dx: 0,
            dy: 0,
            wheel: 0,
        }
    }

    pub fn to_bytes(self) -> [u8; MOUSE_REPORT_LEN] {
        [self.buttons, self.dx as u8, self.dy as u8, self.wheel as u8]
    }
}

/// 将 Linux evdev keycode 映射为 (modifier_bit, hid_usage) 元组。
///
/// 修饰键返回 (modifier_bit, 0)，普通键返回 (0, hid_usage)。
/// 不支持的键返回 None。
pub fn keycode_to_hid(key: evdev::Key) -> Option<(u8, u8)> {
    use evdev::Key;
    match key {
        // 修饰键: modifier_bit 为 modifiers 字节中的位标志
        Key::KEY_LEFTCTRL => Some((0x01, 0)),
        Key::KEY_LEFTSHIFT => Some((0x02, 0)),
        Key::KEY_LEFTALT => Some((0x04, 0)),
        Key::KEY_LEFTMETA => Some((0x08, 0)),
        Key::KEY_RIGHTCTRL => Some((0x10, 0)),
        Key::KEY_RIGHTSHIFT => Some((0x20, 0)),
        Key::KEY_RIGHTALT => Some((0x40, 0)),
        Key::KEY_RIGHTMETA => Some((0x80, 0)),
        // 字母键 A-Z → HID usage 0x04-0x1D
        Key::KEY_A => Some((0, 0x04)),
        Key::KEY_B => Some((0, 0x05)),
        Key::KEY_C => Some((0, 0x06)),
        Key::KEY_D => Some((0, 0x07)),
        Key::KEY_E => Some((0, 0x08)),
        Key::KEY_F => Some((0, 0x09)),
        Key::KEY_G => Some((0, 0x0A)),
        Key::KEY_H => Some((0, 0x0B)),
        Key::KEY_I => Some((0, 0x0C)),
        Key::KEY_J => Some((0, 0x0D)),
        Key::KEY_K => Some((0, 0x0E)),
        Key::KEY_L => Some((0, 0x0F)),
        Key::KEY_M => Some((0, 0x10)),
        Key::KEY_N => Some((0, 0x11)),
        Key::KEY_O => Some((0, 0x12)),
        Key::KEY_P => Some((0, 0x13)),
        Key::KEY_Q => Some((0, 0x14)),
        Key::KEY_R => Some((0, 0x15)),
        Key::KEY_S => Some((0, 0x16)),
        Key::KEY_T => Some((0, 0x17)),
        Key::KEY_U => Some((0, 0x18)),
        Key::KEY_V => Some((0, 0x19)),
        Key::KEY_W => Some((0, 0x1A)),
        Key::KEY_X => Some((0, 0x1B)),
        Key::KEY_Y => Some((0, 0x1C)),
        Key::KEY_Z => Some((0, 0x1D)),
        // 数字键 1-0 → HID usage 0x1E-0x27
        Key::KEY_1 => Some((0, 0x1E)),
        Key::KEY_2 => Some((0, 0x1F)),
        Key::KEY_3 => Some((0, 0x20)),
        Key::KEY_4 => Some((0, 0x21)),
        Key::KEY_5 => Some((0, 0x22)),
        Key::KEY_6 => Some((0, 0x23)),
        Key::KEY_7 => Some((0, 0x24)),
        Key::KEY_8 => Some((0, 0x25)),
        Key::KEY_9 => Some((0, 0x26)),
        Key::KEY_0 => Some((0, 0x27)),
        // 功能键
        Key::KEY_ENTER => Some((0, 0x28)),
        Key::KEY_ESC => Some((0, 0x29)),
        Key::KEY_BACKSPACE => Some((0, 0x2A)),
        Key::KEY_TAB => Some((0, 0x2B)),
        Key::KEY_SPACE => Some((0, 0x2C)),
        Key::KEY_MINUS => Some((0, 0x2D)),
        Key::KEY_EQUAL => Some((0, 0x2E)),
        Key::KEY_LEFTBRACE => Some((0, 0x2F)),
        Key::KEY_RIGHTBRACE => Some((0, 0x30)),
        Key::KEY_BACKSLASH => Some((0, 0x31)),
        Key::KEY_SEMICOLON => Some((0, 0x33)),
        Key::KEY_APOSTROPHE => Some((0, 0x34)),
        Key::KEY_GRAVE => Some((0, 0x35)),
        Key::KEY_COMMA => Some((0, 0x36)),
        Key::KEY_DOT => Some((0, 0x37)),
        Key::KEY_SLASH => Some((0, 0x38)),
        Key::KEY_CAPSLOCK => Some((0, 0x39)),
        // F1-F12
        Key::KEY_F1 => Some((0, 0x3A)),
        Key::KEY_F2 => Some((0, 0x3B)),
        Key::KEY_F3 => Some((0, 0x3C)),
        Key::KEY_F4 => Some((0, 0x3D)),
        Key::KEY_F5 => Some((0, 0x3E)),
        Key::KEY_F6 => Some((0, 0x3F)),
        Key::KEY_F7 => Some((0, 0x40)),
        Key::KEY_F8 => Some((0, 0x41)),
        Key::KEY_F9 => Some((0, 0x42)),
        Key::KEY_F10 => Some((0, 0x43)),
        Key::KEY_F11 => Some((0, 0x44)),
        Key::KEY_F12 => Some((0, 0x45)),
        // 其他常用键
        Key::KEY_DELETE => Some((0, 0x4C)),
        Key::KEY_INSERT => Some((0, 0x49)),
        Key::KEY_HOME => Some((0, 0x4A)),
        Key::KEY_END => Some((0, 0x4D)),
        Key::KEY_PAGEUP => Some((0, 0x4B)),
        Key::KEY_PAGEDOWN => Some((0, 0x4E)),
        Key::KEY_UP => Some((0, 0x52)),
        Key::KEY_DOWN => Some((0, 0x51)),
        Key::KEY_LEFT => Some((0, 0x50)),
        Key::KEY_RIGHT => Some((0, 0x4F)),
        // 不支持的键（如多媒体键、电源键等）
        _ => None,
    }
}

/// 将 i32 值 clamp 到 i8 范围。
pub fn clamp_i8(value: i32) -> i8 {
    value.clamp(-128, 127) as i8
}
