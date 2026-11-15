// Orbiscreen - Qt Connect page (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#include "pages/connectpage.h"

#include "i18n.h"
#include "theme.h"
#include "uikit.h"

#include <QClipboard>
#include <QFont>
#include <QFormLayout>
#include <QFrame>
#include <QGridLayout>
#include <QGuiApplication>
#include <QHBoxLayout>
#include <QLabel>
#include <QLineEdit>
#include <QPixmap>
#include <QPushButton>
#include <QScrollArea>
#include <QStyle>
#include <QTimer>
#include <QVBoxLayout>

namespace {

QString t(const char *key)
{
    return I18n::instance().t(QString::fromLatin1(key));
}

}

ConnectPage::ConnectPage(QWidget *parent)
    : QWidget(parent)
{
    auto *outer = new QVBoxLayout(this);
    outer->setContentsMargins(12, 12, 12, 12);
    outer->setSpacing(10);

    auto *scroll = new QScrollArea(this);
    scroll->setWidgetResizable(true);
    scroll->setFrameShape(QFrame::NoFrame);
    auto *content = new QWidget(scroll);
    auto *layout = new QVBoxLayout(content);
    layout->setContentsMargins(0, 0, 0, 0);
    layout->setSpacing(12);

    m_statusGroup = UiKit::group(t("tabConnect"));
    auto *statusLayout = new QVBoxLayout(m_statusGroup);
    statusLayout->setContentsMargins(12, 10, 12, 10);
    statusLayout->setSpacing(6);
    m_headline = UiKit::label(QString(), false, true);
    QFont headlineFont = m_headline->font();
    headlineFont.setBold(true);
    headlineFont.setPointSizeF(headlineFont.pointSizeF() * 1.25);
    m_headline->setFont(headlineFont);
    m_subtitle = UiKit::label(QString(), true, true);
    m_state = UiKit::label(QString(), false);
    QFont stateFont = m_state->font();
    stateFont.setBold(true);
    m_state->setFont(stateFont);
    statusLayout->addWidget(m_headline);
    statusLayout->addWidget(m_subtitle);
    statusLayout->addWidget(m_state);

    auto *statusForm = UiKit::form();
    m_resolutionLabel = UiKit::label(t("resolution"), true);
    m_encoderLabel = UiKit::label(t("encoder"), true);
    m_devicesLabel = UiKit::label(t("devices"), true);
    m_resolution = UiKit::valueLabel();
    m_encoder = UiKit::valueLabel();
    m_clients = UiKit::valueLabel();
    statusForm->addRow(m_resolutionLabel, m_resolution);
    statusForm->addRow(m_encoderLabel, m_encoder);
    statusForm->addRow(m_devicesLabel, m_clients);
    statusLayout->addLayout(statusForm);

    m_wifiGroup = UiKit::group(t("modeWifi"));
    auto *wifiLayout = new QHBoxLayout(m_wifiGroup);
    wifiLayout->setContentsMargins(12, 10, 12, 10);
    wifiLayout->setSpacing(16);
    m_qr = new QLabel(m_wifiGroup);
    m_qr->setFixedSize(190, 190);
    m_qr->setAlignment(Qt::AlignCenter);
    m_qr->setFrameShape(QFrame::StyledPanel);
    wifiLayout->addWidget(m_qr);
    auto *wifiText = new QVBoxLayout;
    wifiText->setSpacing(8);
    m_wifiPrompt = UiKit::label(t("qrPrompt"), true, true);
    m_url = new QLineEdit(m_wifiGroup);
    m_url->setReadOnly(true);
    m_url->setCursorPosition(0);
    auto *wifiButtons = new QHBoxLayout;
    m_copy = UiKit::button(t("copy"));
    m_open = UiKit::button(t("open"));
    wifiButtons->addWidget(m_copy);
    wifiButtons->addWidget(m_open);
    wifiText->addWidget(m_wifiPrompt);
    wifiText->addWidget(m_url);
    wifiText->addLayout(wifiButtons);
    wifiText->addStretch(1);
    wifiLayout->addLayout(wifiText, 1);

    m_usbGroup = UiKit::group(t("usbTitle"));
    auto *usbLayout = new QHBoxLayout(m_usbGroup);
    usbLayout->setContentsMargins(12, 10, 12, 10);
    usbLayout->setSpacing(16);
    auto *usbText = new QVBoxLayout;
    usbText->setSpacing(6);
    m_usbState = UiKit::label(QString(), false);
    QFont usbFont = m_usbState->font();
    usbFont.setBold(true);
    m_usbState->setFont(usbFont);
    m_usbDesc = UiKit::label(t("usbDesc"), true, true);
    m_usbDevices = UiKit::label(QString(), false, true);
    usbText->addWidget(m_usbState);
    usbText->addWidget(m_usbDesc);
    usbText->addWidget(m_usbDevices);
    usbText->addStretch(1);
    m_openUsb = UiKit::button(t("btnOpenUsb"), true);
    usbLayout->addLayout(usbText, 1);
    usbLayout->addWidget(m_openUsb, 0, Qt::AlignVCenter);

    m_guideGroup = UiKit::group(t("gettingStarted"));
    auto *guideLayout = new QGridLayout(m_guideGroup);
    guideLayout->setContentsMargins(12, 10, 12, 10);
    guideLayout->setHorizontalSpacing(18);
    guideLayout->setVerticalSpacing(8);
    const char *guideKeys[3] = {"guide1", "guide2", "guide3"};
    for (int index = 0; index < 3; ++index) {
        auto *number = new QLabel(QString::number(index + 1), m_guideGroup);
        QFont numberFont = number->font();
        numberFont.setBold(true);
        number->setFont(numberFont);
        number->setAlignment(Qt::AlignTop | Qt::AlignHCenter);
        m_guide[index] = UiKit::label(t(guideKeys[index]), false, true);
        guideLayout->addWidget(number, 0, index * 2);
        guideLayout->addWidget(m_guide[index], 0, index * 2 + 1);
    }

    layout->addWidget(m_statusGroup);
    layout->addWidget(m_wifiGroup);
    layout->addWidget(m_usbGroup);
    layout->addWidget(m_guideGroup);
    layout->addStretch(1);
    scroll->setWidget(content);

    m_toggle = UiKit::button(t("startDisplay"), true);
    outer->addWidget(scroll, 1);
    outer->addWidget(m_toggle);

    connect(m_copy, &QPushButton::clicked, this, [this]() {
        QGuiApplication::clipboard()->setText(m_url->text());
        m_copy->setText(t("copied"));
        QTimer::singleShot(1500, this, [this]() { m_copy->setText(t("copy")); });
    });
    connect(m_open, &QPushButton::clicked, this,
            [this]() { emit openRequested(m_url->text()); });
    connect(m_openUsb, &QPushButton::clicked, this,
            [this]() { emit openUsbRequested(); });
    connect(m_toggle, &QPushButton::clicked, this,
            [this]() { emit startStopRequested(m_running); });

    retranslate();
}

