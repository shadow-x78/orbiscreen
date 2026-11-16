// Orbiscreen - Qt GUI widget helpers (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#include "uikit.h"

#include <QAbstractButton>
#include <QComboBox>
#include <QEvent>
#include <QFont>
#include <QGuiApplication>
#include <QHBoxLayout>
#include <QPainter>
#include <QPixmap>
#include <QPushButton>
#include <QScreen>
#include <QSizePolicy>
#include <QStyle>
#include <QToolButton>
#include <QVBoxLayout>

namespace {

QPixmap dotPixmap(int size, const QColor &color)
{
    const QScreen *screen = QGuiApplication::primaryScreen();
    const qreal ratio = screen ? screen->devicePixelRatio() : 1.0;
    QPixmap pixmap(QSize(qRound(size * ratio), qRound(size * ratio)));
    pixmap.setDevicePixelRatio(ratio);
    pixmap.fill(Qt::transparent);
    QPainter painter(&pixmap);
    painter.setRenderHint(QPainter::Antialiasing, true);
    painter.setPen(Qt::NoPen);
    painter.setBrush(color);
    painter.drawEllipse(QRectF(0, 0, size, size).adjusted(0.5, 0.5, -0.5, -0.5));
    return pixmap;
}

Theme::Glyph statusGlyph(const QString &kind)
{
    if (kind == QStringLiteral("ok")) {
        return Theme::Glyph::Check;
    }
    if (kind == QStringLiteral("warn")) {
        return Theme::Glyph::Warning;
    }
    return Theme::Glyph::Error;
}

class ValueLabel : public QLabel {
public:
    explicit ValueLabel(bool leadingEdge, QWidget *parent = nullptr)
        : QLabel(parent)
        , m_leading(leadingEdge)
    {
        setTextFormat(Qt::PlainText);
        setObjectName(QStringLiteral("Value"));
        setText(QStringLiteral("-"));
        setTextInteractionFlags(Qt::TextSelectableByMouse);
        syncAlignment();
    }

protected:
    void changeEvent(QEvent *event) override
    {
        QLabel::changeEvent(event);
        if (event->type() == QEvent::LayoutDirectionChange) {
            syncAlignment();
        }
    }

private:
    void syncAlignment()
    {
        const bool rtl = layoutDirection() == Qt::RightToLeft;
        const Qt::Alignment horizontal = m_leading
            ? (rtl ? Qt::AlignRight : Qt::AlignLeft)
            : (rtl ? Qt::AlignLeft : Qt::AlignRight);
        setAlignment(horizontal | Qt::AlignVCenter);
    }

    bool m_leading;
};

class ChevronComboBox : public QComboBox {
public:
    explicit ChevronComboBox(QWidget *parent = nullptr) : QComboBox(parent) {}

protected:
    void paintEvent(QPaintEvent *event) override
    {
        QComboBox::paintEvent(event);
        QPainter painter(this);
        painter.setRenderHint(QPainter::Antialiasing, true);
        QColor color = isEnabled() ? Theme::colors().muted : Theme::colors().borderStrong;
        const int side = 16;
        const QPixmap pixmap = Theme::glyphPixmap(Theme::Glyph::Chevron, side, color);
        const QSize size = pixmap.deviceIndependentSize().toSize();
        const QRect target(width() - size.width() - 12,
                           (height() - size.height()) / 2, size.width(), size.height());
        painter.drawPixmap(target, pixmap);
    }
};

class Switch : public QAbstractButton {
public:
    explicit Switch(QWidget *parent = nullptr) : QAbstractButton(parent)
    {
        setCheckable(true);
        setCursor(Qt::PointingHandCursor);
        setFocusPolicy(Qt::StrongFocus);
        connect(this, &QAbstractButton::toggled, this, [this]() { update(); });
    }

