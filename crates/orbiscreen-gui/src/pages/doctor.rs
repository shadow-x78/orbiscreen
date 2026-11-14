// Orbiscreen - Doctor Page (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

use adw::prelude::*;

use crate::app::{AppState, Msg, PageCtx};
use crate::daemon_client::DaemonClient;
use crate::i18n::{self, Lang};
use crate::pages::{self as ui};

pub struct DoctorPage {
    pub root: gtk::Widget,
    scan_button: gtk::Button,
    fix_button: gtk::Button,
    compositor_value: gtk::Label,
    backend_value: gtk::Label,
    backend_dot: gtk::Image,
    uinput_value: gtk::Label,
    uinput_dot: gtk::Image,
    encoder_value: gtk::Label,
    encoder_dot: gtk::Image,
    ports_value: gtk::Label,
    usb_value: gtk::Label,
    usb_dot: gtk::Image,
    lang: Lang,
}

impl DoctorPage {
    pub fn new(ctx: &PageCtx) -> Self {
        let lang = ctx.state.borrow().lang;
        let t = |key: &'static str| i18n::tr(lang, key);

        let (root, content) = ui::shell();

        let header = ui::card();
        let title = gtk::Label::new(Some(t("doctorTitle")));
        title.add_css_class("orbi-hero-title");
        title.set_halign(gtk::Align::Start);
        header.append(&title);
        let desc = ui::dim_label(t("doctorDesc"));
        desc.set_halign(gtk::Align::Start);
        desc.set_wrap(true);
        desc.set_xalign(0.0);
        header.append(&desc);

        let actions_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        actions_row.set_homogeneous(true);
        let scan_button = ui::primary_button(t("scan"));
        scan_button.set_hexpand(true);
        actions_row.append(&scan_button);
        let fix_button = gtk::Button::with_label(t("autoFix"));
        fix_button.add_css_class("orbi-btn");
        fix_button.add_css_class("tonal");
        fix_button.set_hexpand(true);
        actions_row.append(&fix_button);
        header.append(&actions_row);

        let results = ui::card();
        let (compositor_row, compositor_value) = ui::status_row(t("docCompositor"));
        let (backend_row, backend_value, backend_dot) = ui::status_row_with_icon(t("docBackend"));
        let (uinput_row, uinput_value, uinput_dot) = ui::status_row_with_icon(t("docUinput"));
        let (encoder_row, encoder_value, encoder_dot) = ui::status_row_with_icon(t("docEncoder"));
        let (ports_row, ports_value) = ui::status_row(t("portsTitle"));
        let (usb_row, usb_value, usb_dot) = ui::status_row_with_icon(t("docUsb"));
        results.append(&compositor_row);
        results.append(&backend_row);
        results.append(&uinput_row);
        results.append(&encoder_row);
        results.append(&ports_row);
        results.append(&usb_row);

        let notes = ui::card();
        notes.append(&ui::card_title(t("docTipTitle")));
        let notes_text = ui::dim_label(t("docTipDesc"));
        notes_text.set_wrap(true);
        notes_text.set_xalign(0.0);
        notes_text.set_halign(gtk::Align::Start);
        notes.append(&notes_text);

        content.append(&header);
        content.append(&results);
        content.append(&notes);

        let scan_ctx = ctx.clone();
        scan_button.connect_clicked(move |button| {
            button.set_label(i18n::tr(lang, "checking"));
            button.set_sensitive(false);
            let tx = scan_ctx.tx.clone();
            let rt = scan_ctx.rt.clone();
            rt.spawn(async move {
                let msg = match DaemonClient::run_doctor_check().await {
                    Ok(report) => Msg::Doctor(Ok(report)),
                    Err(e) => Msg::Doctor(Err(e)),
                };
                let _ = tx.send(msg).await;
            });
        });

        let fix_ctx = ctx.clone();
        fix_button.connect_clicked(move |button| {
            button.set_label(i18n::tr(lang, "fixing"));
            button.set_sensitive(false);
            let tx = fix_ctx.tx.clone();
            let rt = fix_ctx.rt.clone();
            rt.spawn(async move {
                let _ = tx
                    .send(Msg::DoctorFix(DaemonClient::run_doctor_fix().await))
                    .await;
            });
        });

        Self {
            root,
            scan_button,
            fix_button,
            compositor_value,
            backend_value,
            backend_dot,
            uinput_value,
            uinput_dot,
            encoder_value,
            encoder_dot,
            ports_value,
            usb_value,
            usb_dot,
            lang,
        }
    }

    pub fn scan_finished(&self) {
        self.scan_button.set_label(i18n::tr(self.lang, "scan"));
        self.scan_button.set_sensitive(true);
    }

    pub fn fix_finished(&self) {
        self.fix_button.set_label(i18n::tr(self.lang, "autoFix"));
        self.fix_button.set_sensitive(true);
    }

