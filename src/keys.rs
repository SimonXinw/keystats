pub const MOUSE_LEFT: u16 = 0x1000;
pub const MOUSE_RIGHT: u16 = 0x1001;
pub const MOUSE_MIDDLE: u16 = 0x1002;
pub const MOUSE_X1: u16 = 0x1003;
pub const MOUSE_X2: u16 = 0x1004;
pub const WHEEL_UP: u16 = 0x1005;
pub const WHEEL_DOWN: u16 = 0x1006;
pub const WHEEL_LEFT: u16 = 0x1007;
pub const WHEEL_RIGHT: u16 = 0x1008;
pub const NUMPAD_ENTER: u16 = 0x110D;

const LLKHF_EXTENDED: u32 = 0x01;

pub fn is_mouse(key: u16) -> bool {
    (MOUSE_LEFT..=WHEEL_RIGHT).contains(&key)
}

pub fn category(key: u16) -> &'static str {
    if is_mouse(key) {
        "鼠标"
    } else {
        "键盘"
    }
}

/// 把低级键盘钩子里的 vk / scan / flags 收成稳定的按键编号。
/// 左右 Shift 靠扫描码，左右 Ctrl / Alt 靠扩展标志。
pub fn normalize_vk(vk: u32, scan: u32, flags: u32) -> u16 {
    match vk {
        0x10 => {
            if scan == 0x36 {
                0xA1
            } else {
                0xA0
            }
        }
        0x11 => {
            if flags & LLKHF_EXTENDED != 0 {
                0xA3
            } else {
                0xA2
            }
        }
        0x12 => {
            if flags & LLKHF_EXTENDED != 0 {
                0xA5
            } else {
                0xA4
            }
        }
        0x0D if flags & LLKHF_EXTENDED != 0 => NUMPAD_ENTER,
        value if value <= u16::MAX as u32 => value as u16,
        _ => 0,
    }
}

pub fn label(key: u16) -> String {
    if let Some(name) = special_name(key) {
        return name.to_string();
    }
    match key {
        0x41..=0x5A => char::from(key as u8).to_string(),
        0x30..=0x39 => char::from(key as u8).to_string(),
        0x70..=0x87 => format!("F{}", key - 0x6F),
        0x60..=0x69 => format!("小键盘 {}", key - 0x60),
        _ => format!("未知键 {key:02X}"),
    }
}

fn special_name(key: u16) -> Option<&'static str> {
    Some(match key {
        MOUSE_LEFT => "鼠标左键",
        MOUSE_RIGHT => "鼠标右键",
        MOUSE_MIDDLE => "鼠标中键",
        MOUSE_X1 => "鼠标侧键1",
        MOUSE_X2 => "鼠标侧键2",
        WHEEL_UP => "滚轮上",
        WHEEL_DOWN => "滚轮下",
        WHEEL_LEFT => "滚轮左",
        WHEEL_RIGHT => "滚轮右",
        NUMPAD_ENTER => "小键盘 Enter",
        0x08 => "Backspace",
        0x09 => "Tab",
        0x0D => "Enter",
        0x13 => "Pause",
        0x14 => "Caps Lock",
        0x1B => "Esc",
        0x20 => "空格",
        0x21 => "Page Up",
        0x22 => "Page Down",
        0x23 => "End",
        0x24 => "Home",
        0x25 => "方向左",
        0x26 => "方向上",
        0x27 => "方向右",
        0x28 => "方向下",
        0x2C => "Print Screen",
        0x2D => "Insert",
        0x2E => "Delete",
        0x5B => "左 Win",
        0x5C => "右 Win",
        0x5D => "菜单",
        0x6A => "小键盘 *",
        0x6B => "小键盘 +",
        0x6D => "小键盘 -",
        0x6E => "小键盘 .",
        0x6F => "小键盘 /",
        0x90 => "Num Lock",
        0x91 => "Scroll Lock",
        0xA0 => "左 Shift",
        0xA1 => "右 Shift",
        0xA2 => "左 Ctrl",
        0xA3 => "右 Ctrl",
        0xA4 => "左 Alt",
        0xA5 => "右 Alt",
        0xBA => ";",
        0xBB => "=",
        0xBC => ",",
        0xBD => "-",
        0xBE => ".",
        0xBF => "/",
        0xC0 => "`",
        0xDB => "[",
        0xDC => "\\",
        0xDD => "]",
        0xDE => "'",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_left_and_right_modifiers() {
        assert_eq!(normalize_vk(0x10, 0x2A, 0), 0xA0);
        assert_eq!(normalize_vk(0x10, 0x36, 0), 0xA1);
        assert_eq!(normalize_vk(0x11, 0, 0), 0xA2);
        assert_eq!(normalize_vk(0x11, 0, LLKHF_EXTENDED), 0xA3);
        assert_eq!(normalize_vk(0x12, 0, LLKHF_EXTENDED), 0xA5);
        assert_eq!(normalize_vk(0x0D, 0x1C, LLKHF_EXTENDED), NUMPAD_ENTER);
        assert_eq!(label(0x51), "Q");
        assert_eq!(label(0x70), "F1");
        assert_eq!(label(0x87), "F24");
        assert_eq!(label(MOUSE_LEFT), "鼠标左键");
        assert_eq!(label(0xA0), "左 Shift");
        assert_eq!(label(0x0D), "Enter");
    }
}
