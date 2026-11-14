// Orbiscreen - Connect Page (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

use adw::prelude::*;
use gtk::glib;

use crate::app::{AppState, Msg, PageCtx};
use crate::daemon_client::DaemonClient;
use crate::i18n::{self, Lang};
use crate::pages::{self as ui};
use crate::qr::QrWidget;

pub struct ConnectPage {
    pub root: gtk::Widget,
    hero_title: gtk::Label,
    hero_subtitle: gtk::Label,
    hero_pill: gtk::Label,
    resolution_value: gtk::Label,
    encoder_value: gtk::Label,
    clients_value: gtk::Label,
    url_value: gtk::Label,
    qr: QrWidget,
    usb_pill: gtk::Label,
    usb_status: gtk::Label,
    toggle_button: gtk::Button,
    lang: Lang,
}

impl ConnectPage {
    pub fn new(ctx: &PageCtx) -> Self {
        let lang = ctx.state.borrow().lang;
        let t = |key: &'static str| i18n::tr(lang, key);

        let (root, content) = ui::shell();

        let hero = ui::card();
        let hero_row = gtk::Box::new(gtk::Orientation::Horizontal, 14);
        hero_row.append(&ui::icon_chip("video-display-symbolic", ""));
        let hero_text = gtk::Box::new(gtk::Orientation::Vertical, 4);
        let title_row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        let hero_title = gtk::Label::new(Some(t("statusOfflineHeadline")));
        hero_title.add_css_class("orbi-hero-title");
        hero_title.set_halign(gtk::Align::Start);
        title_row.append(&hero_title);
        let hero_pill = ui::pill(&i18n::tr(lang, "stopped").to_uppercase(), "off");
        hero_pill.set_valign(gtk::Align::Baseline);
        title_row.append(&hero_pill);
        hero_text.append(&title_row);
        let hero_subtitle = ui::dim_label(t("statusOfflineSubtitle"));
        hero_subtitle.set_halign(gtk::Align::Start);
        hero_subtitle.set_wrap(true);
        hero_subtitle.set_xalign(0.0);
        hero_text.append(&hero_subtitle);
        hero_row.append(&hero_text);
        hero.append(&hero_row);

        let resolution_row = ui::spec_row(t("resolution"));
        resolution_row.add_css_class("orbi-sep");
        let resolution_value = ui::value_label("-");
        resolution_row.append(&resolution_value);
        let encoder_row = ui::spec_row(t("encoder"));
        let encoder_value = ui::value_label("-");
        encoder_row.append(&encoder_value);
        let clients_row = ui::spec_row(t("devices"));
        let clients_value = ui::value_label(t("noDevices"));
        clients_row.append(&clients_value);
        hero.append(&resolution_row);
        hero.append(&encoder_row);
        hero.append(&clients_row);

        let wifi = ui::card();
        let wifi_layout = gtk::Box::new(gtk::Orientation::Horizontal, 20);
        wifi_layout.set_valign(gtk::Align::Center);
        let qr = QrWidget::new(190);
        let qr_bin = adw::Bin::new();
        qr_bin.add_css_class("orbi-qr");
        qr_bin.set_overflow(gtk::Overflow::Hidden);
        qr_bin.set_child(Some(qr.widget()));
        qr_bin.set_valign(gtk::Align::Center);
        wifi_layout.append(&qr_bin);

        let wifi_text = gtk::Box::new(gtk::Orientation::Vertical, 10);
        wifi_text.set_hexpand(true);
        let wifi_title = ui::card_title(t("modeWifi"));
        wifi_title.set_halign(gtk::Align::Start);
        wifi_text.append(&wifi_title);
        let prompt = ui::dim_label(t("qrPrompt"));
        prompt.set_halign(gtk::Align::Start);
        prompt.set_wrap(true);
        prompt.set_xalign(0.0);
        wifi_text.append(&prompt);
        let url_value = gtk::Label::new(Some(""));
        url_value.add_css_class("orbi-mono");
        url_value.add_css_class("orbi-value");
        url_value.set_halign(gtk::Align::Start);
        url_value.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
        url_value.set_selectable(true);
        url_value.set_xalign(0.0);
        wifi_text.append(&url_value);

        let url_buttons = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let copy_button = gtk::Button::with_label(t("copy"));
        copy_button.add_css_class("orbi-btn");
        copy_button.add_css_class("tonal");
        copy_button.set_hexpand(true);
        let open_button = gtk::Button::with_label(t("open"));
        open_button.add_css_class("orbi-btn");
        open_button.add_css_class("tonal");
        open_button.set_hexpand(true);
        url_buttons.append(&copy_button);
        url_buttons.append(&open_button);
        wifi_text.append(&url_buttons);
        wifi_layout.append(&wifi_text);
        wifi.append(&wifi_layout);

        let usb = ui::card();
        let usb_row = gtk::Box::new(gtk::Orientation::Horizontal, 14);
        usb_row.append(&ui::icon_chip("media-removable-symbolic", "amber"));
        let usb_text = gtk::Box::new(gtk::Orientation::Vertical, 4);
        let usb_title_row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        let usb_title = gtk::Label::new(Some(t("usbTitle")));
        usb_title.add_css_class("orbi-hero-title");
        usb_title.set_halign(gtk::Align::Start);
        usb_title_row.append(&usb_title);
        let usb_pill = ui::pill(&t("usbWaiting").to_uppercase(), "warn");
        usb_title_row.append(&usb_pill);
        usb_text.append(&usb_title_row);
        let usb_desc = ui::dim_label(&format!(
            "{}\n{}\n{}",
            t("usbDesc"),
            t("usbHint1"),
            t("usbHint2")
        ));
        usb_desc.set_halign(gtk::Align::Start);
        usb_desc.set_wrap(true);
        usb_desc.set_xalign(0.0);
        usb_text.append(&usb_desc);
        let usb_status = gtk::Label::new(Some(t("usbWaiting")));
        usb_status.set_halign(gtk::Align::Start);
        usb_status.set_xalign(0.0);
        usb_status.add_css_class("orbi-value");
        usb_text.append(&usb_status);
        usb_row.append(&usb_text);
        usb.append(&usb_row);

        let usb_open_button = ui::primary_button(t("btnOpenUsb"));
        usb_open_button.set_valign(gtk::Align::Center);
        usb_open_button.set_margin_start(16);
        usb_row.append(&usb_open_button);

        let guide = ui::card();
        guide.append(&ui::card_title(t("tabConnect")));
        let guide_row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        guide_row.set_homogeneous(true);
        for (step, key) in [("1", "guide1"), ("2", "guide2"), ("3", "guide3")] {
            let tile = gtk::Box::new(gtk::Orientation::Vertical, 10);
            tile.add_css_class("orbi-tile");
            let num = gtk::Label::new(Some(step));
            num.add_css_class("orbi-num");
            num.set_halign(gtk::Align::Start);
            tile.append(&num);
            let text = gtk::Label::new(Some(i18n::tr(lang, key)));
            text.set_halign(gtk::Align::Start);
            text.set_xalign(0.0);
            text.set_wrap(true);
            tile.append(&text);
            guide_row.append(&tile);
        }
        guide.append(&guide_row);

        let toggle_button = ui::primary_button(t("startDisplay"));
        toggle_button.set_margin_top(2);

        content.append(&hero);
        content.append(&wifi);
        content.append(&usb);
        content.append(&guide);
        content.append(&toggle_button);

        let copy_ctx = ctx.clone();
        let copy_label = t("copy").to_string();
        let copied_label = t("copied").to_string();
        copy_button.connect_clicked(move |button| {
            let url = copy_ctx.state.borrow().url.clone();
            if let Some(window) = copy_ctx.window.upgrade() {
                window.clipboard().set_text(&url);
            }
            let _ = copy_ctx.tx.try_send(Msg::Toast(
                i18n::tr(copy_ctx.state.borrow().lang, "toastCopied").to_string(),
            ));
            button.set_label(&copied_label);
            let restore = button.clone();
            let restore_label = copy_label.clone();
            glib::timeout_add_local_once(std::time::Duration::from_millis(1500), move || {
                restore.set_label(&restore_label);
            });
        });

        let open_ctx = ctx.clone();
        open_button.connect_clicked(move |_button| {
            let url = open_ctx.state.borrow().url.clone();
            let tx = open_ctx.tx.clone();
            open_ctx.rt.spawn(async move {
                if let Err(e) = DaemonClient::open_browser(&url) {
                    let _ = tx.send(Msg::Toast(format!("Error: {e}"))).await;
                }
            });
        });

        let usb_ctx = ctx.clone();
        usb_open_button.connect_clicked(move |_button| {
            let url = crate::app::usb_url(&usb_ctx.state.borrow().status);
            let tx = usb_ctx.tx.clone();
            usb_ctx.rt.spawn(async move {
                if let Err(e) = DaemonClient::open_browser(&url) {
                    let _ = tx.send(Msg::Toast(format!("Error: {e}"))).await;
                }
            });
        });

        let toggle_ctx = ctx.clone();
        toggle_button.connect_clicked(move |_button| {
            let running = toggle_ctx.state.borrow().status.running;
            let tx = toggle_ctx.tx.clone();
            let rt = toggle_ctx.rt.clone();
            rt.spawn(async move {
                if running {
                    let _ = tx
                        .send(Msg::StopDone(DaemonClient::stop_service().await))
                        .await;
                } else {
                    let _ = tx
                        .send(Msg::StartDone(DaemonClient::start_service().await))
                        .await;
                }
            });
        });

        Self {
            root,
            hero_title,
            hero_subtitle,
            hero_pill,
            resolution_value,
            encoder_value,
            clients_value,
            url_value,
            qr,
            usb_pill,
            usb_status,
            toggle_button,
            lang,
        }
    }

