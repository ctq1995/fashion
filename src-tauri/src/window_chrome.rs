use std::{ffi::c_void, mem::size_of};

use tauri::{Runtime, WebviewWindow, WindowEvent};
use windows::Win32::Graphics::{
    Dwm::{
        DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_DONOTROUND, DWMWCP_ROUND,
        DWM_WINDOW_CORNER_PREFERENCE,
    },
    Gdi::{CreateRoundRectRgn, SetWindowRgn},
};
use windows_version::OsVersion;

const APP_CORNER_RADIUS: f64 = 10.0;
const WINDOWS_11_BUILD: u32 = 22_000;
const FILL_BOUNDS_TOLERANCE: i64 = 24;

pub fn install<R: Runtime>(window: &WebviewWindow<R>) {
    apply_native_chrome(window);

    let tracked_window = window.clone();
    window.on_window_event(move |event| {
        if matches!(
            event,
            WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. }
        ) {
            apply_native_chrome(&tracked_window);
        }
    });
}

fn apply_native_chrome<R: Runtime>(window: &WebviewWindow<R>) {
    let os_version = OsVersion::current();
    let fill_bounds = is_fill_bounds(window);
    let _ = window.set_shadow(os_version.build >= WINDOWS_11_BUILD && !fill_bounds);

    if os_version.build >= WINDOWS_11_BUILD {
        apply_windows_11_corners(window, fill_bounds);
    } else {
        apply_windows_10_region(window, fill_bounds);
    }
}

fn is_fill_bounds<R: Runtime>(window: &WebviewWindow<R>) -> bool {
    if window.is_fullscreen().unwrap_or(false) || window.is_maximized().unwrap_or(false) {
        return true;
    }

    let Ok(Some(monitor)) = window.current_monitor() else {
        return false;
    };
    let Ok(position) = window.outer_position() else {
        return false;
    };
    let Ok(size) = window.outer_size() else {
        return false;
    };

    let work_area = monitor.work_area();
    let screen_pos = monitor.position();
    let screen_size = monitor.size();

    let tolerance = FILL_BOUNDS_TOLERANCE;

    let fits_work_area = (position.x - work_area.position.x).abs() as i64 <= tolerance
        && (position.y - work_area.position.y).abs() as i64 <= tolerance
        && (size.width as i64 - work_area.size.width as i64).abs() <= tolerance
        && (size.height as i64 - work_area.size.height as i64).abs() <= tolerance;

    let fits_screen = (position.x - screen_pos.x).abs() as i64 <= tolerance
        && (position.y - screen_pos.y).abs() as i64 <= tolerance
        && (size.width as i64 - screen_size.width as i64).abs() <= tolerance
        && (size.height as i64 - screen_size.height as i64).abs() <= tolerance;

    let pos_x = position.x as i64;
    let pos_y = position.y as i64;
    let right = pos_x + size.width as i64;
    let bottom = pos_y + size.height as i64;

    let work_left = work_area.position.x as i64;
    let work_top = work_area.position.y as i64;
    let work_right = work_left + work_area.size.width as i64;
    let work_bottom = work_top + work_area.size.height as i64;
    let covers_work_area = pos_x <= work_left + tolerance
        && pos_y <= work_top + tolerance
        && right >= work_right - tolerance
        && bottom >= work_bottom - tolerance;

    let screen_left = screen_pos.x as i64;
    let screen_top = screen_pos.y as i64;
    let screen_right = screen_left + screen_size.width as i64;
    let screen_bottom = screen_top + screen_size.height as i64;
    let covers_screen = pos_x <= screen_left + tolerance
        && pos_y <= screen_top + tolerance
        && right >= screen_right - tolerance
        && bottom >= screen_bottom - tolerance;

    fits_work_area || fits_screen || covers_work_area || covers_screen
}

fn apply_windows_11_corners<R: Runtime>(window: &WebviewWindow<R>, fill_bounds: bool) {
    let Ok(hwnd) = window.hwnd() else {
        return;
    };

    let preference = if fill_bounds {
        DWMWCP_DONOTROUND
    } else {
        DWMWCP_ROUND
    };
    let _ = unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &preference as *const DWM_WINDOW_CORNER_PREFERENCE as *const c_void,
            size_of::<DWM_WINDOW_CORNER_PREFERENCE>() as u32,
        )
    };
}

fn apply_windows_10_region<R: Runtime>(window: &WebviewWindow<R>, fill_bounds: bool) {
    let Ok(hwnd) = window.hwnd() else {
        return;
    };

    if window.is_maximized().unwrap_or(false) || fill_bounds {
        let _ = unsafe { SetWindowRgn(hwnd, None, true) };
        return;
    }

    let Ok(size) = window.inner_size() else {
        return;
    };

    if size.width == 0 || size.height == 0 {
        return;
    }

    let scale_factor = window.scale_factor().unwrap_or(1.0);
    let corner_radius = ((APP_CORNER_RADIUS * scale_factor).round() as i32).max(1);
    let diameter = corner_radius * 2;
    let width = size.width.min(i32::MAX as u32) as i32;
    let height = size.height.min(i32::MAX as u32) as i32;

    // Win10 does not provide rounded undecorated windows, so clip the native region manually.
    let region = unsafe { CreateRoundRectRgn(0, 0, width + 1, height + 1, diameter, diameter) };
    if region.0.is_null() {
        return;
    }

    let _ = unsafe { SetWindowRgn(hwnd, Some(region), true) };
}
