// Orbiscreen - System Tray (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

use gtk::prelude::*;
use ksni::menu::{MenuItem, StandardItem};
use ksni::{Icon, ToolTip, Tray, TrayMethods};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TrayAction {
    Open,
    Start,
    Stop,
    Quit,
}

pub struct OrbiscreenTray {
    tx: async_channel::Sender<TrayAction>,
    running: bool,
    clients: usize,
    pixmap: Vec<Icon>,
}

impl OrbiscreenTray {
    pub fn new(tx: async_channel::Sender<TrayAction>, pixmap: Vec<Icon>) -> Self {
        Self {
            tx,
            running: false,
            clients: 0,
            pixmap,
        }
    }
}

impl Tray for OrbiscreenTray {
    fn id(&self) -> String {
        "orbiscreen-gui".into()
    }

    fn title(&self) -> String {
        "Orbiscreen".into()
    }

    fn icon_name(&self) -> String {
        "orbiscreen-gui".into()
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        self.pixmap.clone()
    }

    fn tool_tip(&self) -> ToolTip {
        let description = if !self.running {
            "Stopped".to_string()
        } else if self.clients > 0 {
            format!("Streaming to {client} device(s)", client = self.clients)
        } else {
            "Ready".to_string()
        };
        ToolTip {
            title: "Orbiscreen Host Control Center".into(),
            description,
            ..ToolTip::default()
        }
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        vec![
            MenuItem::Standard(StandardItem {
                label: "Open Dashboard".into(),
                activate: Box::new(|tray: &mut Self| {
                    let _ = tray.tx.try_send(TrayAction::Open);
                }),
                ..Default::default()
            }),
            MenuItem::Separator,
            MenuItem::Standard(StandardItem {
                label: "Start Service".into(),
                activate: Box::new(|tray: &mut Self| {
                    let _ = tray.tx.try_send(TrayAction::Start);
                }),
                ..Default::default()
            }),
            MenuItem::Standard(StandardItem {
                label: "Stop Service".into(),
                activate: Box::new(|tray: &mut Self| {
                    let _ = tray.tx.try_send(TrayAction::Stop);
                }),
                ..Default::default()
            }),
            MenuItem::Separator,
            MenuItem::Standard(StandardItem {
                label: "Quit".into(),
                activate: Box::new(|tray: &mut Self| {
                    let _ = tray.tx.try_send(TrayAction::Quit);
                }),
                ..Default::default()
            }),
        ]
    }
}

pub struct TrayUpdate {
    pub running: bool,
    pub clients: usize,
}

pub fn spawn_tray(
    rt: &tokio::runtime::Handle,
) -> (
    async_channel::Receiver<TrayAction>,
    tokio::sync::mpsc::UnboundedSender<TrayUpdate>,
) {
    let (action_tx, action_rx) = async_channel::unbounded::<TrayAction>();
    let (update_tx, mut update_rx) = tokio::sync::mpsc::unbounded_channel::<TrayUpdate>();

    let tray = OrbiscreenTray::new(action_tx, embedded_icon());
    let spawn_rt = rt.clone();
    spawn_rt.spawn(async move {
        let handle = match tray.assume_sni_available(true).spawn().await {
            Ok(handle) => handle,
            Err(e) => {
                tracing::warn!("system tray unavailable: {e}");
                return;
            }
        };
        while let Some(update) = update_rx.recv().await {
            let running = update.running;
            let clients = update.clients;
            let _ = handle
                .update(move |t| {
                    t.running = running;
                    t.clients = clients;
                })
                .await;
        }
    });

    (action_rx, update_tx)
}

fn embedded_icon() -> Vec<Icon> {
    let bytes = include_bytes!("../icons/48x48.png");
    let loader = gtk::gdk_pixbuf::PixbufLoader::new();
    if loader.write(bytes).is_err() || loader.close().is_err() {
        return Vec::new();
    }
    let Some(pixbuf) = loader.pixbuf() else {
        return Vec::new();
    };
    let Some(raw) = pixbuf.pixel_bytes() else {
        return Vec::new();
    };
    let raw = raw.as_ref();
    let width = pixbuf.width();
    let height = pixbuf.height();
    let rowstride = pixbuf.rowstride() as usize;
    let channels = pixbuf.n_channels() as usize;
    if channels != 3 && channels != 4 {
        return Vec::new();
    }

    let mut data = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            let offset = y as usize * rowstride + x as usize * channels;
            let (r, g, b, a) = match channels {
                4 => (
                    raw[offset],
                    raw[offset + 1],
                    raw[offset + 2],
                    raw[offset + 3],
                ),
                _ => (raw[offset], raw[offset + 1], raw[offset + 2], 255),
            };
            data.extend_from_slice(&[a, r, g, b]);
        }
    }
    vec![Icon {
        width,
        height,
        data,
    }]
}
