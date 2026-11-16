// Orbiscreen - Qt Doctor page (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#include "pages/doctorpage.h"

#include "i18n.h"
#include "theme.h"
#include "uikit.h"

#include <QFrame>
#include <QHBoxLayout>
#include <QLabel>
#include <QPushButton>
#include <QScrollArea>
#include <QVBoxLayout>

namespace {

QString t(const char *key)
{
    return I18n::instance().t(QString::fromLatin1(key));
}

}

DoctorPage::DoctorPage(QWidget *parent)
    : QWidget(parent)
{
    auto *outer = new QVBoxLayout(this);
    outer->setContentsMargins(20, 12, 20, 16);
    auto *scroll = new QScrollArea(this);
    scroll->setWidgetResizable(true);
    scroll->setFrameShape(QFrame::NoFrame);
    scroll->setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
    auto *content = new QWidget(scroll);
    auto *layout = new QVBoxLayout(content);
    layout->setContentsMargins(0, 0, 0, 0);
    layout->setSpacing(14);

    m_headerCard = UiKit::card(Theme::Glyph::Doctor, t("doctorTitle"));
    auto *headerRow = new QHBoxLayout;
    headerRow->setSpacing(16);
    m_description = UiKit::label(t("doctorDesc"), QStringLiteral("Muted"), true);
    headerRow->addWidget(m_description, 1);
    m_scan = UiKit::button(t("scan"), Theme::Glyph::Refresh, QStringLiteral("primary"));
    m_fix = UiKit::button(t("autoFix"), Theme::Glyph::CheckMark);
    headerRow->addWidget(m_scan);
    headerRow->addWidget(m_fix);
    m_headerCard->body()->addLayout(headerRow);

    m_resultsCard = UiKit::card(Theme::Glyph::Chip, t("doctorTitle"));
    m_empty = UiKit::label(t("doctorNoReport"), QStringLiteral("Muted"), true);
    m_resultsCard->body()->addWidget(m_empty);
    const char *rowKeys[6] = {"docCompositor", "docBackend", "docUinput",
                              "docEncoder", "portsTitle", "docUsb"};
    QLabel **values[6] = {&m_compositor, &m_backend, &m_uinput,
                          &m_encoder, &m_ports, &m_usb};
    QLabel **icons[6] = {nullptr, &m_backendIcon, &m_uinputIcon,
                         &m_encoderIcon, nullptr, &m_usbIcon};
    for (int index = 0; index < 6; ++index) {
        auto *divider = UiKit::divider();
        m_dividers[index] = divider;
        if (index > 0) {
            m_resultsCard->body()->addWidget(divider);
        } else {
            divider->hide();
        }
        auto *row = new QWidget(m_resultsCard);
        auto *rowLayout = new QHBoxLayout(row);
        rowLayout->setContentsMargins(0, 0, 0, 0);
        rowLayout->setSpacing(10);
        m_rowTitles[index] = UiKit::label(t(rowKeys[index]), QStringLiteral("Muted"));
        *values[index] = UiKit::valueLabel();
        rowLayout->addWidget(m_rowTitles[index], 0, Qt::AlignVCenter);
        rowLayout->addWidget(*values[index], 1);
        if (icons[index]) {
            *icons[index] = UiKit::statusIcon(QStringLiteral("warn"), 18);
            rowLayout->addWidget(*icons[index], 0, Qt::AlignVCenter);
        }
        m_resultsCard->body()->addWidget(row);
        m_rowWidgets[index] = row;
    }

    m_notesCard = UiKit::card(Theme::Glyph::Info, t("docTipTitle"));
    m_notesText = UiKit::label(t("docTipDesc"), QStringLiteral("Muted"), true);
    m_notesCard->body()->addWidget(m_notesText);

    layout->addWidget(m_headerCard);
    layout->addWidget(m_resultsCard);
    layout->addWidget(m_notesCard);
    layout->addStretch(1);
    scroll->setWidget(content);
    outer->addWidget(scroll);

    connect(m_scan, &QPushButton::clicked, this, &DoctorPage::scanRequested);
    connect(m_fix, &QPushButton::clicked, this, &DoctorPage::fixRequested);

    retranslate();
}

