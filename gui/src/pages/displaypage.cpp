// Orbiscreen - Qt Display page (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#include "pages/displaypage.h"

#include "i18n.h"
#include "theme.h"
#include "uikit.h"

#include <QCheckBox>
#include <QComboBox>
#include <QFormLayout>
#include <QGroupBox>
#include <QHBoxLayout>
#include <QLabel>
#include <QPushButton>
#include <QScrollArea>
#include <QSize>
#include <QStyle>
#include <QVBoxLayout>

namespace {

QString t(const char *key)
{
    return I18n::instance().t(QString::fromLatin1(key));
}

struct Resolution {
    int width;
    int height;
    const char *label;
};

const Resolution kResolutions[] = {
    {1920, 1080, "1080p"}, {2560, 1536, "2.5K"}, {1920, 1152, "1152p"},
    {2560, 1600, "1600p"}, {2880, 1800, "1800p"}, {1280, 768, "768p"},
    {1280, 800, "800p"},
};

const int kFramerates[] = {30, 60, 90, 120};

}

DisplayPage::DisplayPage(QWidget *parent)
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

    m_geometryGroup = UiKit::group(t("resGroup"));
    auto *geometryForm = UiKit::form();
    m_resolutionLabel = UiKit::label(t("resolution"), true);
    m_fpsLabel = UiKit::label(t("framerate"), true);
    m_resolution = new QComboBox(m_geometryGroup);
    for (const Resolution &resolution : kResolutions) {
        m_resolution->addItem(QString::fromLatin1(resolution.label),
                              QVariant::fromValue(QSize(resolution.width, resolution.height)));
    }
    m_fps = new QComboBox(m_geometryGroup);
    for (int fps : kFramerates) {
        m_fps->addItem(QStringLiteral("%1 Hz").arg(fps), fps);
    }
    m_apply = UiKit::button(t("resolutionApply"), true);
    geometryForm->addRow(m_resolutionLabel, m_resolution);
    geometryForm->addRow(m_fpsLabel, m_fps);
    geometryForm->addRow(QString(), m_apply);
    m_geometryGroup->setLayout(geometryForm);

    m_encoderGroup = UiKit::group(t("encoder"));
    auto *encoderForm = UiKit::form();
    m_encoderLabel = UiKit::label(t("encoder"), true);
    m_encoder = UiKit::valueLabel();
    encoderForm->addRow(m_encoderLabel, m_encoder);
    m_encoderGroup->setLayout(encoderForm);

    m_inputGroup = UiKit::group(t("inputGroup"));
    auto *inputLayout = new QVBoxLayout(m_inputGroup);
    inputLayout->setContentsMargins(12, 10, 12, 10);
    inputLayout->setSpacing(8);
    auto *touchRow = new QHBoxLayout;
    touchRow->addWidget(UiKit::iconLabel(
        {QStringLiteral("input-touchscreen"), QStringLiteral("input-touchscreen-symbolic")},
        QStyle::SP_ComputerIcon));
    auto *touchText = new QVBoxLayout;
    m_touchTitle = UiKit::label(t("touchScreen"));
    m_touchDesc = UiKit::label(t("touchDesc"), true, true);
    touchText->addWidget(m_touchTitle);
    touchText->addWidget(m_touchDesc);
    touchRow->addLayout(touchText, 1);
    inputLayout->addLayout(touchRow);
    auto *stylusRow = new QHBoxLayout;
    stylusRow->addWidget(UiKit::iconLabel(
        {QStringLiteral("input-tablet"), QStringLiteral("input-tablet-symbolic")},
        QStyle::SP_ComputerIcon));
    auto *stylusText = new QVBoxLayout;
    m_stylusTitle = UiKit::label(t("stylus"));
    m_stylusDesc = UiKit::label(t("stylusDesc"), true, true);
    stylusText->addWidget(m_stylusTitle);
    stylusText->addWidget(m_stylusDesc);
    stylusRow->addLayout(stylusText, 1);
    inputLayout->addLayout(stylusRow);

    m_systemGroup = UiKit::group(t("systemGroup"));
    auto *systemForm = UiKit::form();
    m_autostart = new QCheckBox(t("autostart"), m_systemGroup);
    m_autostartDesc = UiKit::label(t("autostartDesc"), true, true);
    systemForm->addRow(m_autostart);
    systemForm->addRow(m_autostartDesc);
    m_systemGroup->setLayout(systemForm);

    layout->addWidget(m_geometryGroup);
    layout->addWidget(m_encoderGroup);
    layout->addWidget(m_inputGroup);
    layout->addWidget(m_systemGroup);
    layout->addStretch(1);
    scroll->setWidget(content);
    outer->addWidget(scroll);

    connect(m_apply, &QPushButton::clicked, this, [this]() {
        const QSize size = m_resolution->currentData().toSize();
        const int fps = m_fps->currentData().toInt();
        emit resolutionApplyRequested(size.width(), size.height(), fps);
    });
    connect(m_autostart, &QCheckBox::toggled, this, [this](bool enabled) {
        if (!m_updating) {
            emit autostartChanged(enabled);
        }
    });

    retranslate();
}

void DisplayPage::update(const DaemonStatus &status, bool autostart)
{
    m_updating = true;
    const QSize current(status.report.displayWidth, status.report.displayHeight);
    int resolutionIndex = m_resolution->findData(QVariant::fromValue(current));
    if (resolutionIndex < 0) {
        resolutionIndex = 0;
    }
    m_resolution->setCurrentIndex(resolutionIndex);
    const int fpsIndex = m_fps->findData(status.report.displayFps);
    if (fpsIndex >= 0) {
        m_fps->setCurrentIndex(fpsIndex);
    }
    m_encoder->setText(status.report.encoder.isEmpty() ? QStringLiteral("-")
                                                       : status.report.encoder);
    m_autostart->setChecked(autostart);
    m_updating = false;
}

void DisplayPage::setAutostart(bool enabled)
{
    m_updating = true;
    m_autostart->setChecked(enabled);
    m_updating = false;
}

void DisplayPage::retranslate()
{
    m_geometryGroup->setTitle(t("resGroup"));
    m_encoderGroup->setTitle(t("encoder"));
    m_inputGroup->setTitle(t("inputGroup"));
    m_systemGroup->setTitle(t("systemGroup"));
    m_resolutionLabel->setText(t("resolution"));
    m_fpsLabel->setText(t("framerate"));
    m_encoderLabel->setText(t("encoder"));
    m_touchTitle->setText(t("touchScreen"));
    m_touchDesc->setText(t("touchDesc"));
    m_stylusTitle->setText(t("stylus"));
    m_stylusDesc->setText(t("stylusDesc"));
    m_autostart->setText(t("autostart"));
    m_autostartDesc->setText(t("autostartDesc"));
    m_apply->setText(t("resolutionApply"));
}
