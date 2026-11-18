#include "pages/doctorpage.h"

#include "i18n.h"
#include "theme.h"
#include "uikit.h"

#include <QFontMetrics>
#include <QFrame>
#include <QGridLayout>
#include <QHBoxLayout>
#include <QLabel>
#include <QProgressBar>
#include <QPushButton>
#include <QVBoxLayout>

namespace {

QString t(const char *key)
{
    return I18n::instance().t(QString::fromLatin1(key));
}

int measureColumn(int characters)
{
    const QFontMetrics metrics(Theme::font(Theme::FontRole::Body));
    return metrics.horizontalAdvance(QStringLiteral("0")) * characters;
}

void setStatusGlyph(UiKit::GlyphWidget *icon, const QString &kind)
{
    if (!icon) {
        return;
    }
    icon->setGlyph(kind == QStringLiteral("ok") ? Theme::Glyph::Check
                                                : Theme::Glyph::Warning);
    icon->setColor(Theme::statusColor(kind));
}

}

DoctorPage::DoctorPage(QWidget *parent)
    : QWidget(parent)
{
    auto *outer = new QVBoxLayout(this);
    outer->setContentsMargins(0, 0, 0, 0);
    outer->setSpacing(0);

    auto *scroll = new UiKit::PageScroll(this);
    QVBoxLayout *layout = scroll->contentLayout();

    m_headerCard = UiKit::card(Theme::Glyph::Doctor, t("doctorTitle"));
    auto *headerRow = new QHBoxLayout;
    headerRow->setSpacing(Theme::Spacing::lg);
    m_description = UiKit::label(t("doctorDesc"), QStringLiteral("Muted"), true);
    m_description->setMaximumWidth(measureColumn(58));
    m_description->setMinimumWidth(measureColumn(34));
    headerRow->addWidget(m_description, 1, Qt::AlignLeading | Qt::AlignVCenter);
    m_scan = UiKit::button(t("scan"), Theme::Glyph::Refresh, QStringLiteral("primary"));
    m_fix = UiKit::button(t("autoFix"), Theme::Glyph::CheckMark);
    headerRow->addWidget(m_scan, 0, Qt::AlignVCenter);
    headerRow->addWidget(m_fix, 0, Qt::AlignVCenter);
    m_headerCard->body()->addLayout(headerRow);

    m_resultsCard = UiKit::card(Theme::Glyph::Dashboard, t("doctorResults"));
    m_progress = new QProgressBar(m_resultsCard);
    m_progress->setRange(0, 0);
    m_progress->setTextVisible(false);
    m_progress->setFixedHeight(4);
    m_progress->hide();
    m_resultsCard->body()->addWidget(m_progress);
    m_empty = UiKit::label(t("doctorNoReport"), QStringLiteral("Muted"), true);
    m_resultsCard->body()->addWidget(m_empty);

    const char *rowKeys[6] = {"docCompositor", "docBackend", "docUinput",
                              "docEncoder", "portsTitle", "docUsb"};
    QLabel **values[6] = {&m_compositor, &m_backend, &m_uinput,
                          &m_encoder, &m_ports, &m_usb};
    UiKit::GlyphWidget **icons[6] = {nullptr, &m_backendIcon, &m_uinputIcon,
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
        row->setMinimumHeight(40);
        auto *rowLayout = new QHBoxLayout(row);
        rowLayout->setContentsMargins(0, Theme::Spacing::xs, 0, Theme::Spacing::xs);
        rowLayout->setSpacing(Theme::Spacing::md);
        m_rowTitles[index] = UiKit::label(t(rowKeys[index]), QStringLiteral("Muted"));
        m_rowTitles[index]->setMinimumWidth(measureColumn(22));
        *values[index] = UiKit::label(QString(), QStringLiteral("Value"), true);
        (*values[index])->setAlignment(Qt::AlignTrailing | Qt::AlignVCenter);
        (*values[index])->setFont(Theme::tabular((*values[index])->font()));
        rowLayout->addWidget(m_rowTitles[index], 0, Qt::AlignVCenter);
        rowLayout->addWidget(*values[index], 1, Qt::AlignVCenter);
        if (icons[index]) {
            *icons[index] = new UiKit::GlyphWidget(Theme::Glyph::Warning, 18,
                                                   Theme::colors().warning, row);
            rowLayout->addWidget(*icons[index], 0, Qt::AlignVCenter);
        }
        m_resultsCard->body()->addWidget(row);
        m_rowWidgets[index] = row;
    }

    m_notesCard = UiKit::card(Theme::Glyph::Info, t("docTipTitle"));
    m_notesText = UiKit::label(t("docTipDesc"), QStringLiteral("Muted"), true);
    m_notesText->setMaximumWidth(measureColumn(56));
    m_notesCard->body()->addWidget(m_notesText);

    auto *columns = new UiKit::ResponsiveColumns(2, scroll);
    columns->addWidget(m_resultsCard, 2);
    columns->addWidget(m_notesCard, 1);

    layout->addWidget(m_headerCard);
    layout->addWidget(columns);
    layout->addStretch(1);
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
    setStatusGlyph(m_backendIcon, report.backendHealthy() ? QStringLiteral("ok")
                                                          : QStringLiteral("bad"));

    const QString permission =
        report.uinputWritable ? t("permVerified") : t("permDenied");
    m_uinput->setText(report.inputBackend.isEmpty()
                          ? permission
                          : QStringLiteral("%1 (%2)").arg(permission, report.inputBackend));
    setStatusGlyph(m_uinputIcon, report.uinputWritable ? QStringLiteral("ok")
                                                       : QStringLiteral("bad"));

    const QString encoder = status.report.encoder;
    const bool encoderOkay = !encoder.isEmpty()
        && encoder.compare(QStringLiteral("unknown"), Qt::CaseInsensitive) != 0
        && encoder.compare(QStringLiteral("none"), Qt::CaseInsensitive) != 0;
    m_encoder->setText(encoderOkay
                           ? QStringLiteral("%1 (%2)").arg(t("available"), encoder)
                           : t("notDetected"));
    setStatusGlyph(m_encoderIcon, encoderOkay ? QStringLiteral("ok")
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
    setStatusGlyph(m_usbIcon, usbOkay ? QStringLiteral("ok")
                                      : QStringLiteral("warn"));
}

void DoctorPage::setBusy(bool scanning, bool fixing)
{
    m_scanning = scanning;
    m_fixing = fixing;
    const bool busy = scanning || fixing;
    m_scan->setEnabled(!busy);
    m_fix->setEnabled(!busy);
    m_scan->setText(scanning ? t("checking") : t("scan"));
    m_fix->setText(fixing ? t("fixing") : t("autoFix"));
    m_progress->setVisible(busy);
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