    pub fn update(&self, state: &AppState) {
        let t = |key: &'static str| i18n::tr(self.lang, key);
        let status = &state.status;

        let (headline, subtitle, pill_text, pill_variant) = if !status.running {
            (
                t("statusOfflineHeadline"),
                t("statusOfflineSubtitle"),
                t("stopped"),
                "off",
            )
        } else if status.active_clients > 0 {
            (
                t("statusStreamingHeadline"),
                t("statusStreamingSubtitle"),
                t("streaming"),
                "live",
            )
        } else {
            (
                t("statusReadyHeadline"),
                t("statusReadySubtitle"),
                t("ready"),
                "ok",
            )
        };
        self.hero_title.set_label(headline);
        self.hero_subtitle.set_label(subtitle);
        ui::set_pill(&self.hero_pill, &pill_text.to_uppercase(), pill_variant);

        self.resolution_value.set_label(&format!(
            "{} x {} @ {}Hz",
            status.display_width, status.display_height, status.display_fps
        ));

        let encoder = if status.encoder.is_empty() {
            "-"
        } else {
            status.encoder.as_str()
        };
        self.encoder_value.set_label(encoder);

        let clients = match status.active_clients {
            0 => t("noDevices").to_string(),
            1 => t("deviceConnected").to_string(),
            n => i18n::format(
                self.lang,
                "devicesConnected",
                u32::try_from(n).unwrap_or(u32::MAX),
            ),
        };
        self.clients_value.set_label(&clients);

        self.url_value.set_label(&state.url);
        self.qr.set_url(&state.url);

        let dev_list = status.usb_connected_devices.join(", ");
        let (usb_pill_text, usb_variant, usb_value) = if status.usb_aoa_ready {
            if dev_list.is_empty() {
                (t("usbReady").to_string(), "ok", t("usbReady").to_string())
            } else {
                (
                    t("usbReady").to_string(),
                    "ok",
                    format!("{dev_list} ({})", t("connected")),
                )
            }
        } else if status.usb_devices > 0 || !dev_list.is_empty() {
            (
                t("connected").to_string(),
                "ok",
                if dev_list.is_empty() {
                    t("usbReady").to_string()
                } else {
                    dev_list.clone()
                },
            )
        } else {
            (
                t("usbWaiting").to_string(),
                "warn",
                t("usbWaiting").to_string(),
            )
        };
        ui::set_pill(&self.usb_pill, &usb_pill_text.to_uppercase(), usb_variant);
        self.usb_status.set_label(&usb_value);

        if status.running {
            self.toggle_button.set_label(t("stopDisplay"));
            self.toggle_button.remove_css_class("primary");
            self.toggle_button.add_css_class("danger");
        } else {
            self.toggle_button.set_label(t("startDisplay"));
            self.toggle_button.remove_css_class("danger");
            self.toggle_button.add_css_class("primary");
        }
    }
}
