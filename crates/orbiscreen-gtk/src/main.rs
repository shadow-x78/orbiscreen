// Orbiscreen - GTK4 / Libadwaita Desktop Control Panel GUI (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

use std::rc::Rc;
use std::sync::{Arc, Mutex};

use gtk4::glib;
use gtk4::prelude::*;
use libadwaita::prelude::*;
use libadwaita::{
    ActionRow, Application, ApplicationWindow, HeaderBar, PreferencesGroup, PreferencesPage,
    ToastOverlay,
};
use tracing::{debug, info, warn, Level};
use tracing_subscriber::EnvFilter;

const APP_ID: &str = "com.orbiscreen.OrbiscreenGtk";
const DEFAULT_CONFIG_PATH: &str = "orbiscreen.toml";

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,zbus=error,ashpd=error"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}

#[derive(Debug, Clone, Default)]
struct DaemonStatus {
    available: bool,
    running: bool,
    frames_forwarded: u64,
    active_clients: u64,
    total_clients: u64,
    encoder: String,
    capture_backend: String,
}

impl DaemonStatus {
    fn parse(json: &str) -> Option<Self> {
        let v: serde_json::Value = serde_json::from_str(json).ok()?;
        Some(Self {
            available: true,
            running: v.get("running")?.as_bool()?,
            frames_forwarded: v
                .get("frames_forwarded")
                .and_then(|x| x.as_u64())
                .unwrap_or(0),
            active_clients: v
                .get("active_clients")
                .and_then(|x| x.as_u64())
                .unwrap_or(0),
            total_clients: v.get("total_clients").and_then(|x| x.as_u64()).unwrap_or(0),
            encoder: v
                .get("encoder")
                .and_then(|x| x.as_str())
                .unwrap_or("?")
                .to_string(),
            capture_backend: v
                .get("capture_backend")
                .and_then(|x| x.as_str())
                .unwrap_or("?")
                .to_string(),
        })
    }
}

#[derive(Debug)]
struct DaemonProxy {
    proxy: zbus::Proxy<'static>,
}

impl DaemonProxy {
    async fn connect() -> zbus::Result<Self> {
        let conn = zbus::connection::Builder::session()?.build().await?;
        let proxy = zbus::Proxy::new_owned(
            conn,
            "com.orbiscreen.Daemon",
            "/com/orbiscreen/Daemon",
            "com.orbiscreen.Daemon",
        )
        .await?;
        Ok(Self { proxy })
    }

    async fn get_status(&self) -> zbus::Result<String> {
        self.proxy.call("GetStatus", &()).await
    }

    async fn stop(&self) -> zbus::Result<String> {
        self.proxy.call("Stop", &()).await
    }
}

#[derive(Debug)]
struct UiHandles {
    switch: gtk4::Switch,
    daemon_row: ActionRow,
    stream_row: ActionRow,
    transport_row: ActionRow,
}

fn apply_status(handles: &UiHandles, status: &DaemonStatus, busy: &Mutex<bool>) {
    let is_busy = busy.lock().map(|g| *g).unwrap_or(false);
    if !status.available {
        handles.daemon_row.set_subtitle(
            "Daemon not running — start it with: orbiscreen start \
             (or: systemctl --user start orbiscreen)",
        );
        handles.switch.set_sensitive(false);
        handles
            .stream_row
            .set_subtitle("no daemon on the session bus");
        handles
            .transport_row
            .set_subtitle("waiting for the daemon…");
        if !is_busy {
            handles.switch.set_state(false);
        }
        return;
    }

    handles.switch.set_sensitive(true);
    if !is_busy {
        handles.switch.set_state(status.running);
    }
    if status.running {
        handles.daemon_row.set_subtitle(
            "Running (com.orbiscreen.Daemon) — toggle off to stop the stream gracefully",
        );
        handles.stream_row.set_subtitle(&format!(
            "encoder: {} · source: {} · frames forwarded: {}",
            status.encoder, status.capture_backend, status.frames_forwarded
        ));
        handles.transport_row.set_subtitle(&format!(
            "connected clients: {} active / {} total",
            status.active_clients, status.total_clients
        ));
    } else {
        handles
            .daemon_row
            .set_subtitle("Stopped — D-Bus name is owned but the stream is down");
        handles.stream_row.set_subtitle("stream inactive");
        handles
            .transport_row
            .set_subtitle("connected clients: 0 (stream down)");
    }
}

fn run_dbus_oneshot(
    call: impl FnOnce(
            DaemonProxy,
        )
            -> std::pin::Pin<Box<dyn std::future::Future<Output = std::string::String> + Send>>
        + Send
        + 'static,
    on_done: impl Fn(String) + 'static,
) {
    let (tx, rx) = std::sync::mpsc::channel::<String>();
    std::thread::Builder::new()
        .name("orbiscreen-gtk-stop".into())
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("stop call runtime");
            let result = runtime.block_on(async {
                match DaemonProxy::connect().await {
                    Ok(proxy) => {
                        tokio::time::timeout(std::time::Duration::from_secs(3), call(proxy))
                            .await
                            .unwrap_or_else(|_| "stop request timed out".to_string())
                    }
                    Err(e) => format!("stop failed: {e}"),
                }
            });
            let _ = tx.send(result);
        })
        .expect("spawn stop-call thread");

    glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
        match rx.try_recv() {
            Ok(message) => {
                on_done(message);
                glib::ControlFlow::Break
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => glib::ControlFlow::Break,
        }
    });
}

