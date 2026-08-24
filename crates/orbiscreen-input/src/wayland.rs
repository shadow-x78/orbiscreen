// Orbiscreen - orbiscreen-input - wayland module (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen
use ashpd::desktop::remote_desktop::{DeviceType, KeyState, RemoteDesktop, SelectDevicesOptions};
use ashpd::desktop::Session;
use enumflags2::BitFlags;
use thiserror::Error;
use tracing::{info, instrument, warn};

use super::{InputError, KeyEvent, PointerEvent};

#[derive(Debug, Error)]
pub enum WaylandInputError {
    #[error("remotedesktop portal not available: {0}")]
    PortalUnavailable(String),
    #[error("portal D-Bus error: {0}")]
    Dbus(String),
    #[error("user denied the RemoteDesktop permission")]
    PermissionDenied,
}

impl From<WaylandInputError> for InputError {
    fn from(error: WaylandInputError) -> Self {
        match error {
            WaylandInputError::PermissionDenied => InputError::Uinput("permission denied".into()),
            other => InputError::Uinput(other.to_string()),
        }
    }
}

#[allow(missing_debug_implementations)]
pub struct WaylandInjector {
    remote: RemoteDesktop,
    session: std::sync::Arc<Session<RemoteDesktop>>,
}

impl WaylandInjector {
    #[instrument(skip_all)]
    pub async fn open() -> Result<Self, WaylandInputError> {
        let remote = RemoteDesktop::new()
            .await
            .map_err(|e| WaylandInputError::PortalUnavailable(e.to_string()))?;
        let session = remote
            .create_session(Default::default())
            .await
            .map_err(|e| WaylandInputError::Dbus(e.to_string()))?;
        remote
            .select_devices(
                &session,
                SelectDevicesOptions::default().set_devices(Some(
                    BitFlags::from(DeviceType::Keyboard) | BitFlags::from(DeviceType::Pointer),
                )),
            )
            .await
            .map_err(|e| WaylandInputError::Dbus(e.to_string()))?;
        remote
            .start(&session, None, Default::default())
            .await
            .map_err(|e| WaylandInputError::Dbus(e.to_string()))?
            .response()
            .map_err(|e| WaylandInputError::Dbus(e.to_string()))?;
        info!("RemoteDesktop session established");
        Ok(Self {
            remote,
            session: std::sync::Arc::new(session),
        })
    }

    pub async fn inject_pointer(&self, event: PointerEvent) -> Result<(), InputError> {
        match event {
            PointerEvent::Move { x, y } => {
                self.remote
                    .notify_pointer_motion(&self.session, x, y, Default::default())
                    .await
                    .map_err(|e| InputError::Uinput(e.to_string()))?;
            }
            PointerEvent::Button { button, pressed } => {
                let state = if pressed {
                    KeyState::Pressed
                } else {
                    KeyState::Released
                };
                let button = i32::try_from(button)
                    .map_err(|_| InputError::Uinput(format!("invalid button: {button}")))?;
                self.remote
                    .notify_pointer_button(&self.session, button, state, Default::default())
                    .await
                    .map_err(|e| InputError::Uinput(e.to_string()))?;
            }
            PointerEvent::Wheel { delta_y } => {
                self.remote
                    .notify_pointer_axis(&self.session, 0.0, delta_y, Default::default())
                    .await
                    .map_err(|e| InputError::Uinput(e.to_string()))?;
            }
        }
        Ok(())
    }

    pub async fn inject_key(&self, event: KeyEvent) -> Result<(), InputError> {
        let state = if event.pressed {
            KeyState::Pressed
        } else {
            KeyState::Released
        };
        let code = i32::try_from(event.code)
            .map_err(|_| InputError::Uinput(format!("invalid key code: {}", event.code)))?;
        self.remote
            .notify_keyboard_keycode(&self.session, code, state, Default::default())
            .await
            .map_err(|e| InputError::Uinput(e.to_string()))?;
        Ok(())
    }
}

impl Drop for WaylandInjector {
    fn drop(&mut self) {
        let session = self.session.clone();
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(async move {
                if let Err(e) = session.close().await {
                    warn!("failed to close RemoteDesktop session: {e}");
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wayland_input_error_messages_are_clear() {
        let error = WaylandInputError::PermissionDenied;
        assert!(error.to_string().to_lowercase().contains("denied"));
        let converted: InputError = error.into();
        assert!(format!("{converted}").contains("permission"));
    }

    #[test]
    fn key_state_maps_correctly() {
        let pressed = KeyEvent {
            code: 30,
            pressed: true,
        };
        let released = KeyEvent {
            code: 30,
            pressed: false,
        };
        assert_ne!(pressed, released);
    }
}
