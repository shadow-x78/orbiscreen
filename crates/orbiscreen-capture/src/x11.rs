// Orbiscreen - orbiscreen-capture - x11 module (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen
use std::sync::Arc;

use tokio::task;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ConnectionExt, ImageFormat, Screen};
use x11rb::xcb_ffi::XCBConnection;

use super::{CaptureError, CapturedFrame};

#[allow(missing_debug_implementations)]
pub struct X11Capture {
    conn: Arc<XCBConnection>,
    screen: Screen,
    width: u32,
    height: u32,
}

impl X11Capture {
    pub fn open(width: u32, height: u32) -> Result<Self, CaptureError> {
        let (conn, screen_num) = XCBConnection::connect(None)?;
        let screen = conn.setup().roots[screen_num].clone();
        let cap_w = width.min(screen.width_in_pixels as u32);
        let cap_h = height.min(screen.height_in_pixels as u32);
        if cap_w != width || cap_h != height {
            tracing::warn!(
                requested = format!("{width}x{height}"),
                actual = format!("{cap_w}x{cap_h}"),
                "capture region clamped to root window size"
            );
        }
        Ok(Self {
            conn: Arc::new(conn),
            screen,
            width: cap_w,
            height: cap_h,
        })
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub async fn next_frame(&self) -> Result<CapturedFrame, CaptureError> {
        let conn = self.conn.clone();
        let screen = self.screen.clone();
        let width = self.width;
        let height = self.height;
        task::spawn_blocking(move || capture_blocking(&conn, &screen, width, height))
            .await
            .map_err(|e| CaptureError::Io(format!("join error: {e}")))?
    }
}

fn capture_blocking(
    conn: &XCBConnection,
    screen: &Screen,
    width: u32,
    height: u32,
) -> Result<CapturedFrame, CaptureError> {
    let cookie = conn.get_image(
        ImageFormat::Z_PIXMAP,
        screen.root,
        0,
        0,
        width as u16,
        height as u16,
        u32::MAX,
    );
    let reply = cookie?.reply().map_err(|e| match e {
        x11rb::errors::ReplyError::X11Error(err) => CaptureError::X11Protocol(err.error_code),
        x11rb::errors::ReplyError::ConnectionError(err) => {
            CaptureError::X11Connect(err.to_string())
        }
    })?;
    let mut data = reply.data;
    let expected = CapturedFrame::size_in_bytes(width, height);
    if data.len() < expected {
        return Err(CaptureError::Io(format!(
            "GetImage reply truncated: need {expected} B, have {}",
            data.len()
        )));
    }
    if data.len() > expected {
        data.truncate(expected);
    }
    Ok(CapturedFrame {
        width,
        height,
        data,
    })
}