fn build_ui(app: &Application) {
    let page = PreferencesPage::new();
    page.set_title("Orbiscreen Control Panel");
    page.set_icon_name(Some("display-symbolic"));

    let status_group = PreferencesGroup::new();
    status_group.set_title("Daemon Service Status");
    status_group.set_description(Some(
        "Live state read from the com.orbiscreen.Daemon D-Bus session service",
    ));

    let server_switch = gtk4::Switch::new();
    server_switch.set_active(false);
    server_switch.set_sensitive(false);
    server_switch.set_valign(gtk4::Align::Center);

    let server_row = ActionRow::new();
    server_row.set_title("Orbiscreen Daemon");
    server_row.set_subtitle("querying D-Bus…");
    server_row.add_suffix(&server_switch);
    status_group.add(&server_row);

    let stream_group = PreferencesGroup::new();
    stream_group.set_title("Stream");

    let stream_row = ActionRow::new();
    stream_row.set_title("Encoder & Frame Pipeline");
    stream_row.set_subtitle("waiting for status…");
    stream_group.add(&stream_row);

    let transport_row = ActionRow::new();
    transport_row.set_title("Connected Clients");
    transport_row.set_subtitle("waiting for status…");
    stream_group.add(&transport_row);

    let display_group = PreferencesGroup::new();
    display_group.set_title("Virtual Display Configuration");

    let cfg = std::fs::read_to_string(DEFAULT_CONFIG_PATH)
        .ok()
        .and_then(|s| orbiscreen_core::load_config(&s).ok())
        .unwrap_or_default();

    let resolution_row = ActionRow::new();
    resolution_row.set_title("Virtual Screen Resolution");
    resolution_row.set_subtitle(&format!(
        "{}x{} @ {} Hz (count = {})",
        cfg.display.width, cfg.display.height, cfg.display.refresh_rate_hz, cfg.display.count
    ));
    display_group.add(&resolution_row);

    let port_row = ActionRow::new();
    port_row.set_title("Transport & Network");
    port_row.set_subtitle(&format!(
        "HTTP MPEG-TS /stream on port {}, mDNS advertise: {}",
        cfg.transport.signaling_port,
        if cfg.transport.mdns_advertise {
            "yes"
        } else {
            "no"
        }
    ));
    display_group.add(&port_row);

    page.add(&status_group);
    page.add(&stream_group);
    page.add(&display_group);

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    let header = HeaderBar::new();
    content.append(&header);
    content.append(&page);

    let toast_overlay = ToastOverlay::new();
    toast_overlay.set_child(Some(&content));

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Orbiscreen Secondary Display Control Panel")
        .default_width(680)
        .default_height(560)
        .content(&toast_overlay)
        .build();

    window.present();
    info!("Orbiscreen GTK4 / Libadwaita desktop control panel presented");

    let busy: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    let busy_for_switch = busy.clone();
    let busy_for_reply = busy.clone();
    let toast = toast_overlay.clone();
    server_switch.connect_state_set(move |_, requested| {
        let mut guard = match busy_for_switch.lock() {
            Ok(g) => g,
            Err(_) => return glib::Propagation::Stop,
        };
        if *guard {
            return glib::Propagation::Stop;
        }
        if requested {
            toast.add_toast(libadwaita::Toast::new(
                "Start the daemon with: systemctl --user start orbiscreen (or: orbiscreen start)",
            ));
            return glib::Propagation::Stop;
        }
        *guard = true;
        let toast_for_reply = toast.clone();
        let busy_for_done = busy_for_reply.clone();
        run_dbus_oneshot(
            |proxy| {
                Box::pin(async move {
                    match proxy.stop().await {
                        Ok(reply) => format!("daemon replied: {reply}"),
                        Err(e) => format!("stop failed: {e}"),
                    }
                })
            },
            move |message| {
                info!("{message}");
                toast_for_reply.add_toast(libadwaita::Toast::new(&message));
                if let Ok(mut g) = busy_for_done.lock() {
                    *g = false;
                }
            },
        );
        glib::Propagation::Proceed
    });

    let handles = Rc::new(UiHandles {
        switch: server_switch,
        daemon_row: server_row,
        stream_row,
        transport_row,
    });
    let busy_for_poller = busy.clone();
    start_status_poller(handles, busy_for_poller);
}

fn start_status_poller(handles: Rc<UiHandles>, busy: Arc<Mutex<bool>>) {
    let (tx, rx) = std::sync::mpsc::channel::<DaemonStatus>();

    std::thread::Builder::new()
        .name("orbiscreen-gtk-dbus".into())
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("glib dbus poller runtime");
            runtime.block_on(async move {
                loop {
                    let update = match DaemonProxy::connect().await {
                        Ok(proxy) => match proxy.get_status().await {
                            Ok(json) => match DaemonStatus::parse(&json) {
                                Some(status) => status,
                                None => {
                                    warn!("unparseable daemon status JSON: {json}");
                                    DaemonStatus::default()
                                }
                            },
                            Err(e) => {
                                debug!("GetStatus failed: {e}");
                                DaemonStatus::default()
                            }
                        },
                        Err(e) => {
                            debug!("D-Bus connect failed: {e}");
                            DaemonStatus::default()
                        }
                    };
                    if tx.send(update).is_err() {
                        break;
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            });
        })
        .expect("spawn dbus poller thread");

    glib::timeout_add_local(std::time::Duration::from_millis(250), move || {
        let mut latest = None;
        for update in rx.try_iter() {
            latest = Some(update);
        }
        if let Some(status) = latest {
            apply_status(&handles, &status, &busy);
        }
        glib::ControlFlow::Continue
    });
}

fn main() -> gtk4::glib::ExitCode {
    init_tracing();
    libadwaita::init().expect("Failed to initialize Libadwaita");

    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}
