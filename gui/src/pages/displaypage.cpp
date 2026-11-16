// Orbiscreen - Qt Display page (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#include "pages/displaypage.h"

#include "i18n.h"
#include "theme.h"
#include "uikit.h"

#include <QAbstractButton>
#include <QComboBox>
#include <QFont>
#include <QFrame>
#include <QHBoxLayout>
#include <QLabel>
#include <QPushButton>
#include <QScrollArea>
#include <QSize>
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

QVBoxLayout *field(const QString &caption, QWidget *control, QLabel **captionLabel)
{
    auto *box = new QVBoxLayout;
    box->setSpacing(6);
    auto *label = UiKit::label(caption, QStringLiteral("Muted"));
    box->addWidget(label);
    box->addWidget(control);
    if (captionLabel) {
        *captionLabel = label;
    }
    return box;
}

}

DisplayPage::DisplayPage(QWidget *parent)
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

    m_geometryCard = UiKit::card(Theme::Glyph::Monitor, t("resGroup"));
    auto *geometryRow = new QHBoxLayout;
    geometryRow->setSpacing(14);
    m_resolution = UiKit::comboBox();
    for (const Resolution &resolution : kResolutions) {
        m_resolution->addItem(QString::fromLatin1(resolution.label),
                              QVariant::fromValue(QSize(resolution.width,
                                                        resolution.height)));
    }
    m_fps = UiKit::comboBox();
    for (int fps : kFramerates) {
        m_fps->addItem(QStringLiteral("%1 Hz").arg(fps), fps);
    }
    geometryRow->addLayout(field(t("resolution"), m_resolution, &m_resolutionCaption), 1);
    geometryRow->addLayout(field(t("framerate"), m_fps, &m_fpsCaption), 1);
    m_geometryCard->body()->addLayout(geometryRow);
    m_apply = UiKit::button(t("resolutionApply"), Theme::Glyph::CheckMark,
                            QStringLiteral("primary"));
    auto *applyRow = new QHBoxLayout;
    applyRow->addStretch(1);
    applyRow->addWidget(m_apply);
    m_geometryCard->body()->addLayout(applyRow);

    m_encoderCard = UiKit::card(Theme::Glyph::Chip, t("encoder"));
    auto *encoderRow = new QHBoxLayout;
    encoderRow->setSpacing(14);
    encoderRow->addWidget(UiKit::badgeLabel(Theme::Glyph::Chip), 0, Qt::AlignTop);
    auto *encoderText = new QVBoxLayout;
    encoderText->setSpacing(4);
    m_encoder = UiKit::valueLabel(true);
    QFont encoderFont = m_encoder->font();
    if (encoderFont.pointSizeF() > 0.0) {
        encoderFont.setPointSizeF(encoderFont.pointSizeF() * 1.12);
    }
    encoderFont.setWeight(QFont::DemiBold);
    m_encoder->setFont(encoderFont);
    encoderText->addWidget(m_encoder);
    encoderRow->addLayout(encoderText, 1);
    m_encoderCard->body()->addLayout(encoderRow);

    m_inputCard = UiKit::card(Theme::Glyph::Phone, t("inputGroup"));
    const Theme::Glyph inputGlyphs[2] = {Theme::Glyph::Monitor, Theme::Glyph::Pen};
    for (int index = 0; index < 2; ++index) {
        auto *row = new QHBoxLayout;
        row->setSpacing(12);
        row->addWidget(UiKit::iconLabel(inputGlyphs[index], 20, Theme::colors().accent), 0,
                       Qt::AlignTop);
        auto *text = new QVBoxLayout;
        text->setSpacing(3);
        QLabel *title = UiKit::label(QString(), QStringLiteral("Value"));
        QLabel *desc = UiKit::label(QString(), QStringLiteral("Muted"), true);
        text->addWidget(title);
        text->addWidget(desc);
        row->addLayout(text, 1);
        if (index == 0) {
            m_touchTitle = title;
            m_touchDesc = desc;
        } else {
            m_stylusTitle = title;
            m_stylusDesc = desc;
        }
        m_inputCard->body()->addLayout(row);
        if (index == 0) {
            m_inputCard->body()->addWidget(UiKit::divider());
        }
    }

    m_systemCard = UiKit::card(Theme::Glyph::Dashboard, t("systemGroup"));
    auto *systemRow = new QHBoxLayout;
    systemRow->setSpacing(16);
    auto *systemText = new QVBoxLayout;
    systemText->setSpacing(4);
    m_autostartTitle = UiKit::label(t("autostart"), QStringLiteral("Value"));
    m_autostartDesc = UiKit::label(t("autostartDesc"), QStringLiteral("Muted"), true);
    systemText->addWidget(m_autostartTitle);
    systemText->addWidget(m_autostartDesc);
    systemRow->addLayout(systemText, 1);
    m_autostart = UiKit::toggle(false);
    systemRow->addWidget(m_autostart, 0, Qt::AlignVCenter);
    m_systemCard->body()->addLayout(systemRow);

    layout->addWidget(m_geometryCard);
    layout->addWidget(m_encoderCard);
    layout->addWidget(m_inputCard);
    layout->addWidget(m_systemCard);
    layout->addStretch(1);
    scroll->setWidget(content);
    outer->addWidget(scroll);

    connect(m_apply, &QPushButton::clicked, this, [this]() {
        const QSize size = m_resolution->currentData().toSize();
        const int fps = m_fps->currentData().toInt();
        m_pendingResolution = true;
        emit resolutionApplyRequested(size.width(), size.height(), fps);
    });
    connect(m_autostart, &QAbstractButton::toggled, this, [this](bool enabled) {
        if (!m_updating) {
            m_pendingAutostart = true;
            emit autostartChanged(enabled);
        }
    });

    retranslate();
}

