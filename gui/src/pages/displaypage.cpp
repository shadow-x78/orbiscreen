// Orbiscreen - Qt Display page (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#include "pages/displaypage.h"

#include "i18n.h"
#include "theme.h"
#include "uikit.h"

#include <QAbstractButton>
#include <QComboBox>
#include <QFontMetrics>
#include <QFrame>
#include <QHBoxLayout>
#include <QLabel>
#include <QPushButton>
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

int measureColumn(int characters)
{
    const QFontMetrics metrics(Theme::font(Theme::FontRole::Body));
    return metrics.horizontalAdvance(QStringLiteral("0")) * characters;
}

QVBoxLayout *field(const QString &captionText, QWidget *control, QLabel **captionLabel)
{
    auto *box = new QVBoxLayout;
    box->setContentsMargins(0, 0, 0, 0);
    box->setSpacing(Theme::Spacing::xs + 2);
    auto *label = UiKit::caption(captionText);
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
    outer->setContentsMargins(0, 0, 0, 0);
    outer->setSpacing(0);

    auto *scroll = new UiKit::PageScroll(this);
    QVBoxLayout *layout = scroll->contentLayout();

    m_geometryCard = UiKit::card(Theme::Glyph::Monitor, t("resGroup"));
    auto *geometryRow = new QHBoxLayout;
    geometryRow->setSpacing(Theme::Metrics::cardGap);
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
    geometryRow->addLayout(field(t("resolution"), m_resolution, &m_resolutionCaption), 3);
    geometryRow->addLayout(field(t("framerate"), m_fps, &m_fpsCaption), 2);
    // Apply sits on the fields' baseline instead of in a second row that left a 60px void under
    // the combos. The wrapper pins it to the bottom so it lines up with the inputs.
    m_apply = UiKit::button(t("resolutionApply"), Theme::Glyph::CheckMark,
                            QStringLiteral("primary"));
    auto *applyCell = new QVBoxLayout;
    applyCell->setContentsMargins(0, 0, 0, 0);
    applyCell->addStretch(1);
    applyCell->addWidget(m_apply);
    geometryRow->addLayout(applyCell, 0);
    m_geometryCard->body()->addLayout(geometryRow);

    m_encoderCard = UiKit::card(Theme::Glyph::Chip, t("encoder"));
    auto *encoderRow = new QHBoxLayout;
    encoderRow->setSpacing(Theme::Spacing::md);
    m_encoder = UiKit::label(QStringLiteral("-"), QStringLiteral("Value"), false);
    m_encoder->setTextInteractionFlags(Qt::TextSelectableByMouse);
    encoderRow->addWidget(m_encoder, 1, Qt::AlignVCenter);
    m_encoderCard->body()->addLayout(encoderRow);

    m_inputCard = UiKit::card(Theme::Glyph::Touch, t("inputGroup"));
    m_inputCard->setBodySpacing(Theme::Spacing::lg);
    const Theme::Glyph inputGlyphs[2] = {Theme::Glyph::Touch, Theme::Glyph::Pen};
    for (int index = 0; index < 2; ++index) {
        auto *row = new QHBoxLayout;
        row->setSpacing(Theme::Spacing::md);
        auto *icon = new UiKit::GlyphWidget(inputGlyphs[index], 20, Theme::colors().muted,
                                           m_inputCard);
        row->addWidget(icon, 0, Qt::AlignTop);
        auto *text = new QVBoxLayout;
        text->setContentsMargins(0, 0, 0, 0);
        text->setSpacing(Theme::Spacing::xs - 2);
        QLabel *title = UiKit::label(QString(), QStringLiteral("Value"));
        QLabel *desc = UiKit::caption(QString(), true);
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
    }

    m_systemCard = UiKit::card(Theme::Glyph::Login, t("systemGroup"));
    auto *systemRow = new QHBoxLayout;
    systemRow->setSpacing(Theme::Spacing::md);
    auto *systemText = new QVBoxLayout;
    systemText->setContentsMargins(0, 0, 0, 0);
    systemText->setSpacing(Theme::Spacing::xs - 2);
    m_autostartTitle = UiKit::label(t("autostart"), QStringLiteral("Value"));
    m_autostartDesc = UiKit::caption(t("autostartDesc"), true);
    m_autostartDesc->setMaximumWidth(measureColumn(56));
    systemText->addWidget(m_autostartTitle);
    systemText->addWidget(m_autostartDesc);
    systemRow->addLayout(systemText, 1);
    m_autostart = UiKit::toggle(false);
    systemRow->addWidget(m_autostart, 0, Qt::AlignVCenter);
    m_systemCard->body()->addLayout(systemRow);

    auto *columns = new UiKit::ResponsiveColumns(2, scroll);
    auto *secondary = new QWidget(columns);
    secondary->setObjectName(QStringLiteral("PageSurface"));
    auto *secondaryLayout = new QVBoxLayout(secondary);
    secondaryLayout->setContentsMargins(0, 0, 0, 0);
    secondaryLayout->setSpacing(Theme::Metrics::cardGap);
    secondaryLayout->addWidget(m_encoderCard);
    secondaryLayout->addWidget(m_systemCard);
    secondaryLayout->addStretch(1);

    layout->addWidget(m_geometryCard);
    columns->addWidget(m_inputCard, 1);
    columns->addWidget(secondary, 1);
    layout->addWidget(columns);
    layout->addStretch(1);
    outer->addWidget(scroll);

    connect(m_apply, &QPushButton::clicked, this, &DisplayPage::applySelection);
    connect(m_resolution, &QComboBox::currentIndexChanged, this,
            &DisplayPage::updateApplyState);
    connect(m_fps, &QComboBox::currentIndexChanged, this, &DisplayPage::updateApplyState);
    connect(m_autostart, &QAbstractButton::toggled, this, [this](bool enabled) {
        if (!m_updating) {
            m_pendingAutostart = true;
            emit autostartChanged(enabled);
        }
    });

    retranslate();
    updateApplyState();
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
        m_appliedResolution = current;
        m_appliedFps = status.report.displayFps;
        m_encoder->setText(status.report.encoder.isEmpty() ? QStringLiteral("-")
                                                           : status.report.encoder);
        m_updating = false;
        updateApplyState();
    } else {
        m_encoder->setText(status.report.encoder.isEmpty() ? QStringLiteral("-")
                                                           : status.report.encoder);
        m_updating = false;
    }
    if (!m_pendingAutostart) {
        m_updating = true;
        m_autostart->setChecked(autostart);
        m_updating = false;
    }
}

