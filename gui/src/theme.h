// Orbiscreen - Qt GUI theme helpers (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#pragma once

#include <QColor>
#include <QIcon>
#include <QPixmap>
#include <QString>

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
    QColor success;
    QColor warning;
    QColor danger;
    QColor qrDark;
};

void apply();
const Colors &colors();
QColor statusColor(const QString &kind);
QIcon glyph(Glyph glyph);
QIcon glyph(Glyph glyph, const QColor &color);
QPixmap glyphPixmap(Glyph glyph, int size, const QColor &color);
QPixmap logo(int size);
QIcon logoIcon();

}
