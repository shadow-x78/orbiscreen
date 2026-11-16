// Orbiscreen - Qt Connect page (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#include "pages/connectpage.h"

#include "i18n.h"
#include "qrview.h"
#include "theme.h"
#include "uikit.h"

#include <QClipboard>
#include <QFont>
#include <QFrame>
#include <QGuiApplication>
#include <QHBoxLayout>
#include <QLabel>
#include <QLineEdit>
#include <QPushButton>
#include <QScrollArea>
#include <QTimer>
#include <QToolButton>
#include <QVBoxLayout>

namespace {

QString t(const char *key)
{
    return I18n::instance().t(QString::fromLatin1(key));
}

QLabel *stepBadge(const QString &number, QWidget *parent)
{
    auto *badge = new QLabel(number, parent);
    badge->setObjectName(QStringLiteral("StepBadge"));
    badge->setFixedSize(26, 26);
    badge->setAlignment(Qt::AlignCenter);
    return badge;
}

}

ConnectPage::ConnectPage(QWidget *parent)
    : QWidget(parent)
{
    auto *outer = new QVBoxLayout(this);
    outer->setContentsMargins(20, 12, 20, 16);
    outer->setSpacing(0);

    auto *scroll = new QScrollArea(this);
    scroll->setWidgetResizable(true);
    scroll->setFrameShape(QFrame::NoFrame);
    scroll->setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
    auto *content = new QWidget(scroll);
    auto *layout = new QVBoxLayout(content);
    layout->setContentsMargins(0, 0, 0, 0);
    layout->setSpacing(14);

    m_statusCard = UiKit::card(Theme::Glyph::Monitor, t("tabConnect"));
    m_headline = UiKit::headline(QString(), true);
    m_subtitle = UiKit::label(QString(), QStringLiteral("Muted"), true);
    m_statusCard->body()->addWidget(m_headline);
    m_statusCard->body()->addWidget(m_subtitle);
    auto *stats = new QHBoxLayout;
    stats->setSpacing(10);
    m_resolution = UiKit::statTile(Theme::Glyph::Monitor, t("resolution"));
    m_encoder = UiKit::statTile(Theme::Glyph::Chip, t("encoder"));
    m_devices = UiKit::statTile(Theme::Glyph::Phone, t("devices"));
    stats->addWidget(m_resolution.frame, 1);
    stats->addWidget(m_encoder.frame, 1);
    stats->addWidget(m_devices.frame, 1);
    m_statusCard->body()->addSpacing(4);
    m_statusCard->body()->addLayout(stats);

    m_wifiCard = UiKit::card(Theme::Glyph::Wifi, t("modeWifi"));
    auto *wifiRow = new QHBoxLayout;
    wifiRow->setSpacing(20);
    m_qr = new QrView(m_wifiCard);
    m_qr->setFixedSize(210, 210);
    wifiRow->addWidget(m_qr, 0, Qt::AlignTop);
    auto *wifiText = new QVBoxLayout;
    wifiText->setSpacing(10);
    m_wifiPrompt = UiKit::label(t("qrPrompt"), QStringLiteral("Muted"), true);
    m_url = new QLineEdit(m_wifiCard);
    m_url->setReadOnly(true);
    m_url->setCursorPosition(0);
    m_url->setClearButtonEnabled(false);
    m_copy = UiKit::iconButton(Theme::Glyph::Copy, t("copy"), 18);
    auto *urlRow = new QHBoxLayout;
    urlRow->setSpacing(8);
    urlRow->addWidget(m_url, 1);
    urlRow->addWidget(m_copy);
    m_open = UiKit::button(t("open"), Theme::Glyph::ExternalLink, QStringLiteral("primary"));
    auto *wifiActions = new QHBoxLayout;
    wifiActions->addWidget(m_open);
    wifiActions->addStretch(1);
    wifiText->addWidget(m_wifiPrompt);
    wifiText->addLayout(urlRow);
    wifiText->addLayout(wifiActions);
    wifiText->addStretch(1);
    wifiRow->addLayout(wifiText, 1);
    m_wifiCard->body()->addLayout(wifiRow);

    m_usbCard = UiKit::card(Theme::Glyph::Usb, t("usbTitle"));
    auto *usbRow = new QHBoxLayout;
    usbRow->setSpacing(16);
    auto *usbText = new QVBoxLayout;
    usbText->setSpacing(6);
    m_usbState = UiKit::label(QString(), QStringLiteral("Value"));
    m_usbDesc = UiKit::label(t("usbDesc"), QStringLiteral("Muted"), true);
    m_usbDevices = UiKit::label(QString(), QStringLiteral("Muted"), true);
    usbText->addWidget(m_usbState);
    usbText->addWidget(m_usbDesc);
    usbText->addWidget(m_usbDevices);
    usbRow->addLayout(usbText, 1);
    m_openUsb = UiKit::button(t("btnOpenUsb"), Theme::Glyph::Usb);
    usbRow->addWidget(m_openUsb, 0, Qt::AlignVCenter);
    m_usbCard->body()->addLayout(usbRow);

    m_guideCard = UiKit::card(Theme::Glyph::Info, t("gettingStarted"));
    const char *guideKeys[3] = {"guide1", "guide2", "guide3"};
    for (int index = 0; index < 3; ++index) {
        auto *row = new QHBoxLayout;
        row->setSpacing(12);
        row->addWidget(stepBadge(QString::number(index + 1), m_guideCard), 0,
                       Qt::AlignTop);
        m_guide[index] = UiKit::label(t(guideKeys[index]), QString(), true);
        row->addWidget(m_guide[index], 1);
        m_guideCard->body()->addLayout(row);
    }

    layout->addWidget(m_statusCard);
    layout->addWidget(m_wifiCard);
    layout->addWidget(m_usbCard);
    layout->addWidget(m_guideCard);
    layout->addStretch(1);
    scroll->setWidget(content);
    outer->addWidget(scroll);

    connect(m_copy, &QToolButton::clicked, this, [this]() {
        QGuiApplication::clipboard()->setText(m_url->text());
        m_copy->setIcon(Theme::glyph(Theme::Glyph::CheckMark, Theme::colors().success));
        m_copy->setToolTip(t("copied"));
        QTimer::singleShot(1500, this, [this]() {
            m_copy->setIcon(Theme::glyph(Theme::Glyph::Copy));
            m_copy->setToolTip(t("copy"));
        });
    });
    connect(m_open, &QPushButton::clicked, this,
            [this]() { emit openRequested(m_url->text()); });
    connect(m_openUsb, &QPushButton::clicked, this,
            [this]() { emit openUsbRequested(); });

    retranslate();
}