    QSize sizeHint() const override { return QSize(42, 24); }
    QSize minimumSizeHint() const override { return QSize(42, 24); }

protected:
    void paintEvent(QPaintEvent *) override
    {
        const Theme::Colors &c = Theme::colors();
        QPainter painter(this);
        painter.setRenderHint(QPainter::Antialiasing, true);
        QRectF track(0.5, 0.5, width() - 1.0, height() - 1.0);
        const qreal radius = track.height() / 2.0;
        painter.setPen(QPen(isChecked() ? c.accent : c.borderStrong, 1.0));
        painter.setBrush(isChecked() ? c.accent : c.surfaceAlt);
        painter.drawRoundedRect(track, radius, radius);
        const qreal diameter = height() - 8.0;
        const QRectF knob(isChecked() ? width() - diameter - 4.0 : 4.0, 4.0, diameter, diameter);
        painter.setPen(Qt::NoPen);
        painter.setBrush(isChecked() ? QColor(QStringLiteral("#FFFFFF")) : c.muted);
        painter.drawEllipse(knob);
        if (hasFocus()) {
            painter.setPen(QPen(c.accent, 1.0));
            painter.setBrush(Qt::NoBrush);
            painter.drawRoundedRect(track.adjusted(-1.5, -1.5, 1.5, 1.5), radius + 1.5,
                                    radius + 1.5);
        }
    }

    void nextCheckState() override
    {
        QAbstractButton::nextCheckState();
        update();
    }
};

}

