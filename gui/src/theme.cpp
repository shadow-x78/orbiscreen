// Orbiscreen - Qt GUI theme helpers (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#include "theme.h"

#include <QApplication>
#include <QGuiApplication>
#include <QIconEngine>
#include <QPainter>
#include <QPainterPath>
#include <QPalette>
#include <QPolygonF>
#include <QScreen>
#include <QStyleFactory>
#include <QtMath>

namespace {

Theme::Colors g_colors;
bool g_ready = false;

QString rgba(const QColor &color, int alpha)
{
    return QStringLiteral("rgba(%1, %2, %3, %4)")
        .arg(color.red())
        .arg(color.green())
        .arg(color.blue())
        .arg(alpha);
}

Theme::Colors detectColors(const QColor &window)
{
    bool dark = window.lightness() < 140;
    const QString forced = qEnvironmentVariable("ORBISCREEN_THEME").toLower();
    if (forced == QStringLiteral("dark")) {
        dark = true;
    } else if (forced == QStringLiteral("light")) {
        dark = false;
    }
    Theme::Colors c;
    c.dark = dark;
    if (dark) {
        c.background = QColor(QStringLiteral("#0E1116"));
        c.surface = QColor(QStringLiteral("#151A21"));
        c.surfaceAlt = QColor(QStringLiteral("#1B222B"));
        c.surfaceHover = QColor(QStringLiteral("#232C38"));
        c.border = QColor(QStringLiteral("#252D38"));
        c.borderStrong = QColor(QStringLiteral("#333D4B"));
        c.text = QColor(QStringLiteral("#E8EDF5"));
        c.muted = QColor(QStringLiteral("#93A0B4"));
        c.accent = QColor(QStringLiteral("#4C8DFF"));
        c.accentText = QColor(QStringLiteral("#FFFFFF"));
        c.success = QColor(QStringLiteral("#34D399"));
        c.warning = QColor(QStringLiteral("#FBBF24"));
        c.danger = QColor(QStringLiteral("#F87171"));
        c.qrDark = QColor(QStringLiteral("#0B1220"));
    } else {
        c.background = QColor(QStringLiteral("#F3F5F9"));
        c.surface = QColor(QStringLiteral("#FFFFFF"));
        c.surfaceAlt = QColor(QStringLiteral("#F6F8FC"));
        c.surfaceHover = QColor(QStringLiteral("#EDF1F8"));
        c.border = QColor(QStringLiteral("#E1E6EF"));
        c.borderStrong = QColor(QStringLiteral("#CBD3E1"));
        c.text = QColor(QStringLiteral("#141A24"));
        c.muted = QColor(QStringLiteral("#64708A"));
        c.accent = QColor(QStringLiteral("#2563EB"));
        c.accentText = QColor(QStringLiteral("#FFFFFF"));
        c.success = QColor(QStringLiteral("#059669"));
        c.warning = QColor(QStringLiteral("#B45309"));
        c.danger = QColor(QStringLiteral("#DC2626"));
        c.qrDark = QColor(QStringLiteral("#111827"));
    }
    c.accentHover = dark ? c.accent.lighter(112) : c.accent.darker(108);
    c.accentPress = dark ? c.accent.darker(112) : c.accent.darker(118);
    return c;
}

qreal screenRatio()
{
    if (QScreen *screen = QGuiApplication::primaryScreen()) {
        return screen->devicePixelRatio();
    }
    return 1.0;
}

void paintArrowHead(QPainter *painter, const QPointF &tip, const QPointF &direction,
                    qreal length, qreal half, const QColor &color)
{
    QPointF d = direction;
    const qreal norm = std::hypot(d.x(), d.y());
    if (norm > 0.0) {
        d /= norm;
    }
    const QPointF base = tip - d * length;
    const QPointF normal(-d.y(), d.x());
    QPolygonF head;
    head << tip << base + normal * half << base - normal * half;
    painter->setPen(Qt::NoPen);
    painter->setBrush(color);
    painter->drawPolygon(head);
    painter->setBrush(Qt::NoBrush);
}

void paintCheck(QPainter *painter, const QColor &color)
{
    painter->setPen(Qt::NoPen);
    painter->setBrush(color);
    painter->drawEllipse(QPointF(12, 12), 9.0, 9.0);
    QPen pen(QColor(QStringLiteral("#FFFFFF")), 2.0, Qt::SolidLine, Qt::RoundCap,
             Qt::RoundJoin);
    painter->setPen(pen);
    QPainterPath path;
    path.moveTo(7.6, 12.4);
    path.lineTo(10.7, 15.4);
    path.lineTo(16.6, 8.8);
    painter->drawPath(path);
}

void paintWarning(QPainter *painter, const QColor &color)
{
    QPainterPath path;
    path.moveTo(12.0, 3.6);
    path.lineTo(21.6, 20.2);
    path.lineTo(2.4, 20.2);
    path.closeSubpath();
    painter->setPen(QPen(color, 1.6, Qt::SolidLine, Qt::RoundCap, Qt::RoundJoin));
    painter->setBrush(color);
    painter->drawPath(path);
    painter->setPen(QPen(QColor(QStringLiteral("#FFFFFF")), 1.9, Qt::SolidLine, Qt::RoundCap));
    painter->drawLine(QPointF(12.0, 9.2), QPointF(12.0, 14.6));
    painter->setPen(Qt::NoPen);
    painter->setBrush(QColor(QStringLiteral("#FFFFFF")));
    painter->drawEllipse(QPointF(12.0, 17.3), 1.1, 1.1);
}

void paintError(QPainter *painter, const QColor &color)
{
    painter->setPen(Qt::NoPen);
    painter->setBrush(color);
    painter->drawEllipse(QPointF(12, 12), 9.0, 9.0);
    painter->setPen(QPen(QColor(QStringLiteral("#FFFFFF")), 2.0, Qt::SolidLine, Qt::RoundCap));
    painter->drawLine(QPointF(8.6, 8.6), QPointF(15.4, 15.4));
    painter->drawLine(QPointF(15.4, 8.6), QPointF(8.6, 15.4));
}

void paintGlyph(QPainter *painter, Theme::Glyph glyph, const QColor &color)
{
    const QPen pen(color, 2.0, Qt::SolidLine, Qt::RoundCap, Qt::RoundJoin);
    painter->setPen(pen);
    painter->setBrush(Qt::NoBrush);

    switch (glyph) {
    case Theme::Glyph::Play: {
        painter->setPen(QPen(color, 1.6, Qt::SolidLine, Qt::RoundCap, Qt::RoundJoin));
        painter->setBrush(color);
        QPainterPath path;
        path.moveTo(8.4, 5.4);
        path.lineTo(18.8, 12.0);
        path.lineTo(8.4, 18.6);
        path.closeSubpath();
        painter->drawPath(path);
        break;
    }
    case Theme::Glyph::Stop: {
        painter->setPen(Qt::NoPen);
        painter->setBrush(color);
        painter->drawRoundedRect(QRectF(7.2, 7.2, 9.6, 9.6), 2.4, 2.4);
        break;
    }
    case Theme::Glyph::Restart: {
        const QRectF arc(3.4, 3.4, 17.2, 17.2);
        painter->drawArc(arc, 90 * 16, -280 * 16);
        const qreal radius = 8.6;
        const qreal angle = qDegreesToRadians(-10.0);
        const QPointF point(12 + radius * qCos(angle), 12 - radius * qSin(angle));
        const QPointF tangent(qSin(angle), qCos(angle));
        paintArrowHead(painter, point + tangent * 2.2, tangent, 5.8, 2.8, color);
        break;
    }
    case Theme::Glyph::Refresh: {
        const QRectF arc(3.4, 3.4, 17.2, 17.2);
        painter->drawArc(arc, 90 * 16, 280 * 16);
        const qreal radius = 8.6;
        const qreal angle = qDegreesToRadians(10.0);
        const QPointF point(12 + radius * qCos(angle), 12 - radius * qSin(angle));
        const QPointF tangent(-qSin(angle), -qCos(angle));
        paintArrowHead(painter, point + tangent * 2.2, tangent, 5.8, 2.8, color);
        break;
    }
    case Theme::Glyph::Dashboard: {
        painter->drawEllipse(QPointF(12, 12), 8.6, 8.6);
        painter->drawLine(QPointF(3.4, 12.0), QPointF(20.6, 12.0));
        painter->drawEllipse(QRectF(7.6, 3.4, 8.8, 17.2));
        break;
    }
    case Theme::Glyph::Phone: {
        painter->drawRoundedRect(QRectF(7.0, 3.0, 10.0, 18.0), 2.4, 2.4);
        painter->drawLine(QPointF(10.4, 18.2), QPointF(13.6, 18.2));
        break;
    }
    case Theme::Glyph::Monitor: {
        painter->drawRoundedRect(QRectF(3.0, 5.0, 18.0, 12.0), 2.2, 2.2);
        painter->drawLine(QPointF(12.0, 17.0), QPointF(12.0, 20.0));
        painter->drawLine(QPointF(8.6, 20.0), QPointF(15.4, 20.0));
        break;
    }
    case Theme::Glyph::Doctor: {
        QPainterPath path;
        path.moveTo(3.2, 12.0);
        path.lineTo(7.6, 12.0);
        path.lineTo(9.6, 6.8);
        path.lineTo(12.6, 17.2);
        path.lineTo(14.6, 12.0);
        path.lineTo(20.8, 12.0);
        painter->drawPath(path);
        break;
    }
    case Theme::Glyph::Wifi: {
        painter->drawArc(QRectF(8.0, 14.0, 8.0, 8.0), 45 * 16, 90 * 16);
        painter->drawArc(QRectF(5.0, 11.0, 14.0, 14.0), 45 * 16, 90 * 16);
        painter->drawArc(QRectF(2.0, 8.0, 20.0, 20.0), 45 * 16, 90 * 16);
        painter->setPen(Qt::NoPen);
        painter->setBrush(color);
        painter->drawEllipse(QPointF(12.0, 19.2), 1.5, 1.5);
        break;
    }
    case Theme::Glyph::Usb: {
        painter->drawLine(QPointF(12.0, 4.4), QPointF(12.0, 20.2));
        painter->drawEllipse(QPointF(12.0, 20.6), 1.5, 1.5);
        painter->setPen(Qt::NoPen);
        painter->setBrush(color);
        QPolygonF head;
        head << QPointF(12.0, 2.4) << QPointF(9.7, 6.4) << QPointF(14.3, 6.4);
        painter->drawPolygon(head);
        painter->setPen(pen);
        painter->setBrush(Qt::NoBrush);
        QPainterPath left;
        left.moveTo(12.0, 16.2);
        left.lineTo(6.6, 12.6);
        left.lineTo(6.6, 9.6);
        painter->drawPath(left);
        painter->drawRect(QRectF(5.3, 6.8, 2.6, 2.6));
        QPainterPath right;
        right.moveTo(12.0, 12.6);
        right.lineTo(17.4, 9.2);
        right.lineTo(17.4, 7.6);
        painter->drawPath(right);
        painter->setPen(Qt::NoPen);
        painter->setBrush(color);
        painter->drawEllipse(QPointF(17.4, 5.9), 1.4, 1.4);
        break;
    }
    case Theme::Glyph::Copy: {
        painter->drawRoundedRect(QRectF(4.0, 4.0, 12.0, 12.0), 2.4, 2.4);
        painter->drawRoundedRect(QRectF(8.0, 8.0, 12.0, 12.0), 2.4, 2.4);
        break;
    }
    case Theme::Glyph::ExternalLink: {
        painter->drawLine(QPointF(19.6, 4.2), QPointF(11.4, 12.4));
        QPainterPath path;
        path.moveTo(20.4, 10.2);
        path.lineTo(20.4, 4.0);
        path.lineTo(14.2, 4.0);
        painter->drawPath(path);
        QPainterPath box;
        box.moveTo(18.0, 14.4);
        box.lineTo(18.0, 18.4);
        box.quadTo(18.0, 20.2, 16.2, 20.2);
        box.lineTo(6.2, 20.2);
        box.quadTo(4.4, 20.2, 4.4, 18.4);
        box.lineTo(4.4, 8.4);
        box.quadTo(4.4, 6.6, 6.2, 6.6);
        box.lineTo(10.2, 6.6);
        painter->drawPath(box);
        break;
    }
    case Theme::Glyph::Check:
        paintCheck(painter, color);
        break;
    case Theme::Glyph::CheckMark: {
        QPainterPath path;
        path.moveTo(5.4, 12.6);
        path.lineTo(10.1, 17.2);
        path.lineTo(18.6, 7.6);
        painter->setPen(QPen(color, 2.2, Qt::SolidLine, Qt::RoundCap, Qt::RoundJoin));
        painter->drawPath(path);
        break;
    }
    case Theme::Glyph::Warning:
        paintWarning(painter, color);
        break;
    case Theme::Glyph::Error:
        paintError(painter, color);
        break;
    case Theme::Glyph::Info: {
        painter->drawEllipse(QPointF(12, 12), 8.6, 8.6);
        painter->drawLine(QPointF(12.0, 11.2), QPointF(12.0, 16.6));
        painter->setPen(Qt::NoPen);
        painter->setBrush(color);
        painter->drawEllipse(QPointF(12.0, 7.9), 1.15, 1.15);
        break;
    }
    case Theme::Glyph::Download: {
        painter->drawLine(QPointF(12.0, 3.6), QPointF(12.0, 14.6));
        QPainterPath arrow;
        arrow.moveTo(7.4, 10.2);
        arrow.lineTo(12.0, 14.8);
        arrow.lineTo(16.6, 10.2);
        painter->drawPath(arrow);
        QPainterPath tray;
        tray.moveTo(4.6, 15.6);
        tray.lineTo(4.6, 18.4);
        tray.quadTo(4.6, 20.4, 6.6, 20.4);
        tray.lineTo(17.4, 20.4);
        tray.quadTo(19.4, 20.4, 19.4, 18.4);
        tray.lineTo(19.4, 15.6);
        painter->drawPath(tray);
        break;
    }
    case Theme::Glyph::More: {
        painter->setPen(Qt::NoPen);
        painter->setBrush(color);
        painter->drawEllipse(QPointF(12.0, 5.4), 1.9, 1.9);
        painter->drawEllipse(QPointF(12.0, 12.0), 1.9, 1.9);
        painter->drawEllipse(QPointF(12.0, 18.6), 1.9, 1.9);
        break;
    }
    case Theme::Glyph::Chevron: {
        QPainterPath path;
        path.moveTo(7.4, 10.0);
        path.lineTo(12.0, 14.8);
        path.lineTo(16.6, 10.0);
        painter->drawPath(path);
        break;
    }
    case Theme::Glyph::Chip: {
        painter->drawRoundedRect(QRectF(7.0, 7.0, 10.0, 10.0), 2.2, 2.2);
        const qreal offsets[3] = {9.0, 12.0, 15.0};
        for (qreal offset : offsets) {
            painter->drawLine(QPointF(offset, 3.4), QPointF(offset, 7.0));
            painter->drawLine(QPointF(offset, 17.0), QPointF(offset, 20.6));
            painter->drawLine(QPointF(3.4, offset), QPointF(7.0, offset));
            painter->drawLine(QPointF(17.0, offset), QPointF(20.6, offset));
        }
        break;
    }
    case Theme::Glyph::Pen: {
        QPainterPath body;
        body.moveTo(4.6, 19.4);
        body.lineTo(6.2, 14.4);
        body.lineTo(15.4, 5.2);
        body.quadTo(16.7, 3.9, 18.0, 5.2);
        body.quadTo(19.3, 6.5, 18.0, 7.8);
        body.lineTo(8.8, 17.0);
        body.closeSubpath();
        painter->drawPath(body);
        painter->drawLine(QPointF(14.8, 5.8), QPointF(18.2, 9.2));
        break;
    }
    }
}

void paintLogo(QPainter *painter, const QColor &color)
{
    painter->setPen(QPen(color, 6.75, Qt::SolidLine, Qt::RoundCap));
    painter->setBrush(Qt::NoBrush);
    painter->drawEllipse(QPointF(32.0, 32.0), 21.25, 21.25);
    painter->setPen(Qt::NoPen);
    painter->setBrush(color);
    painter->drawEllipse(QPointF(47.0, 47.0), 7.75, 7.75);
}

class GlyphEngine : public QIconEngine {
public:
    GlyphEngine(Theme::Glyph glyph, const QColor &color, bool fixed)
        : m_glyph(glyph)
        , m_color(color)
        , m_fixed(fixed)
    {
    }

