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

            while device_state.get_keys().contains(&Keycode::M) {
                sleep(Duration::from_millis(50));
            }
        }

        sleep(Duration::from_millis(100));
    }
}
