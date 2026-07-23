// Orbiscreen - GTK4 / Libadwaita Desktop Control Panel GUI (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

use gtk4::prelude::*;
use libadwaita::prelude::*;
use libadwaita::{
    ActionRow, Application, ApplicationWindow, HeaderBar, PreferencesGroup, PreferencesPage,
};
use tracing::{info, Level};
use tracing_subscriber::EnvFilter;

const APP_ID: &str = "com.orbiscreen.OrbiscreenGtk";

fn init_tracing() {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(Level::INFO.as_str()));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}

fn build_ui(app: &Application) {
    let page = PreferencesPage::new();
    page.set_title("Orbiscreen Control Panel");
    page.set_icon_name(Some("display-symbolic"));

    // Server Status & Controls Group
    let status_group = PreferencesGroup::new();
    status_group.set_title("Daemon Service Status");
    status_group.set_description(Some(
        "Control background secondary display streaming service",
    ));

    let server_switch = gtk4::Switch::new();
    server_switch.set_active(true);
    server_switch.set_valign(gtk4::Align::Center);

    let server_row = ActionRow::new();
    server_row.set_title("Orbiscreen Daemon");
    server_row.set_subtitle("D-Bus Session Service: Active (com.orbiscreen.Daemon)");
    server_row.add_suffix(&server_switch);
    status_group.add(&server_row);

    // Display Settings Group
    let display_group = PreferencesGroup::new();
    display_group.set_title("Virtual Display Configuration");

    let resolution_row = ActionRow::new();
    resolution_row.set_title("Virtual Screen Resolution");
    resolution_row.set_subtitle("Default: 1920x1080 @ 60Hz");
    display_group.add(&resolution_row);

    let transport_row = ActionRow::new();
    transport_row.set_title("Transport & Network");
    transport_row.set_subtitle("HTTP Direct Stream /stream (Port 8788) & WebRTC");
    display_group.add(&transport_row);

    page.add(&status_group);
    page.add(&display_group);

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    let header = HeaderBar::new();
    content.append(&header);
    content.append(&page);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Orbiscreen Secondary Display Control Panel")
        .default_width(680)
        .default_height(540)
        .content(&content)
        .build();

    window.present();
    info!("Orbiscreen GTK4 / Libadwaita desktop control panel presented");
}

fn main() -> gtk4::glib::ExitCode {
    init_tracing();
    libadwaita::init().expect("Failed to initialize Libadwaita");

    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}
