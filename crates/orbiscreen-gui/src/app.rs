// Orbiscreen - Application Window (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use adw::prelude::*;
use gtk::gio;
use gtk::glib;

use crate::daemon_client::{DaemonClient, DaemonStatus};
use crate::i18n::{self, Lang};
use crate::pages::Pages;
use crate::tray;

pub const APP_ID: &str = "orbiscreen.gui";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Connect,
    Display,
    Doctor,
}

impl Tab {
    const ALL: [Self; 3] = [Self::Connect, Self::Display, Self::Doctor];

    fn name(self) -> &'static str {
        match self {
            Self::Connect => "connect",
            Self::Display => "display",
            Self::Doctor => "doctor",
        }
    }

    fn label_key(self) -> &'static str {
        match self {
            Self::Connect => "tabConnect",
            Self::Display => "tabDisplay",
            Self::Doctor => "tabDoctor",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Self::Connect => "smartphone-symbolic",
            Self::Display => "video-display-symbolic",
            Self::Doctor => "system-run-symbolic",
        }
    }
}

#[derive(Debug)]
pub enum Msg {
    Status(DaemonStatus),
    StartDone(Result<String, String>),
    StopDone(Result<String, String>),
    RestartDone(Result<String, String>),
    Autostart(bool),
    Doctor(Result<orbiscreen_core::report::DoctorReport, String>),
    DoctorFix(Result<String, String>),
    DisplaySet(Result<(u32, u32, u32), String>),
    Toast(String),
}

#[derive(Clone, Debug, Default)]
pub struct AppState {
    pub status: DaemonStatus,
    pub doctor: Option<orbiscreen_core::report::DoctorReport>,
    pub autostart: bool,
    pub lang: Lang,
    pub url: String,
}

#[derive(Clone)]
pub struct PageCtx {
    pub rt: tokio::runtime::Handle,
    pub tx: async_channel::Sender<Msg>,
    pub state: Rc<RefCell<AppState>>,
    pub window: glib::WeakRef<adw::ApplicationWindow>,
}

pub struct Ui {
    pub app: adw::Application,
    pub window: adw::ApplicationWindow,
    header: adw::HeaderBar,
    menu_button: gtk::MenuButton,
    content_box: gtk::Box,
    toast_overlay: adw::ToastOverlay,
    msg_rx: RefCell<Option<async_channel::Receiver<Msg>>>,
    tray_rx: RefCell<Option<async_channel::Receiver<tray::TrayAction>>>,
    tray_update: RefCell<Option<tokio::sync::mpsc::UnboundedSender<tray::TrayUpdate>>>,
    state: Rc<RefCell<AppState>>,
    pages: RefCell<Option<Rc<Pages>>>,
    tab_buttons: RefCell<Vec<gtk::ToggleButton>>,
    tab_sync: Rc<Cell<bool>>,
    current_tab: RefCell<Option<String>>,
    tx: async_channel::Sender<Msg>,
    rt: tokio::runtime::Handle,
}

impl Ui {
    fn new(app: &adw::Application, rt: tokio::runtime::Handle) -> Self {
        let (tx, msg_rx) = async_channel::unbounded::<Msg>();
        let (tray_action_rx, tray_update) = tray::spawn_tray(&rt);

        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Orbiscreen")
            .default_width(760)
            .default_height(720)
            .icon_name("orbiscreen-gui")
            .build();
        window.set_size_request(420, 520);

        let header = adw::HeaderBar::new();

        let menu_button = gtk::MenuButton::new();
        menu_button.set_icon_name("open-menu-symbolic");
        menu_button.add_css_class("orbi-icon-btn");
        header.pack_end(&menu_button);

        let content_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let toolbar = adw::ToolbarView::new();
        toolbar.add_top_bar(&header);
        toolbar.set_content(Some(&content_box));
        let toast_overlay = adw::ToastOverlay::new();
        toast_overlay.set_child(Some(&toolbar));

        window.set_content(Some(&toast_overlay));

        crate::theme::attach(&window);

        window.connect_close_request(|win| {
            win.set_visible(false);
            glib::Propagation::Stop
        });

        let lang = i18n::load_lang();
        gtk::Widget::set_default_direction(match lang {
            Lang::Ar => gtk::TextDirection::Rtl,
            Lang::En => gtk::TextDirection::Ltr,
        });
        let state = Rc::new(RefCell::new(AppState {
            lang,
            ..AppState::default()
        }));

        Self {
            app: app.clone(),
            window,
            header,
            menu_button,
            content_box,
            toast_overlay,
            msg_rx: RefCell::new(Some(msg_rx)),
            tray_rx: RefCell::new(Some(tray_action_rx)),
            tray_update: RefCell::new(Some(tray_update)),
            state,
            pages: RefCell::new(None),
            tab_buttons: RefCell::new(Vec::new()),
            tab_sync: Rc::new(Cell::new(false)),
            current_tab: RefCell::new(None),
            tx,
            rt,
        }
    }