void DisplayPage::setAutostart(bool enabled)
{
    m_updating = true;
    m_autostart->setChecked(enabled);
    m_updating = false;
    m_pendingAutostart = false;
}

void DisplayPage::applySelection()
{
    if (!m_apply->isEnabled()) {
        return;
    }
    const QSize size = m_resolution->currentData().toSize();
    const int fps = m_fps->currentData().toInt();
    m_pendingResolution = true;
    m_apply->setText(t("applying"));
    m_apply->setEnabled(false);
    m_resolution->setEnabled(false);
    m_fps->setEnabled(false);
    emit resolutionApplyRequested(size.width(), size.height(), fps);
}

void DisplayPage::updateApplyState()
{
    if (m_updating) {
        return;
    }
    const QSize size = m_resolution->currentData().toSize();
    const int fps = m_fps->currentData().toInt();
    const bool dirty = size != m_appliedResolution || fps != m_appliedFps;
    m_apply->setEnabled(dirty);
    m_apply->setText(t("resolutionApply"));
    m_resolution->setEnabled(true);
    m_fps->setEnabled(true);
}

void DisplayPage::resolutionApplied()
{
    m_pendingResolution = false;
    m_appliedResolution = m_resolution->currentData().toSize();
    m_appliedFps = m_fps->currentData().toInt();
    updateApplyState();
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
    m_apply->setText(m_pendingResolution ? t("applying") : t("resolutionApply"));
}
