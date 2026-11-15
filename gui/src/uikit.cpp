// Orbiscreen - Qt GUI widget helpers (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#include "uikit.h"

#include "theme.h"

#include <QFont>
#include <QFormLayout>
#include <QPalette>
#include <QPixmap>
#include <QSizePolicy>

namespace UiKit {

QLabel *label(const QString &text, bool dim, bool wrap)
{
    auto *widget = new QLabel(text);
    widget->setTextFormat(Qt::PlainText);
    widget->setWordWrap(wrap);
    widget->setAlignment(Qt::AlignLeft | Qt::AlignVCenter);
    if (dim) {
        QFont font = widget->font();
        if (font.pointSizeF() > 0.0) {
            font.setPointSizeF(font.pointSizeF() * 0.94);
        }
        widget->setFont(font);
        QPalette palette = widget->palette();
        palette.setColor(QPalette::WindowText, palette.color(QPalette::PlaceholderText));
        widget->setPalette(palette);
    }
    return widget;
}

QLabel *valueLabel()
{
    auto *widget = new QLabel(QStringLiteral("-"));
    widget->setTextFormat(Qt::PlainText);
    widget->setAlignment(Qt::AlignRight | Qt::AlignVCenter);
    widget->setTextInteractionFlags(Qt::TextSelectableByMouse);
    widget->setSizePolicy(QSizePolicy::Preferred, QSizePolicy::Preferred);
    return widget;
}

QGroupBox *group(const QString &title)
{
    auto *box = new QGroupBox(title);
    box->setSizePolicy(QSizePolicy::Preferred, QSizePolicy::Preferred);
    return box;
}

QFormLayout *form()
{
    auto *layout = new QFormLayout;
    layout->setFieldGrowthPolicy(QFormLayout::AllNonFixedFieldsGrow);
    layout->setRowWrapPolicy(QFormLayout::WrapLongRows);
    layout->setLabelAlignment(Qt::AlignLeft | Qt::AlignVCenter);
    layout->setContentsMargins(12, 10, 12, 10);
    layout->setHorizontalSpacing(14);
    layout->setVerticalSpacing(8);
    return layout;
}

QPushButton *button(const QString &text, bool primary)
{
    auto *widget = new QPushButton(text);
    if (primary) {
        widget->setDefault(true);
        widget->setAutoDefault(true);
    }
    return widget;
}

QLabel *iconLabel(const QStringList &themeNames, QStyle::StandardPixmap fallback, int size)
{
    auto *widget = new QLabel;
    widget->setFixedSize(size, size);
    widget->setAlignment(Qt::AlignCenter);
    widget->setPixmap(Theme::icon(themeNames, fallback).pixmap(size, size));
    return widget;
}

QLabel *statusIcon(const QString &kind, int size)
{
    auto *widget = new QLabel;
    widget->setFixedSize(size, size);
    widget->setAlignment(Qt::AlignCenter);
    widget->setPixmap(Theme::statusIcon(kind).pixmap(size, size));
    return widget;
}

void setStatusIcon(QLabel *label, const QString &kind, int size)
{
    if (!label) {
        return;
    }
    label->setPixmap(Theme::statusIcon(kind).pixmap(size, size));
    label->setToolTip(QString());
}

}