void DisplayPage::update(const DaemonStatus &status, bool autostart)
{
    m_updating = true;
    if (!m_pendingResolution && !m_resolution->hasFocus() && !m_fps->hasFocus()) {
        const QSize current(status.report.displayWidth, status.report.displayHeight);
        int resolutionIndex = m_resolution->findData(QVariant::fromValue(current));
        if (resolutionIndex < 0) {
            m_resolution->addItem(QStringLiteral("%1 × %2")
                                      .arg(status.report.displayWidth)
                                      .arg(status.report.displayHeight),
                                  QVariant::fromValue(current));
            resolutionIndex = m_resolution->count() - 1;
        }
        m_resolution->setCurrentIndex(resolutionIndex);
        const int fpsIndex = m_fps->findData(status.report.displayFps);
        if (fpsIndex >= 0) {
            m_fps->setCurrentIndex(fpsIndex);
        }
    }
    m_encoder->setText(status.report.encoder.isEmpty() ? QStringLiteral("-")
                                                       : status.report.encoder);
    if (!m_pendingAutostart) {
        m_autostart->setChecked(autostart);
    }
    m_updating = false;
}

void DisplayPage::setAutostart(bool enabled)
{
    m_updating = true;
    m_autostart->setChecked(enabled);
    m_updating = false;
    m_pendingAutostart = false;
}

void DisplayPage::resolutionApplied()
{
    m_pendingResolution = false;
}

void DisplayPage::retranslate()
{
    m_geometryCard->setTitle(t("resGroup"));
    m_encoderCard->setTitle(t("encoder"));
    m_inputCard->setTitle(t("inputGroup"));
    m_systemCard->setTitle(t("systemGroup"));
    m_resolutionCaption->setText(t("resolution"));
    m_fpsCaption->setText(t("framerate"));
    m_touchTitle->setText(t("touchScreen"));
    m_touchDesc->setText(t("touchDesc"));
    m_stylusTitle->setText(t("stylus"));
    m_stylusDesc->setText(t("stylusDesc"));
    m_autostartTitle->setText(t("autostart"));
    m_autostartDesc->setText(t("autostartDesc"));
    m_apply->setText(t("resolutionApply"));
}