    pub fn update(&self, state: &AppState) {
        let t = |key: &'static str| i18n::tr(self.lang, key);

        let ports_text = format!(
            "{} / {} - {}",
            state.status.signaling_port,
            state.status.udp_port,
            t("docPortsDesc")
        );
        self.ports_value.set_label(&ports_text);
        self.ports_value.set_tooltip_text(Some(&ports_text));

        let Some(report) = state.doctor.as_ref() else {
            return;
        };

        let compositor_text = match report.current_desktop.as_deref() {
            Some(desktop) if !desktop.is_empty() => {
                format!("{} / {} ({desktop})", report.session, report.compositor)
            }
            _ => format!("{} / {}", report.session, report.compositor),
        };
        self.compositor_value.set_label(&compositor_text);
        self.compositor_value
            .set_tooltip_text(Some(&compositor_text));

        let mut backend = if report.capture_plan.is_empty() {
            report.display_backend.clone()
        } else {
            format!(
                "{} ({})",
                report.display_backend,
                report.capture_plan.join(", ")
            )
        };
        if let Some(ipc) = report.wlroots_virtual_output_ipc.as_deref() {
            backend.push_str(&format!(" (IPC: {ipc})"));
        }
        self.backend_value.set_label(&backend);
        self.backend_value.set_tooltip_text(Some(&backend));
        let backend_ok = report.display_backend_kind.is_healthy();
        ui::set_status_icon(&self.backend_dot, if backend_ok { "ok" } else { "bad" });

        let uinput_state = if report.uinput_writable {
            t("permVerified")
        } else {
            t("permDenied")
        };
        let uinput_text = if report.input_backend.is_empty() {
            uinput_state.to_string()
        } else {
            format!("{uinput_state} ({})", report.input_backend)
        };
        self.uinput_value.set_label(&uinput_text);
        self.uinput_value.set_tooltip_text(Some(&uinput_text));
        ui::set_status_icon(
            &self.uinput_dot,
            if report.uinput_writable { "ok" } else { "bad" },
        );

        let encoder = state.status.encoder.as_str();
        let encoder_ok = !encoder.is_empty()
            && !encoder.eq_ignore_ascii_case("unknown")
            && !encoder.eq_ignore_ascii_case("none");
        let encoder_text = if encoder_ok {
            format!("{} ({encoder})", t("available"))
        } else {
            t("notDetected").to_string()
        };
        self.encoder_value.set_label(&encoder_text);
        self.encoder_value.set_tooltip_text(Some(&encoder_text));
        ui::set_status_icon(&self.encoder_dot, if encoder_ok { "ok" } else { "warn" });

        let usb_names = report.usb.devices.join(", ");
        let usb_ok = report.usb.aoa_ready && !report.usb.permission_denied;
        let usb_text = if report.usb.permission_denied {
            t("usbPermissionDenied").to_string()
        } else if report.usb.aoa_ready {
            if usb_names.is_empty() {
                t("usbReady").to_string()
            } else {
                format!("{usb_names} ({})", t("usbReady"))
            }
        } else if !usb_names.is_empty() {
            format!("{usb_names} ({})", t("connected"))
        } else {
            t("usbNoDevices").to_string()
        };
        self.usb_value.set_label(&usb_text);
        self.usb_value.set_tooltip_text(Some(&usb_text));
        ui::set_status_icon(&self.usb_dot, if usb_ok { "ok" } else { "warn" });
    }
}

#[cfg(test)]
mod tests {
    use orbiscreen_core::report::DoctorReport;

    const REAL_REPORT: &str = r#"{"capture_plan":["kwin-virtual","portal"],"compositor":"KDE Plasma","cosmic_comp":false,"cosmic_randr":false,"current_desktop":"KDE","display_backend":"Native (KWin Virtual)","display_backend_kind":"native_kwin","hyprctl":false,"input_backend":"Wayland","portal_on_session_bus":true,"remote_desktop_saved_token":false,"screencast_saved_token":false,"session":"wayland","swaymsg":false,"uinput_writable":true,"usb":{"aoa_ready":false,"devices":[],"permission_denied":false},"wlroots_virtual_output_ipc":null}"#;

    #[test]
    fn parses_real_daemon_report() {
        let report: DoctorReport = serde_json::from_str(REAL_REPORT).expect("valid report");
        assert_eq!(report.session, "wayland");
        assert_eq!(report.compositor, "KDE Plasma");
        assert_eq!(report.current_desktop.as_deref(), Some("KDE"));
        assert_eq!(report.display_backend, "Native (KWin Virtual)");
        assert_eq!(
            report.capture_plan,
            vec!["kwin-virtual".to_string(), "portal".to_string()]
        );
        assert_eq!(report.input_backend, "Wayland");
        assert!(report.uinput_writable);
        assert!(report.usb.devices.is_empty());
        assert!(!report.usb.aoa_ready);
        assert!(!report.usb.permission_denied);
        assert!(report.wlroots_virtual_output_ipc.is_none());
    }

    #[test]
    fn parses_report_with_usb_devices() {
        let raw = r#"{"session":"x11","compositor":"X11","capture_plan":[],"display_backend":"XSHM","input_backend":"XTest","uinput_writable":false,"usb":{"aoa_ready":true,"devices":["Google Pixel 8 (Ready)"],"permission_denied":false}}"#;
        let report: DoctorReport = serde_json::from_str(raw).expect("valid report");
        assert_eq!(report.session, "x11");
        assert!(!report.uinput_writable);
        assert!(report.usb.aoa_ready);
        assert_eq!(
            report.usb.devices,
            vec!["Google Pixel 8 (Ready)".to_string()]
        );
        assert!(report.current_desktop.is_none());
    }

    #[test]
    fn rejects_malformed_report() {
        assert!(serde_json::from_str::<DoctorReport>("not json").is_err());
    }
}
