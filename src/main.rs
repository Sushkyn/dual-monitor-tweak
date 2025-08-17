#[link(name = "Xrandr")]
extern "C" {}

use std::os::raw::*;
use std::ptr;
use std::thread::sleep;
use std::time::Duration;

use device_query::{DeviceQuery, DeviceState, Keycode};
use x11::xlib::*;
use x11::xrandr::*;

#[derive(Debug, Clone, Copy)]
struct Monitor {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

fn get_monitors(display: *mut Display, screen: c_int) -> Vec<Monitor> {
    unsafe {
        let root = XRootWindow(display, screen);
        let mut count = 0;
        let monitors = XRRGetMonitors(display, root, 1, &mut count);
        let slice = std::slice::from_raw_parts(monitors, count as usize);
        let result = slice
            .iter()
            .map(|m| Monitor {
                x: m.x,
                y: m.y,
                width: m.width,
                height: m.height,
            })
            .collect();
        XRRFreeMonitors(monitors);
        result
    }
}

fn get_cursor_position(display: *mut Display, root: Window) -> (i32, i32) {
    unsafe {
        let (mut root_x, mut root_y) = (0, 0);
        let (mut win_x, mut win_y) = (0, 0);
        let (mut mask, mut child, mut root_return) = (0, 0, 0);

        XQueryPointer(
            display,
            root,
            &mut root_return,
            &mut child,
            &mut root_x,
            &mut root_y,
            &mut win_x,
            &mut win_y,
            &mut mask,
        );

        (root_x, root_y)
    }
}

fn move_cursor(display: *mut Display, root: Window, x: i32, y: i32) {
    unsafe {
        XWarpPointer(display, 0, root, 0, 0, 0, 0, x, y);
        XFlush(display);
    }
}

fn get_monitor_index(monitors: &[Monitor], x: i32, y: i32) -> usize {
    for (i, m) in monitors.iter().enumerate() {
        if x >= m.x && x < m.x + m.width && y >= m.y && y < m.y + m.height {
            return i;
        }
    }
    0
}

fn get_focused_window(display: *mut Display) -> Option<Window> {
    unsafe {
        let mut focused: Window = 0;
        let mut revert_to: c_int = 0;
        XGetInputFocus(display, &mut focused, &mut revert_to);
        if focused != 0 {
            Some(focused)
        } else {
            None
        }
    }
}

fn move_window_to_monitor(display: *mut Display, window: Window, monitor: Monitor) {
    unsafe {
        let mut attrs: XWindowAttributes = std::mem::zeroed();
        if XGetWindowAttributes(display, window, &mut attrs) == 0 {
            return;
        }

        let width = attrs.width;
        let height = attrs.height;

        // Center window inside target monitor
        let new_x = monitor.x + (monitor.width - width) / 2;
        let new_y = monitor.y + (monitor.height - height) / 2;

        XMoveResizeWindow(display, window, new_x, new_y, width as u32, height as u32);
        XFlush(display);
    }
}

fn main() {
    let device_state = DeviceState::new();
    let display = unsafe { XOpenDisplay(ptr::null()) };
    if display.is_null() {
        eprintln!("Failed to open X display.");
        return;
    }

    let screen = unsafe { XDefaultScreen(display) };
    let root = unsafe { XRootWindow(display, screen) };

    loop {
        let keys = device_state.get_keys();

        // Ctrl+M # cycle through monitors
        if (keys.contains(&Keycode::LControl) || keys.contains(&Keycode::RControl))
            && keys.contains(&Keycode::M)
        {
            let monitors = get_monitors(display, screen);
            let (x, y) = get_cursor_position(display, root);
            let current_index = get_monitor_index(&monitors, x, y);
            let next_index = (current_index + 1) % monitors.len();
            let target = monitors[next_index];

            let target_x = target.x + target.width / 2;
            let target_y = target.y + target.height / 2;

            move_cursor(display, root, target_x, target_y);

            if let Some(window) = get_focused_window(display) {
                move_window_to_monitor(display, window, target);
            }

            while device_state.get_keys().contains(&Keycode::M) {
                sleep(Duration::from_millis(50));
            }
        }

        // Ctrl+1 # Ctrl+9 # send directly to specific monitor
        if keys.contains(&Keycode::LControl) || keys.contains(&Keycode::RControl) {
            let monitors = get_monitors(display, screen);

            let number_keys = [
                Keycode::Key1,
                Keycode::Key2,
                Keycode::Key3,
                Keycode::Key4,
                Keycode::Key5,
                Keycode::Key6,
                Keycode::Key7,
                Keycode::Key8,
                Keycode::Key9,
            ];

            for (i, key) in number_keys.iter().enumerate() {
                if keys.contains(key) && i < monitors.len() {
                    let target = monitors[i];

                    let target_x = target.x + target.width / 2;
                    let target_y = target.y + target.height / 2;

                    move_cursor(display, root, target_x, target_y);

                    if let Some(window) = get_focused_window(display) {
                        move_window_to_monitor(display, window, target);
                    }

                    while device_state.get_keys().contains(key) {
                        sleep(Duration::from_millis(50));
                    }
                }
            }
        }

        sleep(Duration::from_millis(100));
    }
}