void DoctorPage::update(const DaemonStatus &status, const DoctorReport &report,
                        bool hasReport)
{
    m_ports->setText(QStringLiteral("%1 / %2 - %3")
                         .arg(status.report.signalingPort)
                         .arg(status.udpPort)
                         .arg(t("docPortsDesc")));
    m_empty->setVisible(!hasReport);
    for (QWidget *row : m_rowWidgets) {
        if (row) {
            row->setVisible(hasReport);
        }
    }
    for (int index = 1; index < 6; ++index) {
        if (m_dividers[index]) {
            m_dividers[index]->setVisible(hasReport);
        }
    }
    if (!hasReport) {
        return;
    }

    const QString compositor = report.currentDesktop.isEmpty()
        ? QStringLiteral("%1 / %2").arg(report.session, report.compositor)
        : QStringLiteral("%1 / %2 (%3)")
              .arg(report.session, report.compositor, report.currentDesktop);
    m_compositor->setText(compositor);

    QString backend = report.displayBackend;
    if (!report.capturePlan.isEmpty()) {
        backend += QStringLiteral(" (%1)").arg(report.capturePlan.join(QStringLiteral(", ")));
    }
    if (!report.wlrootsVirtualOutputIpc.isEmpty()) {
        backend += QStringLiteral(" (IPC: %1)").arg(report.wlrootsVirtualOutputIpc);
    }
    m_backend->setText(backend);
    UiKit::setStatusIcon(m_backendIcon, report.backendHealthy() ? QStringLiteral("ok")
                                                                : QStringLiteral("bad"));

    const QString permission =
        report.uinputWritable ? t("permVerified") : t("permDenied");
    m_uinput->setText(report.inputBackend.isEmpty()
                          ? permission
                          : QStringLiteral("%1 (%2)").arg(permission, report.inputBackend));
    UiKit::setStatusIcon(m_uinputIcon, report.uinputWritable ? QStringLiteral("ok")
                                                             : QStringLiteral("bad"));

    const QString encoder = status.report.encoder;
    const bool encoderOkay = !encoder.isEmpty()
        && encoder.compare(QStringLiteral("unknown"), Qt::CaseInsensitive) != 0
        && encoder.compare(QStringLiteral("none"), Qt::CaseInsensitive) != 0;
    m_encoder->setText(encoderOkay
                           ? QStringLiteral("%1 (%2)").arg(t("available"), encoder)
                           : t("notDetected"));
    UiKit::setStatusIcon(m_encoderIcon, encoderOkay ? QStringLiteral("ok")
                                                    : QStringLiteral("warn"));

    const QStringList names = report.usb.devices;
    const QString joined = names.join(QStringLiteral(", "));
    const bool usbOkay = report.usb.aoaReady && !report.usb.permissionDenied;
    if (report.usb.permissionDenied) {
        m_usb->setText(t("usbPermissionDenied"));
    } else if (report.usb.aoaReady) {
        m_usb->setText(joined.isEmpty()
                           ? t("usbReady")
                           : QStringLiteral("%1 (%2)").arg(joined, t("usbReady")));
    } else if (!names.isEmpty()) {
        m_usb->setText(QStringLiteral("%1 (%2)").arg(joined, t("connected")));
    } else {
        m_usb->setText(t("usbNoDevices"));
    }
    UiKit::setStatusIcon(m_usbIcon, usbOkay ? QStringLiteral("ok")
                                            : QStringLiteral("warn"));
}

void DoctorPage::setBusy(bool scanning, bool fixing)
{
    m_scanning = scanning;
    m_fixing = fixing;
    m_scan->setEnabled(!scanning && !fixing);
    m_fix->setEnabled(!scanning && !fixing);
    m_scan->setText(scanning ? t("checking") : t("scan"));
    m_fix->setText(fixing ? t("fixing") : t("autoFix"));
}

void DoctorPage::retranslate()
{
    m_headerCard->setTitle(t("doctorTitle"));
    m_resultsCard->setTitle(t("doctorResults"));
    m_notesCard->setTitle(t("docTipTitle"));
    m_description->setText(t("doctorDesc"));
    m_empty->setText(t("doctorNoReport"));
    m_notesText->setText(t("docTipDesc"));
    const char *rowKeys[6] = {"docCompositor", "docBackend", "docUinput",
                              "docEncoder", "portsTitle", "docUsb"};
    for (int index = 0; index < 6; ++index) {
        m_rowTitles[index]->setText(t(rowKeys[index]));
    }
    m_scan->setText(m_scanning ? t("checking") : t("scan"));
    m_fix->setText(m_fixing ? t("fixing") : t("autoFix"));
}
