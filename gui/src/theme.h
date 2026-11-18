// Orbiscreen - Qt GUI theme helpers (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#pragma once

#include <QColor>
#include <QFont>
#include <QIcon>
#include <QPixmap>
#include <QRectF>
#include <QString>

class QPainter;

namespace Theme {

enum class Glyph {
    Play,
    Stop,
    Restart,
    Refresh,
    Dashboard,
    Phone,
    Monitor,
    Doctor,
    Wifi,
    Usb,
    Copy,
    ExternalLink,
    Check,
    CheckMark,
    Warning,
    Error,
    Info,
    Download,
    More,
    Chevron,
    Chip,
    Pen,
    Touch,
    Login,
    Count,
};

struct Colors {
    bool dark = true;
    QColor background;
    QColor surface;
    QColor surfaceAlt;
    QColor surfaceHover;
    QColor border;
    QColor borderStrong;
    QColor text;
    QColor muted;
    QColor accent;
    QColor accentHover;
    QColor accentPress;
    QColor accentText;
    QColor brand;
    QColor success;
    QColor warning;
    QColor danger;
    QColor qrDark;
};

namespace Spacing {
constexpr int xs = 4;
constexpr int sm = 8;
constexpr int md = 12;
constexpr int lg = 16;
constexpr int xl = 20;
constexpr int xxl = 24;
constexpr int xxxl = 32;
}

namespace Radius {
constexpr int control = 10;
constexpr int tile = 12;
constexpr int card = 16;
constexpr int pill = 999;
}

namespace Metrics {
constexpr int contentMaxWidth = 1040;
constexpr int pageGutter = 24;
constexpr int cardGap = 16;
constexpr int headerHeight = 64;
constexpr int controlHeight = 36;
constexpr int iconButtonSize = 36;
constexpr int breakpointTwoColumns = 780;
constexpr int statusBarHeight = 28;
constexpr int qrTileSize = 168;
constexpr int badgeSize = 24;
constexpr int tabPadding = 14;
constexpr int tabIconSize = 16;
constexpr int buttonIconSize = 16;
constexpr int cardIconSize = 20;
}

enum class FontRole {
    Caption,
    Body,
    Label,
    SectionTitle,
    Headline,
    AppTitle,
};

void apply();
const Colors &colors();
QColor statusColor(const QString &kind);

QFont font(FontRole role);
QFont tabular(const QFont &base);
QIcon glyph(Glyph glyph);
QIcon glyph(Glyph glyph, const QColor &color);
QPixmap glyphPixmap(Glyph glyph, int size, const QColor &color, qreal dpr = 1.0);
QPixmap logo(int size, qreal dpr = 1.0);
QIcon logoIcon();

void paintGlyph(QPainter *painter, Glyph glyph, const QRectF &rect, const QColor &color);
void paintLogo(QPainter *painter, const QRectF &rect, const QColor &color,
               bool compact = false);

}