    pub fn launch(app: &adw::Application, rt: tokio::runtime::Handle) -> Rc<Ui> {
        let ui = Rc::new(Ui::new(app, rt));

        let ui_clone = ui.clone();
        if let Some(rx) = ui.msg_rx.borrow_mut().take() {
            glib::MainContext::default().spawn_local(async move {
                while let Ok(msg) = rx.recv().await {
                    ui_clone.handle_msg(msg);
                }
            });
        }

        let ui_clone = ui.clone();
        if let Some(rx) = ui.tray_rx.borrow_mut().take() {
            glib::MainContext::default().spawn_local(async move {
                while let Ok(action) = rx.recv().await {
                    ui_clone.handle_tray_action(action);
                }
            });
        }

        ui.install_actions();
        ui.build_content();
        ui.start_polling();
        ui.refresh_status();
        ui.fetch_autostart();
        ui.window.present();
        ui
    }

    fn install_actions(self: &Rc<Self>) {
        let actions = gio::SimpleActionGroup::new();

        let language = gio::SimpleAction::new("language", None);
        let language_ui = self.clone();
        language.connect_activate(move |_, _| {
            language_ui.toggle_language();
        });
        actions.add_action(&language);

        let restart = gio::SimpleAction::new("restart", None);
        let restart_tx = self.tx.clone();
        let restart_rt = self.rt.clone();
        let restart_lang = self.lang();
        restart.connect_activate(move |_, _| {
            let _ = restart_tx.try_send(Msg::Toast(
                i18n::tr(restart_lang, "toastRestart").to_string(),
            ));
            let tx = restart_tx.clone();
            restart_rt.spawn(async move {
                let _ = tx
                    .send(Msg::RestartDone(DaemonClient::restart_service().await))
                    .await;
            });
        });
        actions.add_action(&restart);

        let about = gio::SimpleAction::new("about", None);
        let about_window = self.window.clone();
        about.connect_activate(move |_, _| {
            let dialog = adw::AboutWindow::builder()
                .application_name("Orbiscreen")
                .version(env!("CARGO_PKG_VERSION"))
                .comments("Secondary monitor for Linux, powered by an Android tablet or phone.")
                .developer_name("shadow-x78")
                .website("https://github.com/shadow-x78/orbiscreen")
                .issue_url("https://github.com/shadow-x78/orbiscreen/issues")
                .license_type(gtk::License::Gpl30)
                .application_icon("orbiscreen-gui")
                .build();
            dialog.set_transient_for(Some(&about_window));
            dialog.present();
        });
        actions.add_action(&about);

        let quit = gio::SimpleAction::new("quit", None);
        let quit_app = self.app.clone();
        quit.connect_activate(move |_, _| {
            quit_app.quit();
        });
        actions.add_action(&quit);

        self.window.insert_action_group("win", Some(&actions));
        self.app.set_accels_for_action("win.quit", &["<Primary>Q"]);
    }