namespace UiKit {

Card::Card(Theme::Glyph glyph, const QString &title, QWidget *parent)
    : QFrame(parent)
{
    setObjectName(QStringLiteral("Card"));
    auto *outer = new QVBoxLayout(this);
    outer->setContentsMargins(18, 16, 18, 18);
    outer->setSpacing(12);
    auto *head = new QHBoxLayout;
    head->setSpacing(10);
    m_icon = new QLabel(this);
    m_icon->setFixedSize(18, 18);
    m_icon->setPixmap(Theme::glyphPixmap(glyph, 18, Theme::colors().accent));
    m_title = new QLabel(title, this);
    m_title->setObjectName(QStringLiteral("CardTitle"));
    head->addWidget(m_icon);
    head->addWidget(m_title, 1);
    outer->addLayout(head);
    m_body = new QVBoxLayout;
    m_body->setSpacing(10);
    outer->addLayout(m_body, 1);
}

void Card::setTitle(const QString &title)
{
    m_title->setText(title);
}

StatusPill::StatusPill(QWidget *parent)
    : QFrame(parent)
{
    setObjectName(QStringLiteral("StatusPill"));
    auto *layout = new QHBoxLayout(this);
    layout->setContentsMargins(12, 6, 14, 6);
    layout->setSpacing(8);
    m_dot = new QLabel(this);
    m_dot->setFixedSize(10, 10);
    m_text = new QLabel(this);
    layout->addWidget(m_dot);
    layout->addWidget(m_text);
    setStatus(QStringLiteral("stopped"), QString());
}

void StatusPill::setStatus(const QString &state, const QString &text)
{
    m_text->setText(text);
    if (state == m_state) {
        return;
    }
    m_state = state;
    setProperty("state", state);
    QColor color = Theme::colors().danger;
    if (state == QStringLiteral("ready")) {
        color = Theme::colors().success;
    } else if (state == QStringLiteral("streaming")) {
        color = Theme::colors().accent;
    }
    m_dot->setPixmap(dotPixmap(10, color));
    repolish(this);
    repolish(m_text);
}

Card *card(Theme::Glyph glyph, const QString &title, QWidget *parent)
{
    return new Card(glyph, title, parent);
}

QLabel *label(const QString &text, const QString &role, bool wrap)
{
    auto *widget = new QLabel(text);
    widget->setTextFormat(Qt::PlainText);
    widget->setWordWrap(wrap);
    widget->setAlignment(Qt::AlignLeft | Qt::AlignVCenter);
    if (!role.isEmpty()) {
        widget->setObjectName(role);
    }
    return widget;
}

QLabel *headline(const QString &text, bool wrap)
{
    auto *widget = new QLabel(text);
    widget->setTextFormat(Qt::PlainText);
    widget->setWordWrap(wrap);
    QFont font = widget->font();
    if (font.pointSizeF() > 0.0) {
        font.setPointSizeF(font.pointSizeF() * 1.32);
    }
    font.setWeight(QFont::DemiBold);
    widget->setFont(font);
    return widget;
}

QLabel *valueLabel(bool leadingEdge)
{
    return new ValueLabel(leadingEdge);
}

QLabel *iconLabel(Theme::Glyph glyph, int size, const QColor &color)
{
    auto *widget = new QLabel;
    widget->setFixedSize(size, size);
    widget->setPixmap(Theme::glyphPixmap(glyph, size,
                                         color.isValid() ? color : Theme::colors().muted));
    return widget;
}

QLabel *badgeLabel(Theme::Glyph glyph, int size)
{
    auto *widget = new QLabel;
    widget->setObjectName(QStringLiteral("Badge"));
    widget->setFixedSize(size, size);
    widget->setAlignment(Qt::AlignCenter);
    widget->setPixmap(Theme::glyphPixmap(glyph, qRound(size * 0.5), Theme::colors().accent));
    return widget;
}

QLabel *statusIcon(const QString &kind, int size)
{
    auto *widget = new QLabel;
    widget->setFixedSize(size, size);
    widget->setAlignment(Qt::AlignCenter);
    widget->setPixmap(Theme::glyphPixmap(statusGlyph(kind), size, Theme::statusColor(kind)));
    return widget;
}

void setStatusIcon(QLabel *label, const QString &kind, int size)
{
    if (!label) {
        return;
    }
    label->setPixmap(Theme::glyphPixmap(statusGlyph(kind), size, Theme::statusColor(kind)));
    label->setToolTip(QString());
}

QPushButton *button(const QString &text, const QString &variant)
{
    auto *widget = new QPushButton(text);
    widget->setCursor(Qt::PointingHandCursor);
    if (!variant.isEmpty()) {
        widget->setProperty("variant", variant);
    }
    return widget;
}

QPushButton *button(const QString &text, Theme::Glyph glyph, const QString &variant)
{
    auto *widget = button(text, variant);
    widget->setIcon(Theme::glyph(glyph, variant == QStringLiteral("primary")
                                           ? Theme::colors().accentText
                                           : Theme::colors().text));
    widget->setIconSize(QSize(18, 18));
    return widget;
}

QToolButton *iconButton(Theme::Glyph glyph, const QString &tooltip, int size)
{
    auto *widget = new QToolButton;
    widget->setObjectName(QStringLiteral("IconButton"));
    widget->setIcon(Theme::glyph(glyph));
    widget->setIconSize(QSize(size, size));
    widget->setFixedSize(size + 20, size + 20);
    widget->setAutoRaise(true);
    widget->setCursor(Qt::PointingHandCursor);
    widget->setToolTip(tooltip);
    return widget;
}

QComboBox *comboBox()
{
    return new ChevronComboBox;
}

QAbstractButton *toggle(bool checked)
{
    auto *widget = new Switch;
    widget->setChecked(checked);
    return widget;
}

StatTile statTile(Theme::Glyph glyph, const QString &caption)
{
    StatTile tile;
    tile.frame = new QFrame;
    tile.frame->setObjectName(QStringLiteral("StatTile"));
    auto *layout = new QVBoxLayout(tile.frame);
    layout->setContentsMargins(14, 12, 14, 12);
    layout->setSpacing(6);
    auto *top = new QHBoxLayout;
    top->setSpacing(8);
    tile.caption = label(caption, QStringLiteral("Muted"), true);
    top->addWidget(tile.caption, 1);
    top->addWidget(iconLabel(glyph, 16), 0, Qt::AlignTop);
    tile.value = valueLabel(true);
    layout->addLayout(top);
    layout->addWidget(tile.value);
    return tile;
}

QFrame *divider()
{
    auto *widget = new QFrame;
    widget->setObjectName(QStringLiteral("Divider"));
    widget->setFixedHeight(1);
    widget->setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Fixed);
    return widget;
}

void repolish(QWidget *widget)
{
    if (!widget) {
        return;
    }
    widget->style()->unpolish(widget);
    widget->style()->polish(widget);
    widget->update();
}

void setVariant(QWidget *widget, const QString &variant)
{
    if (!widget) {
        return;
    }
    widget->setProperty("variant", variant);
    repolish(widget);
}

}