void ConnectPage::update(const DaemonStatus &status, const QImage &qr, const QString &url)
{
    const StatusReport &report = status.report;

    QString headline;
    QString subtitle;
    if (!report.running) {
        headline = t("statusOfflineHeadline");
        subtitle = t("statusOfflineSubtitle");
    } else if (report.activeClients > 0) {
        headline = t("statusStreamingHeadline");
        subtitle = t("statusStreamingSubtitle");
    } else {
        headline = t("statusReadyHeadline");
        subtitle = t("statusReadySubtitle");
    }
    m_headline->setText(headline);
    m_subtitle->setText(subtitle);

    m_resolution.value->setText(QStringLiteral("%1 × %2 @ %3 Hz")
                                    .arg(report.displayWidth)
                                    .arg(report.displayHeight)
                                    .arg(report.displayFps));
    m_encoder.value->setText(report.encoder.isEmpty() ? QStringLiteral("-") : report.encoder);
    m_devices.value->setText(report.activeClients == 0
                                 ? t("noDevices")
                                 : report.activeClients == 1
                                       ? t("deviceConnected")
                                       : I18n::instance().format(
                                             QStringLiteral("devicesConnected"),
                                             report.activeClients));

    m_url->setText(url);
    m_url->setCursorPosition(0);
    setQr(qr);

    const QStringList devices = report.usbConnectedDevices;
    const QString deviceText = devices.join(QStringLiteral(", "));
    if (report.usbAoaReady) {
        m_usbState->setText(t("usbReady"));
        m_usbDevices->setText(deviceText.isEmpty()
                                  ? QString()
                                  : QStringLiteral("%1 (%2)").arg(deviceText, t("connected")));
    } else if (report.usbDevices > 0 || !deviceText.isEmpty()) {
        m_usbState->setText(t("connected"));
        m_usbDevices->setText(deviceText.isEmpty() ? t("usbReady") : deviceText);
    } else {
        m_usbState->setText(t("usbWaiting"));
        m_usbDevices->setText(QString());
    }
}

void ConnectPage::setQr(const QImage &qr)
{
    if (qr.isNull()) {
        m_qr->clear();
        return;
    }
    m_qr->setMatrix(qr);
}

void ConnectPage::retranslate()
{
    m_statusCard->setTitle(t("tabConnect"));
    m_wifiCard->setTitle(t("modeWifi"));
    m_usbCard->setTitle(t("usbTitle"));
    m_guideCard->setTitle(t("gettingStarted"));
    m_resolution.caption->setText(t("resolution"));
    m_encoder.caption->setText(t("encoder"));
    m_devices.caption->setText(t("devices"));
    m_wifiPrompt->setText(t("qrPrompt"));
    m_usbDesc->setText(t("usbDesc"));
    const char *guideKeys[3] = {"guide1", "guide2", "guide3"};
    for (int index = 0; index < 3; ++index) {
        m_guide[index]->setText(t(guideKeys[index]));
    }
    m_copy->setToolTip(t("copy"));
    m_open->setText(t("open"));
    m_openUsb->setText(t("btnOpenUsb"));
}