    fn build_content(&self) {
        let lang = self.state.borrow().lang;
        let t = |key: &'static str| i18n::tr(lang, key);

        let stack = adw::ViewStack::new();
        let ctx = PageCtx {
            rt: self.rt.clone(),
            tx: self.tx.clone(),
            state: self.state.clone(),
            window: self.window.downgrade(),
        };
        let pages = Rc::new(Pages::new(&ctx));

        for tab in Tab::ALL {
            let root = match tab {
                Tab::Connect => &pages.connect.root,
                Tab::Display => &pages.display.root,
                Tab::Doctor => &pages.doctor.root,
            };
            stack.add_titled_with_icon(root, Some(tab.name()), t(tab.label_key()), tab.icon());
        }

        let initial_tab = self
            .current_tab
            .borrow()
            .clone()
            .unwrap_or_else(|| "connect".to_string());
        stack.set_visible_child_name(&initial_tab);

        let tabs_box = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        tabs_box.set_homogeneous(true);
        let mut tab_buttons = Vec::new();
        for tab in Tab::ALL {
            let button = gtk::ToggleButton::new();
            let content = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            let image = gtk::Image::from_icon_name(tab.icon());
            image.set_pixel_size(16);
            let label = gtk::Label::new(Some(t(tab.label_key())));
            content.append(&image);
            content.append(&label);
            button.set_child(Some(&content));
            button.add_css_class("orbi-tab");
            button.set_active(tab.name() == initial_tab);
            let tab_name = tab.name().to_string();
            let weak_stack = stack.downgrade();
            let sync = self.tab_sync.clone();
            button.connect_toggled(move |source| {
                if !source.is_active() || sync.get() {
                    return;
                }
                if let Some(stack) = weak_stack.upgrade() {
                    stack.set_visible_child_name(&tab_name);
                }
            });
            tabs_box.append(&button);
            tab_buttons.push(button);
        }
        let tabs = adw::Bin::new();
        tabs.add_css_class("orbi-tabs");
        tabs.set_child(Some(&tabs_box));
        self.header.set_title_widget(Some(&tabs));
        *self.tab_buttons.borrow_mut() = tab_buttons;

        let current = self.current_tab.clone();
        let sync_buttons = self.tab_buttons.borrow().clone();
        let sync_flag = self.tab_sync.clone();
        stack.connect_visible_child_name_notify(move |source| {
            if let Some(name) = source.visible_child_name() {
                *current.borrow_mut() = Some(name.to_string());
                if !sync_flag.get() {
                    sync_flag.set(true);
                    for (index, button) in sync_buttons.iter().enumerate() {
                        let tab = Tab::ALL.get(index).copied().unwrap_or(Tab::Connect);
                        button.set_active(tab.name() == name);
                    }
                    sync_flag.set(false);
                }
            }
        });

        if let Some(old) = self.content_box.first_child() {
            self.content_box.remove(&old);
        }
        self.content_box.append(&stack);

        let menu = gio::Menu::new();
        let language_label = if lang == Lang::Ar {
            "English"
        } else {
            "العربية"
        };
        menu.append(Some(language_label), Some("win.language"));
        menu.append(Some(t("restartTitle")), Some("win.restart"));
        menu.append(Some(t("about")), Some("win.about"));
        menu.append(Some(t("quit")), Some("win.quit"));
        self.menu_button.set_menu_model(Some(&menu));

        *self.pages.borrow_mut() = Some(pages);

        self.apply_state();
    }

    fn toggle_language(&self) {
        let new_lang = self.state.borrow().lang.other();
        self.state.borrow_mut().lang = new_lang;
        i18n::save_lang(new_lang);
        gtk::Widget::set_default_direction(match new_lang {
            Lang::Ar => gtk::TextDirection::Rtl,
            Lang::En => gtk::TextDirection::Ltr,
        });
        self.build_content();
    }

    fn apply_state(&self) {
        let snapshot = self.state.borrow().clone();
        if let Some(pages) = self.pages.borrow().as_ref() {
            pages.update(&snapshot);
        }
        if let Some(sender) = self.tray_update.borrow().as_ref() {
            let _ = sender.send(tray::TrayUpdate {
                running: snapshot.status.running,
                clients: snapshot.status.active_clients,
            });
        }
    }

    fn toast(&self, message: &str) {
        self.toast_overlay.add_toast(adw::Toast::new(message));
    }

    fn refresh_status(&self) {
        spawn_status_fetch(&self.rt, &self.tx);
    }

    fn fetch_autostart(&self) {
        let tx = self.tx.clone();
        self.rt.spawn(async move {
            let _ = tx
                .send(Msg::Autostart(DaemonClient::is_autostart_enabled()))
                .await;
        });
    }

    fn start_polling(&self) {
        let tx = self.tx.clone();
        let rt = self.rt.clone();
        glib::timeout_add_seconds_local(3, move || {
            spawn_status_fetch(&rt, &tx);
            glib::ControlFlow::Continue
        });
    }

    fn handle_msg(&self, msg: Msg) {
        match msg {
            Msg::Status(status) => {
                {
                    let mut state = self.state.borrow_mut();
                    state.url = session_url(&status);
                    state.status = status;
                }
                self.apply_state();
            }
            Msg::StartDone(result) => {
                match result {
                    Ok(_) => self.toast(i18n::tr(self.lang(), "toastStarted")),
                    Err(e) => self.toast(&format!("Error: {e}")),
                }
                self.refresh_status();
            }
            Msg::StopDone(result) => {
                match result {
                    Ok(_) => self.toast(i18n::tr(self.lang(), "toastStopped")),
                    Err(e) => self.toast(&format!("Error: {e}")),
                }
                self.refresh_status();
            }
            Msg::RestartDone(result) => match result {
                Ok(_) => {
                    self.toast(i18n::tr(self.lang(), "toastRestartDone"));
                    self.refresh_status();
                }
                Err(e) => self.toast(&format!("Error: {e}")),
            },
            Msg::Autostart(enabled) => {
                self.state.borrow_mut().autostart = enabled;
                if let Some(pages) = self.pages.borrow().as_ref() {
                    pages.set_autostart(enabled);
                }
            }
            Msg::Doctor(result) => {
                if let Some(pages) = self.pages.borrow().as_ref() {
                    pages.doctor_scan_finished();
                }
                match result {
                    Ok(report) => {
                        self.state.borrow_mut().doctor = Some(report);
                        self.apply_state();
                    }
                    Err(e) => {
                        tracing::warn!("doctor check failed: {e}");
                        self.toast(i18n::tr(self.lang(), "toastDoctorFailed"));
                    }
                }
            }
            Msg::DoctorFix(result) => {
                if let Some(pages) = self.pages.borrow().as_ref() {
                    pages.doctor_fix_finished();
                }
                match result {
                    Ok(_) => self.toast(i18n::tr(self.lang(), "toastFixDone")),
                    Err(e) => {
                        tracing::warn!("doctor fix failed: {e}");
                        self.toast(i18n::tr(self.lang(), "toastFixFailed"));
                    }
                }
            }
            Msg::DisplaySet(result) => match result {
                Ok((w, h, fps)) => {
                    {
                        let mut state = self.state.borrow_mut();
                        state.status.display_width = w;
                        state.status.display_height = h;
                        state.status.display_fps = fps;
                    }
                    self.apply_state();
                }
                Err(e) => {
                    tracing::warn!("failed to apply display settings: {e}");
                    self.toast(i18n::tr(self.lang(), "toastSetFailed"));
                    self.refresh_status();
                }
            },
            Msg::Toast(message) => self.toast(&message),
        }
    }