    void paint(QPainter *painter, const QRect &rect, QIcon::Mode mode, QIcon::State) override
    {
        const Theme::Colors &c = Theme::colors();
        QColor color = m_color;
        if (!m_fixed) {
            if (mode == QIcon::Disabled) {
                color = c.muted;
            } else if (mode == QIcon::Active || mode == QIcon::Selected) {
                color = c.accent;
            } else {
                color = c.text;
            }
        }
        draw(painter, rect, color);
    }

    QPixmap pixmap(const QSize &size, QIcon::Mode mode, QIcon::State state) override
    {
        return render(size, mode, state, 1.0);
    }

    QPixmap scaledPixmap(const QSize &size, QIcon::Mode mode, QIcon::State state,
                         qreal scale) override
    {
        return render(size, mode, state, scale);
    }

    QIconEngine *clone() const override { return new GlyphEngine(*this); }

private:
    QPixmap render(const QSize &size, QIcon::Mode mode, QIcon::State state, qreal scale)
    {
        const qreal ratio = scale > 0.0 ? scale : 1.0;
        QPixmap pixmap(QSize(qRound(size.width() * ratio), qRound(size.height() * ratio)));
        pixmap.setDevicePixelRatio(ratio);
        pixmap.fill(Qt::transparent);
        QPainter painter(&pixmap);
        painter.setRenderHint(QPainter::Antialiasing, true);
        paint(&painter, QRect(QPoint(0, 0), size), mode, state);
        return pixmap;
    }

