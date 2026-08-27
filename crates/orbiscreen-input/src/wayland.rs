// Orbiscreen - orbiscreen-input - wayland module (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen
use ashpd::desktop::remote_desktop::{
    DeviceType, KeyState, RemoteDesktop, SelectDevicesOptions, SelectedDevices,
};
use ashpd::desktop::{PersistMode, ResponseError, Session};
use enumflags2::BitFlags;
use orbiscreen_core::portal_state;
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

fn device_options() -> SelectDevicesOptions {
    SelectDevicesOptions::default()
        .set_devices(Some(
            BitFlags::from(DeviceType::Keyboard) | BitFlags::from(DeviceType::Pointer),
        ))
        .set_persist_mode(PersistMode::ExplicitlyRevoked)
}

async fn negotiate_remote_desktop(
    remote: &RemoteDesktop,
    restore_token: Option<&str>,
) -> Result<(Session<RemoteDesktop>, SelectedDevices), WaylandInputError> {
    let session = remote
        .create_session(Default::default())
        .await
        .map_err(|e| WaylandInputError::Dbus(e.to_string()))?;
    remote
        .select_devices(&session, device_options().set_restore_token(restore_token))
        .await
        .map_err(|e| WaylandInputError::Dbus(e.to_string()))?;
    let selected = remote
        .start(&session, None, Default::default())
        .await
        .map_err(|e| WaylandInputError::Dbus(e.to_string()))?
        .response()
        .map_err(|e| match e {
            ashpd::Error::Response(ResponseError::Cancelled) => WaylandInputError::PermissionDenied,
            other => WaylandInputError::Dbus(other.to_string()),
        })?;
    Ok((session, selected))
}

impl WaylandInjector {
    #[instrument(skip_all)]
    pub async fn open() -> Result<Self, WaylandInputError> {
        let remote = RemoteDesktop::new()
            .await
            .map_err(|e| WaylandInputError::PortalUnavailable(e.to_string()))?;
        let mut state = portal_state::load_portal_state();
        let saved_token = state.remote_desktop_restore_token.clone();
        let (session, selected) =
            match negotiate_remote_desktop(&remote, saved_token.as_deref()).await {
                Ok(pair) => pair,
                Err(first_error) => {
                    if saved_token.is_some() {
                        warn!(
                            "saved remote-desktop restore token was not accepted \
                             ({first_error}); asking for a fresh grant"
                        );
                        negotiate_remote_desktop(&remote, None).await?
                    } else {
                        return Err(first_error);
                    }
                }
            };
        if let Some(token) = selected.restore_token() {
            state.remote_desktop_restore_token = Some(token.to_string());
            match portal_state::save_portal_state(&state) {
                Ok(()) => info!(
                    "remote-desktop permission persisted — the grant dialog will not \
                     reappear on the next runs"
                ),
                Err(e) => warn!("failed to persist portal state: {e}"),
            }
        }
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
