// Orbiscreen - Qt Doctor page (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#include "pages/doctorpage.h"

#include "i18n.h"
#include "uikit.h"

#include <QFont>
#include <QFormLayout>
#include <QGroupBox>
#include <QHBoxLayout>
#include <QLabel>
#include <QPushButton>
#include <QScrollArea>
#include <QStyle>
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
    outer->setContentsMargins(0, 0, 0, 0);
    auto *scroll = new QScrollArea(this);
    scroll->setWidgetResizable(true);
    auto *content = new QWidget(scroll);
    auto *layout = new QVBoxLayout(content);
    layout->setContentsMargins(12, 12, 12, 12);
    layout->setSpacing(12);

    m_headerGroup = UiKit::group(t("doctorTitle"));
    auto *headerLayout = new QVBoxLayout(m_headerGroup);
    headerLayout->setContentsMargins(12, 10, 12, 10);
    headerLayout->setSpacing(8);
    m_title = UiKit::label(t("doctorTitle"));
    QFont titleFont = m_title->font();
    titleFont.setBold(true);
    titleFont.setPointSizeF(titleFont.pointSizeF() * 1.15);
    m_title->setFont(titleFont);
    m_description = UiKit::label(t("doctorDesc"), true, true);
    auto *actions = new QHBoxLayout;
    m_scan = UiKit::button(t("scan"), true);
    m_fix = UiKit::button(t("autoFix"));
    actions->addWidget(m_scan);
    actions->addWidget(m_fix);
    headerLayout->addWidget(m_title);
    headerLayout->addWidget(m_description);
    headerLayout->addLayout(actions);

    m_resultsGroup = UiKit::group(t("doctorTitle"));
    auto *form = UiKit::form();
    m_empty = UiKit::label(t("doctorNoReport"), true, true);
    form->addRow(m_empty);
    const char *rowKeys[6] = {"docCompositor", "docBackend", "docUinput",
                              "docEncoder", "portsTitle", "docUsb"};
    QLabel **values[6] = {&m_compositor, &m_backend, &m_uinput,
                          &m_encoder, &m_ports, &m_usb};
    QLabel **icons[6] = {nullptr, &m_backendIcon, &m_uinputIcon,
                         &m_encoderIcon, nullptr, &m_usbIcon};
    for (int index = 0; index < 6; ++index) {
        m_rowTitles[index] = UiKit::label(t(rowKeys[index]), true);
        auto *row = new QWidget(m_resultsGroup);
        auto *rowLayout = new QHBoxLayout(row);
        rowLayout->setContentsMargins(0, 0, 0, 0);
        rowLayout->setSpacing(8);
        *values[index] = UiKit::valueLabel();
        rowLayout->addWidget(*values[index], 1);
        if (icons[index]) {
            *icons[index] = UiKit::statusIcon(QStringLiteral("warn"));
            rowLayout->addWidget(*icons[index]);
        }
        form->addRow(m_rowTitles[index], row);
    }
    m_resultsGroup->setLayout(form);

    m_notesGroup = UiKit::group(t("docTipTitle"));
    auto *notesLayout = new QVBoxLayout(m_notesGroup);
    notesLayout->setContentsMargins(12, 10, 12, 10);
    m_notesTitle = UiKit::label(t("docTipTitle"));
    m_notesText = UiKit::label(t("docTipDesc"), true, true);
    notesLayout->addWidget(m_notesTitle);
    notesLayout->addWidget(m_notesText);

    layout->addWidget(m_headerGroup);
    layout->addWidget(m_resultsGroup);
    layout->addWidget(m_notesGroup);
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
    m_headerGroup->setTitle(t("doctorTitle"));
    m_resultsGroup->setTitle(t("doctorTitle"));
    m_notesGroup->setTitle(t("docTipTitle"));
    m_title->setText(t("doctorTitle"));
    m_description->setText(t("doctorDesc"));
    m_empty->setText(t("doctorNoReport"));
    m_notesTitle->setText(t("docTipTitle"));
    m_notesText->setText(t("docTipDesc"));
    const char *rowKeys[6] = {"docCompositor", "docBackend", "docUinput",
                              "docEncoder", "portsTitle", "docUsb"};
    for (int index = 0; index < 6; ++index) {
        m_rowTitles[index]->setText(t(rowKeys[index]));
    }
    m_scan->setText(m_scanning ? t("checking") : t("scan"));
    m_fix->setText(m_fixing ? t("fixing") : t("autoFix"));
}