    void draw(QPainter *painter, const QRect &rect, const QColor &color) const
    {
        painter->save();
        painter->setRenderHint(QPainter::Antialiasing, true);
        const qreal side = qMin(rect.width(), rect.height());
        if (side <= 0.0) {
            painter->restore();
            return;
        }
        painter->translate(rect.x() + (rect.width() - side) / 2.0,
                           rect.y() + (rect.height() - side) / 2.0);
        painter->scale(side / 24.0, side / 24.0);
        paintGlyph(painter, m_glyph, color);
        painter->restore();
    }

    Theme::Glyph m_glyph;
    QColor m_color;
    bool m_fixed;
};

QString buildStyleSheet(const Theme::Colors &c)
{
    QString css = QStringLiteral(R"qss(
QMainWindow { background: @bg@; }
QWidget#Central { background: @bg@; }
QWidget#Header { background: @surface@; border-bottom: 1px solid @border@; }
QLabel { color: @text@; }
QLabel#AppTitle { font-size: 20px; font-weight: 600; }
QLabel#Muted { color: @muted@; }
QLabel#Value { font-weight: 600; }
QLabel#CardTitle { font-weight: 600; }
QLabel#Badge { background: @accentSoft@; border-radius: 12px; }
QLabel#StepBadge { background: @accentSoft@; color: @accent@; border-radius: 13px; font-weight: 700; }
QLabel#StatusText { color: @muted@; }
QLabel#StatusState { color: @text@; font-weight: 600; }

QFrame#Card { background: @surface@; border: 1px solid @border@; border-radius: 14px; }
QFrame#StatTile { background: @surfaceAlt@; border: 1px solid @border@; border-radius: 10px; }
QFrame#Divider { background: @border@; border: none; }
QFrame#UpdateBar { background: @accentSoft@; border: 1px solid @accentBorder@; border-radius: 12px; }
QFrame#StatusPill { border-radius: 14px; }
QFrame#StatusPill[state="stopped"] { background: @dangerSoft@; border: 1px solid @dangerBorder@; }
QFrame#StatusPill[state="ready"] { background: @successSoft@; border: 1px solid @successBorder@; }
QFrame#StatusPill[state="streaming"] { background: @accentSoft@; border: 1px solid @accentBorder@; }
QFrame#StatusPill[state="stopped"] QLabel { color: @danger@; font-weight: 600; }
QFrame#StatusPill[state="ready"] QLabel { color: @success@; font-weight: 600; }
QFrame#StatusPill[state="streaming"] QLabel { color: @accent@; font-weight: 600; }

QPushButton { background: @surfaceAlt@; color: @text@; border: 1px solid @border@; border-radius: 10px; padding: 7px 16px; min-height: 22px; font-weight: 500; }
QPushButton:hover { background: @surfaceHover@; border-color: @borderStrong@; }
QPushButton:pressed { background: @border@; }
QPushButton:focus { border-color: @accent@; outline: none; }
QPushButton:disabled { color: @muted@; background: @surface@; border-color: @border@; }
QPushButton[variant="primary"] { background: @accent@; color: @accentText@; border: 1px solid @accent@; font-weight: 600; }
QPushButton[variant="primary"]:hover { background: @accentHover@; border-color: @accentHover@; }
QPushButton[variant="primary"]:pressed { background: @accentPress@; border-color: @accentPress@; }
QPushButton[variant="primary"]:disabled { background: @surfaceAlt@; color: @muted@; border-color: @border@; }
QPushButton[variant="danger"] { background: @dangerSoft@; color: @danger@; border: 1px solid @dangerBorder@; font-weight: 600; }
QPushButton[variant="danger"]:hover { background: @dangerHover@; }
QPushButton[variant="ghost"] { background: transparent; color: @muted@; border: 1px solid transparent; }
QPushButton[variant="ghost"]:hover { background: @surfaceHover@; color: @text@; }

QToolButton#IconButton { background: transparent; border: none; border-radius: 9px; padding: 6px; }
QToolButton#IconButton:hover { background: @surfaceHover@; }
QToolButton#IconButton:pressed { background: @border@; }
QToolButton#IconButton:disabled { background: transparent; }
QToolButton#HeaderAction { background: @accent@; color: @accentText@; border: 1px solid @accent@; border-radius: 10px; padding: 8px 18px; font-weight: 600; }
QToolButton#HeaderAction:hover { background: @accentHover@; border-color: @accentHover@; }
QToolButton#HeaderAction:pressed { background: @accentPress@; }
QToolButton#HeaderAction[variant="danger"] { background: @dangerSoft@; color: @danger@; border: 1px solid @dangerBorder@; }
QToolButton#HeaderAction[variant="danger"]:hover { background: @dangerHover@; }
QToolButton#HeaderAction:disabled { background: @surfaceAlt@; color: @muted@; border-color: @border@; }
QToolButton::menu-indicator { image: none; width: 0px; }

QLineEdit { background: @surfaceAlt@; color: @text@; border: 1px solid @border@; border-radius: 10px; padding: 8px 12px; selection-background-color: @accent@; selection-color: @accentText@; }
QLineEdit:hover { border-color: @borderStrong@; }
QLineEdit:focus { border-color: @accent@; }

QComboBox { background: @surfaceAlt@; color: @text@; border: 1px solid @border@; border-radius: 10px; padding: 7px 34px 7px 12px; min-height: 22px; }
QComboBox:hover { border-color: @borderStrong@; }
QComboBox:focus { border-color: @accent@; outline: none; }
QComboBox::drop-down { width: 0px; border: none; }
QComboBox::down-arrow { image: none; width: 0px; height: 0px; }
QComboBox QAbstractItemView { background: @surface@; color: @text@; border: 1px solid @border@; border-radius: 10px; padding: 4px; outline: none; selection-background-color: @accentSoft@; selection-color: @text@; }
QComboBox QAbstractItemView::item { padding: 6px 10px; border-radius: 6px; min-height: 22px; }

QTabWidget::pane { border: none; background: transparent; }
QTabWidget::tab-bar { left: 10px; }
QTabBar { background: transparent; }
QTabBar::tab { background: transparent; color: @muted@; border: 1px solid transparent; border-radius: 10px; padding: 8px 16px; margin: 10px 6px 4px 0px; font-weight: 500; }
QTabBar::tab:hover { color: @text@; background: @surfaceAlt@; }
QTabBar::tab:selected { color: @accent@; background: @accentSoft@; border: 1px solid @accentBorder@; }

QScrollArea { border: none; background: transparent; }
QScrollArea > QWidget > QWidget { background: transparent; }
QAbstractScrollArea::corner { background: transparent; }
QScrollBar:vertical { background: transparent; width: 10px; margin: 2px; }
QScrollBar::handle:vertical { background: @borderStrong@; border-radius: 3px; min-height: 32px; }
QScrollBar::handle:vertical:hover { background: @muted@; }
QScrollBar:horizontal { background: transparent; height: 10px; margin: 2px; }
QScrollBar::handle:horizontal { background: @borderStrong@; border-radius: 3px; min-width: 32px; }
QScrollBar::handle:horizontal:hover { background: @muted@; }
QScrollBar::add-line, QScrollBar::sub-line { width: 0px; height: 0px; }
QScrollBar::add-page, QScrollBar::sub-page { background: transparent; }

QStatusBar { background: @surface@; border-top: 1px solid @border@; }
QStatusBar::item { border: none; }
QStatusBar QLabel { padding: 2px 4px; }

QMenu { background: @surface@; color: @text@; border: 1px solid @border@; border-radius: 10px; padding: 6px; }
QMenu::item { padding: 7px 30px 7px 14px; border-radius: 6px; }
QMenu::item:selected { background: @accentSoft@; color: @text@; }
QMenu::item:disabled { color: @muted@; }
QMenu::separator { height: 1px; background: @border@; margin: 6px 10px; }

QToolTip { background: @surface@; color: @text@; border: 1px solid @border@; border-radius: 8px; padding: 6px 8px; }
QMessageBox { background: @bg@; }
QMessageBox QLabel { color: @text@; }
)qss");

    const auto replace = [&css](const char *key, const QString &value) {
        css.replace(QLatin1String(key), value);
    };
    replace("@bg@", c.background.name());
    replace("@surfaceAlt@", c.surfaceAlt.name());
    replace("@surfaceHover@", c.surfaceHover.name());
    replace("@surface@", c.surface.name());
    replace("@borderStrong@", c.borderStrong.name());
    replace("@border@", c.border.name());
    replace("@text@", c.text.name());
    replace("@muted@", c.muted.name());
    replace("@accentSoft@", rgba(c.accent, c.dark ? 46 : 30));
    replace("@accentBorder@", rgba(c.accent, c.dark ? 90 : 70));
    replace("@accentHover@", c.accentHover.name());
    replace("@accentPress@", c.accentPress.name());
    replace("@accentText@", c.accentText.name());
    replace("@accent@", c.accent.name());
    replace("@successSoft@", rgba(c.success, c.dark ? 40 : 28));
    replace("@successBorder@", rgba(c.success, c.dark ? 90 : 70));
    replace("@success@", c.success.name());
    replace("@dangerSoft@", rgba(c.danger, c.dark ? 40 : 28));
    replace("@dangerHover@", rgba(c.danger, c.dark ? 64 : 46));
    replace("@dangerBorder@", rgba(c.danger, c.dark ? 90 : 70));
    replace("@danger@", c.danger.name());
    return css;
}

}

