// Orbiscreen - Qt GUI widget helpers (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#pragma once

#include <QFrame>
#include <QLabel>
#include <QString>

#include "theme.h"

class QAbstractButton;
class QComboBox;
class QHBoxLayout;
class QPushButton;
class QToolButton;
class QVBoxLayout;

namespace UiKit {

struct StatTile {
    QFrame *frame = nullptr;
    QLabel *caption = nullptr;
    QLabel *value = nullptr;
};

class Card : public QFrame {
public:
    Card(Theme::Glyph glyph, const QString &title, QWidget *parent = nullptr);

    void setTitle(const QString &title);
    QVBoxLayout *body() const { return m_body; }

private:
    QLabel *m_icon = nullptr;
    QLabel *m_title = nullptr;
    QVBoxLayout *m_body = nullptr;
};

class StatusPill : public QFrame {
public:
    explicit StatusPill(QWidget *parent = nullptr);

    void setStatus(const QString &state, const QString &text);

private:
    QLabel *m_dot = nullptr;
    QLabel *m_text = nullptr;
    QString m_state;
};

Card *card(Theme::Glyph glyph, const QString &title, QWidget *parent = nullptr);

QLabel *label(const QString &text, const QString &role = QString(), bool wrap = false);
QLabel *headline(const QString &text, bool wrap = false);
QLabel *valueLabel(bool leadingEdge = false);
QLabel *iconLabel(Theme::Glyph glyph, int size = 18, const QColor &color = QColor());
QLabel *badgeLabel(Theme::Glyph glyph, int size = 44);
QLabel *statusIcon(const QString &kind, int size = 18);
void setStatusIcon(QLabel *label, const QString &kind, int size = 18);

QPushButton *button(const QString &text, const QString &variant = QString());
QPushButton *button(const QString &text, Theme::Glyph glyph,
                    const QString &variant = QString());
QToolButton *iconButton(Theme::Glyph glyph, const QString &tooltip, int size = 18);
QComboBox *comboBox();
QAbstractButton *toggle(bool checked = false);

StatTile statTile(Theme::Glyph glyph, const QString &caption);
QFrame *divider();

void repolish(QWidget *widget);
void setVariant(QWidget *widget, const QString &variant);

}