    fn handle_tray_action(&self, action: tray::TrayAction) {
        match action {
            tray::TrayAction::Open => self.window.present(),
            tray::TrayAction::Start => {
                let tx = self.tx.clone();
                self.rt.spawn(async move {
                    let _ = DaemonClient::start_service().await;
                    let _ = tx.send(Msg::Status(DaemonClient::get_status().await)).await;
                });
            }
            tray::TrayAction::Stop => {
                let tx = self.tx.clone();
                self.rt.spawn(async move {
                    let _ = DaemonClient::stop_service().await;
                    let _ = tx.send(Msg::Status(DaemonClient::get_status().await)).await;
                });
            }
            tray::TrayAction::Quit => self.app.quit(),
        }
    }

    fn lang(&self) -> Lang {
        self.state.borrow().lang
    }
}

fn spawn_status_fetch(rt: &tokio::runtime::Handle, tx: &async_channel::Sender<Msg>) {
    let tx = tx.clone();
    rt.spawn(async move {
        let _ = tx.send(Msg::Status(DaemonClient::get_status().await)).await;
    });
}

fn session_url(status: &DaemonStatus) -> String {
    let token = status.token();
    session_url_with(status, token.as_deref())
}

fn session_url_with(status: &DaemonStatus, token: Option<&str>) -> String {
    let ip = status
        .local_ips
        .first()
        .map(String::as_str)
        .filter(|ip| !ip.is_empty())
        .unwrap_or("127.0.0.1");
    let port = if status.signaling_port == 0 {
        8788
    } else {
        status.signaling_port
    };
    let token_part = token
        .map(|token| format!("#token={token}"))
        .unwrap_or_default();
    format!("http://{ip}:{port}/client/{token_part}")
}

pub fn usb_url(status: &DaemonStatus) -> String {
    let token = status.token();
    usb_url_with(status, token.as_deref())
}

fn usb_url_with(status: &DaemonStatus, token: Option<&str>) -> String {
    let port = if status.signaling_port == 0 {
        8788
    } else {
        status.signaling_port
    };
    let token_part = token
        .map(|token| format!("#token={token}"))
        .unwrap_or_default();
    format!("http://127.0.0.1:{port}/client/{token_part}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn status(ip: &str, port: u16) -> DaemonStatus {
        let mut status = DaemonStatus::new(orbiscreen_core::report::StatusReport {
            signaling_port: port,
            ..orbiscreen_core::report::StatusReport::new()
        });
        status.local_ips = vec![ip.to_string()];
        status
    }

    #[test]
    fn session_url_uses_the_lan_ip_and_signaling_port() {
        let url = session_url_with(&status("192.168.1.5", 8788), None);
        assert_eq!(url, "http://192.168.1.5:8788/client/");
    }

    #[test]
    fn session_url_carries_the_token_fragment() {
        let url = session_url_with(&status("192.168.1.5", 8788), Some("abc123"));
        assert_eq!(url, "http://192.168.1.5:8788/client/#token=abc123");
    }

    #[test]
    fn usb_url_pins_loopback_but_keeps_the_token() {
        let url = usb_url_with(&status("192.168.1.5", 9000), Some("abc123"));
        assert_eq!(url, "http://127.0.0.1:9000/client/#token=abc123");
    }

    #[test]
    fn session_url_falls_back_to_loopback_and_default_port() {
        let url = session_url_with(&status("", 0), None);
        assert_eq!(url, "http://127.0.0.1:8788/client/");
    }
}
