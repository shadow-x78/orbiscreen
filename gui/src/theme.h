// Orbiscreen - Qt GUI theme helpers (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#pragma once

#include <QIcon>
#include <QStringList>
#include <QStyle>

#include <functional>

namespace Theme {

void apply();
void resolveSystemIconTheme(std::function<void()> ready);
QIcon icon(const QStringList &themeNames, QStyle::StandardPixmap fallback);
QIcon statusIcon(const QString &kind);
QString iconName(const QString &kind);

}