namespace Theme {

void apply()
{
    QApplication::setApplicationName(QStringLiteral("orbiscreen-gui"));
    QApplication::setApplicationDisplayName(QStringLiteral("Orbiscreen"));
    QApplication::setDesktopFileName(QStringLiteral("orbiscreen.gui"));

    const QColor systemWindow = QApplication::palette().color(QPalette::Window);
    if (QStyle *fusion = QStyleFactory::create(QStringLiteral("Fusion"))) {
        QApplication::setStyle(fusion);
    }

    g_colors = detectColors(systemWindow);
    g_ready = true;
    const Colors &c = g_colors;

    QPalette palette = QApplication::palette();
    palette.setColor(QPalette::Window, c.background);
    palette.setColor(QPalette::WindowText, c.text);
    palette.setColor(QPalette::Base, c.surface);
    palette.setColor(QPalette::AlternateBase, c.surfaceAlt);
    palette.setColor(QPalette::Text, c.text);
    palette.setColor(QPalette::Button, c.surfaceAlt);
    palette.setColor(QPalette::ButtonText, c.text);
    palette.setColor(QPalette::Highlight, c.accent);
    palette.setColor(QPalette::HighlightedText, c.accentText);
    palette.setColor(QPalette::PlaceholderText, c.muted);
    palette.setColor(QPalette::ToolTipBase, c.surface);
    palette.setColor(QPalette::ToolTipText, c.text);
    palette.setColor(QPalette::Light, c.surfaceHover);
    palette.setColor(QPalette::Midlight, c.surfaceHover);
    palette.setColor(QPalette::Mid, c.border);
    palette.setColor(QPalette::Dark, c.borderStrong);
    palette.setColor(QPalette::Shadow, c.background);
    palette.setColor(QPalette::Link, c.accent);
    palette.setColor(QPalette::Disabled, QPalette::Text, c.muted);
    palette.setColor(QPalette::Disabled, QPalette::WindowText, c.muted);
    palette.setColor(QPalette::Disabled, QPalette::ButtonText, c.muted);
    palette.setColor(QPalette::Disabled, QPalette::Highlight, c.borderStrong);
    palette.setColor(QPalette::Disabled, QPalette::HighlightedText, c.muted);
    QApplication::setPalette(palette);

    qApp->setStyleSheet(buildStyleSheet(c));
    QApplication::setWindowIcon(logoIcon());
}

const Colors &colors()
{
    if (!g_ready) {
        g_colors = detectColors(QApplication::palette().color(QPalette::Window));
        g_ready = true;
    }
    return g_colors;
}

QColor statusColor(const QString &kind)
{
    if (kind == QStringLiteral("ok")) {
        return colors().success;
    }
    if (kind == QStringLiteral("warn")) {
        return colors().warning;
    }
    return colors().danger;
}

QIcon glyph(Glyph value)
{
    return QIcon(new GlyphEngine(value, QColor(), false));
}

QIcon glyph(Glyph value, const QColor &color)
{
    return QIcon(new GlyphEngine(value, color, true));
}

QPixmap glyphPixmap(Glyph value, int size, const QColor &color)
{
    return glyph(value, color).pixmap(QSize(size, size), screenRatio());
}

QPixmap logo(int size)
{
    const qreal ratio = screenRatio();
    QPixmap pixmap(QSize(qRound(size * ratio), qRound(size * ratio)));
    pixmap.setDevicePixelRatio(ratio);
    pixmap.fill(Qt::transparent);
    QPainter painter(&pixmap);
    painter.setRenderHint(QPainter::Antialiasing, true);
    painter.scale(size / 64.0, size / 64.0);
    paintLogo(&painter, colors().dark ? QColor(QStringLiteral("#89B4FA"))
                                      : QColor(QStringLiteral("#4C8DFF")));
    return pixmap;
}

QIcon logoIcon()
{
    const qreal ratio = screenRatio();
    const auto render = [ratio](int size) {
        QPixmap pixmap(QSize(qRound(size * ratio), qRound(size * ratio)));
        pixmap.setDevicePixelRatio(ratio);
        pixmap.fill(Qt::transparent);
        QPainter painter(&pixmap);
        painter.setRenderHint(QPainter::Antialiasing, true);
        painter.scale(size / 64.0, size / 64.0);
        paintLogo(&painter, QColor(QStringLiteral("#89B4FA")));
        return pixmap;
    };
    QIcon icon;
    const int sizes[] = {16, 24, 32, 48, 64, 128, 256};
    for (int size : sizes) {
        icon.addPixmap(render(size));
    }
    return icon;
}

}