void ConnectPage::update(const DaemonStatus &status, const QImage &qr, const QString &url)
{
    const StatusReport &report = status.report;
    m_running = report.running;

    QString headline;
    QString subtitle;
    QString state;
    if (!report.running) {
        headline = t("statusOfflineHeadline");
        subtitle = t("statusOfflineSubtitle");
        state = t("stopped");
    } else if (report.activeClients > 0) {
        headline = t("statusStreamingHeadline");
        subtitle = t("statusStreamingSubtitle");
        state = t("streaming");
    } else {
        headline = t("statusReadyHeadline");
        subtitle = t("statusReadySubtitle");
        state = t("ready");
    }
    m_headline->setText(headline);
    m_subtitle->setText(subtitle);
    m_state->setText(state);
    m_resolution->setText(QStringLiteral("%1 × %2 @ %3 Hz")
                              .arg(report.displayWidth)
                              .arg(report.displayHeight)
                              .arg(report.displayFps));
    m_encoder->setText(report.encoder.isEmpty() ? QStringLiteral("-") : report.encoder);
    m_clients->setText(report.activeClients == 0
                           ? t("noDevices")
                           : report.activeClients == 1
                                 ? t("deviceConnected")
                                 : I18n::instance().format(QStringLiteral("devicesConnected"),
                                                           report.activeClients));
    m_url->setText(url);
    m_url->setCursorPosition(0);
    setQr(qr);

    const QStringList devices = report.usbConnectedDevices;
    const QString deviceText = devices.join(QStringLiteral(", "));
    if (report.usbAoaReady) {
        m_usbState->setText(t("usbReady"));
        m_usbDevices->setText(deviceText.isEmpty()
                                  ? t("usbReady")
                                  : QStringLiteral("%1 (%2)").arg(deviceText, t("connected")));
    } else if (report.usbDevices > 0 || !deviceText.isEmpty()) {
        m_usbState->setText(t("connected"));
        m_usbDevices->setText(deviceText.isEmpty() ? t("usbReady") : deviceText);
    } else {
        m_usbState->setText(t("usbWaiting"));
        m_usbDevices->setText(QString());
    }

    m_toggle->setText(report.running ? t("stopDisplay") : t("startDisplay"));
    m_toggle->setIcon(Theme::icon(
        {report.running ? QStringLiteral("media-playback-stop")
                        : QStringLiteral("media-playback-start")},
        report.running ? QStyle::SP_MediaStop : QStyle::SP_MediaPlay));
}

void ConnectPage::setQr(const QImage &qr)
{
    if (qr.isNull()) {
        m_qr->clear();
        return;
    }
    m_qr->setPixmap(QPixmap::fromImage(qr).scaled(180, 180, Qt::KeepAspectRatio,
                                                   Qt::SmoothTransformation));
}

void ConnectPage::retranslate()
{
    m_statusGroup->setTitle(t("tabConnect"));
    m_wifiGroup->setTitle(t("modeWifi"));
    m_usbGroup->setTitle(t("usbTitle"));
    m_guideGroup->setTitle(t("gettingStarted"));
    m_resolutionLabel->setText(t("resolution"));
    m_encoderLabel->setText(t("encoder"));
    m_devicesLabel->setText(t("devices"));
    m_wifiPrompt->setText(t("qrPrompt"));
    m_usbDesc->setText(t("usbDesc"));
    const char *guideKeys[3] = {"guide1", "guide2", "guide3"};
    for (int index = 0; index < 3; ++index) {
        m_guide[index]->setText(t(guideKeys[index]));
    }
    m_copy->setText(t("copy"));
    m_open->setText(t("open"));
    m_openUsb->setText(t("btnOpenUsb"));
    m_toggle->setText(m_running ? t("stopDisplay") : t("startDisplay"));
}
