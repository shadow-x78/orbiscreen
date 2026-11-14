// Orbiscreen - UI Pages (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

pub mod connect;
pub mod display;
pub mod doctor;

use adw::prelude::*;

use crate::app::AppState;

pub struct Pages {
    pub connect: connect::ConnectPage,
    pub display: display::DisplayPage,
    pub doctor: doctor::DoctorPage,
}

impl Pages {
    pub fn new(ctx: &crate::app::PageCtx) -> Self {
        Self {
            connect: connect::ConnectPage::new(ctx),
            display: display::DisplayPage::new(ctx),
            doctor: doctor::DoctorPage::new(ctx),
        }
    }

    pub fn update(&self, state: &AppState) {
        self.connect.update(state);
        self.display.update(state);
        self.doctor.update(state);
    }

    pub fn set_autostart(&self, enabled: bool) {
        self.display.set_autostart(enabled);
    }

    pub fn doctor_scan_finished(&self) {
        self.doctor.scan_finished();
    }

    pub fn doctor_fix_finished(&self) {
        self.doctor.fix_finished();
    }
}

pub fn shell() -> (gtk::Widget, gtk::Box) {
    let scrolled = gtk::ScrolledWindow::new();
    scrolled.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scrolled.set_vexpand(true);
    let content = gtk::Box::new(gtk::Orientation::Vertical, 14);
    content.set_margin_top(10);
    content.set_margin_bottom(20);
    content.set_margin_start(14);
    content.set_margin_end(14);
    scrolled.set_child(Some(&content));
    (scrolled.upcast(), content)
}

pub fn card() -> gtk::Box {
    let card = gtk::Box::new(gtk::Orientation::Vertical, 8);
    card.add_css_class("orbi-card");
    card
}

pub fn card_title(label: &str) -> gtk::Label {
    let title = gtk::Label::new(Some(&label.to_uppercase()));
    title.add_css_class("orbi-card-title");
    title.set_halign(gtk::Align::Start);
    title
}

pub fn dim_label(label: &str) -> gtk::Label {
    let text = gtk::Label::new(Some(label));
    text.add_css_class("orbi-dim");
    text
}

pub fn pill(label: &str, variant: &str) -> gtk::Label {
    let pill = gtk::Label::new(Some(label));
    pill.add_css_class("orbi-pill");
    pill.add_css_class(variant);
    pill.set_valign(gtk::Align::Center);
    pill
}

pub fn set_pill(pill: &gtk::Label, label: &str, variant: &str) {
    pill.set_label(label);
    for class in ["ok", "live", "off", "warn"] {
        pill.remove_css_class(class);
    }
    pill.add_css_class(variant);
}

pub fn icon_chip(icon: &str, variant: &str) -> gtk::Widget {
    let bin = adw::Bin::new();
    bin.add_css_class("orbi-chip-icon");
    if !variant.is_empty() {
        bin.add_css_class(variant);
    }
    let image = gtk::Image::from_icon_name(icon);
    image.set_pixel_size(24);
    image.set_halign(gtk::Align::Center);
    image.set_valign(gtk::Align::Center);
    bin.set_child(Some(&image));
    bin.set_valign(gtk::Align::Center);
    bin.upcast()
}

pub fn spec_row(title: &str) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    row.add_css_class("orbi-row");
    let name = dim_label(title);
    name.set_halign(gtk::Align::Start);
    name.set_valign(gtk::Align::Center);
    row.append(&name);
    let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    row.append(&spacer);
    row
}

pub fn status_icon(variant: &str) -> gtk::Image {
    let icon = gtk::Image::from_icon_name(status_icon_name(variant));
    icon.add_css_class("orbi-status");
    icon.add_css_class(variant);
    icon.set_pixel_size(14);
    icon.set_valign(gtk::Align::Center);
    icon
}

pub fn status_icon_name(variant: &str) -> &'static str {
    match variant {
        "ok" => "emblem-ok-symbolic",
        "warn" => "dialog-warning-symbolic",
        _ => "dialog-error-symbolic",
    }
}

pub fn set_status_icon(icon: &gtk::Image, variant: &str) {
    icon.set_icon_name(Some(status_icon_name(variant)));
    for class in ["ok", "warn", "bad"] {
        icon.remove_css_class(class);
    }
    icon.add_css_class(variant);
}

pub fn value_label(text: &str) -> gtk::Label {
    let value = gtk::Label::new(Some(text));
    value.add_css_class("orbi-value");
    value.set_halign(gtk::Align::End);
    value.set_ellipsize(gtk::pango::EllipsizeMode::End);
    value
}

pub fn status_row(title: &str) -> (gtk::Box, gtk::Label) {
    let (row, value, _icon) = status_row_with_icon(title);
    (row, value)
}

pub fn status_row_with_icon(title: &str) -> (gtk::Box, gtk::Label, gtk::Image) {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    row.add_css_class("orbi-row");
    let name = gtk::Label::new(Some(title));
    name.add_css_class("orbi-value");
    name.set_halign(gtk::Align::Start);
    row.append(&name);
    let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    row.append(&spacer);
    let value = gtk::Label::new(Some("-"));
    value.add_css_class("orbi-dim");
    value.set_halign(gtk::Align::End);
    value.set_ellipsize(gtk::pango::EllipsizeMode::End);
    value.set_max_width_chars(36);
    let icon = status_icon("ok");
    row.append(&value);
    row.append(&icon);
    (row, value, icon)
}

pub fn primary_button(label: &str) -> gtk::Button {
    let button = gtk::Button::with_label(label);
    button.add_css_class("orbi-btn");
    button.add_css_class("primary");
    button
}

pub fn chip_button(label: &str) -> gtk::ToggleButton {
    let button = gtk::ToggleButton::with_label(label);
    button.add_css_class("orbi-chip");
    button
}
