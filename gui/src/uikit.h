// Orbiscreen - Qt GUI widget helpers (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#pragma once

#include <QGroupBox>
#include <QLabel>
#include <QPushButton>
#include <QStyle>

class QFormLayout;

namespace UiKit {

QLabel *label(const QString &text, bool dim = false, bool wrap = false);
QLabel *valueLabel();
QGroupBox *group(const QString &title);
QFormLayout *form();
QPushButton *button(const QString &text, bool primary = false);
QLabel *iconLabel(const QStringList &themeNames, QStyle::StandardPixmap fallback, int size = 18);
QLabel *statusIcon(const QString &kind, int size = 16);
void setStatusIcon(QLabel *label, const QString &kind, int size = 16);

}
