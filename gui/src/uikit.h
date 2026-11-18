// Orbiscreen - Qt GUI widget helpers (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#pragma once

#include <QFrame>
#include <QGridLayout>
#include <QLabel>
#include <QList>
#include <QScrollArea>
#include <QString>
#include <QToolButton>
#include <QWidget>

#include "theme.h"

class QAbstractButton;
class QButtonGroup;
class QComboBox;
class QHBoxLayout;
class QPushButton;
class QStackedWidget;
class QVBoxLayout;

namespace UiKit {

class GlyphWidget : public QWidget {
    Q_OBJECT

public:
    explicit GlyphWidget(Theme::Glyph glyph, int size = 20, const QColor &color = QColor(),
                         QWidget *parent = nullptr);

    void setGlyph(Theme::Glyph glyph);
    void setColor(const QColor &color);
    void setGlyphSize(int size);

    QSize sizeHint() const override;
    QSize minimumSizeHint() const override;

protected:
    void paintEvent(QPaintEvent *event) override;

private:
    QColor resolvedColor() const;

    Theme::Glyph m_glyph;
    QColor m_color;
    int m_size;
};

class LogoWidget : public QWidget {
    Q_OBJECT

public:
    explicit LogoWidget(int size = 44, QWidget *parent = nullptr);

    void setLogoSize(int size);

    QSize sizeHint() const override;
    QSize minimumSizeHint() const override;

protected:
    void paintEvent(QPaintEvent *event) override;

private:
    int m_size;
};

struct StatTile {
    QFrame *frame = nullptr;
    QLabel *caption = nullptr;
    QLabel *value = nullptr;
    GlyphWidget *icon = nullptr;
};

class Card : public QFrame {
    Q_OBJECT

public:
    Card(Theme::Glyph glyph, const QString &title, QWidget *parent = nullptr);

    void setTitle(const QString &title);
    void setBodySpacing(int spacing);
    QVBoxLayout *body() const { return m_body; }

private:
    GlyphWidget *m_icon = nullptr;
    QLabel *m_title = nullptr;
    QVBoxLayout *m_body = nullptr;
};

// A small circular step marker. Painted rather than styled with QSS: a QLabel with a
// border-radius background positions its text inside a padding box that Qt sizes from the font
// metrics, which leaves the digit visibly low and right of centre. Painting the disc and the text
// on the same rect centres them by construction.
class Badge : public QWidget {
    Q_OBJECT

public:
    explicit Badge(const QString &text, QWidget *parent = nullptr);

    void setText(const QString &text);

    QSize sizeHint() const override;
    QSize minimumSizeHint() const override { return sizeHint(); }

protected:
    void paintEvent(QPaintEvent *event) override;
    void changeEvent(QEvent *event) override;

private:
    QString m_text;
    int m_side;
};

class StatusPill : public QFrame {
    Q_OBJECT

public:
    explicit StatusPill(QWidget *parent = nullptr);

    void setStatus(const QString &state, const QString &text);

private:
    QLabel *m_dot = nullptr;
    QLabel *m_text = nullptr;
    QString m_state;
};

class PageScroll : public QScrollArea {
    Q_OBJECT

public:
    explicit PageScroll(QWidget *parent = nullptr);

    QVBoxLayout *contentLayout() const { return m_layout; }
    void setGutter(int gutter);
    void setMaxContentWidth(int width);

protected:
    bool eventFilter(QObject *watched, QEvent *event) override;

private:
    void updateColumnWidth();

    QWidget *m_content = nullptr;
    QVBoxLayout *m_layout = nullptr;
    int m_gutter;
    int m_maxWidth;
};

class ContentColumn : public QWidget {
    Q_OBJECT

public:
    explicit ContentColumn(QWidget *parent = nullptr);

    void setMaxContentWidth(int width);
    void setGutter(int gutter);

    QVBoxLayout *body() const { return m_layout; }

protected:
    void resizeEvent(QResizeEvent *event) override;

private:
    void updateWidth();

    QVBoxLayout *m_layout = nullptr;
    int m_maxWidth;
    int m_gutter;
};

class ResponsiveColumns : public QWidget {
    Q_OBJECT

public:
    explicit ResponsiveColumns(int columns = 2, QWidget *parent = nullptr);

    void addWidget(QWidget *widget, int weight = 1);
    int activeColumns() const { return m_activeColumns; }
    void setBreakpoint(int width);

signals:
    void columnsChanged(int columns);

protected:
    void resizeEvent(QResizeEvent *event) override;

private:
    void relayout(int columns);

    QGridLayout *m_grid = nullptr;
    QList<QWidget *> m_widgets;
    QList<int> m_weights;
    int m_maxColumns;
    int m_breakpoint;
    int m_activeColumns = 0;
    bool m_relayouting = false;
};

class TabButton : public QToolButton {
    Q_OBJECT

public:
    TabButton(const QString &text, Theme::Glyph glyph, QWidget *parent);

    void setLabel(const QString &text);
    void setGlyph(Theme::Glyph glyph);

    QSize sizeHint() const override;
    QSize minimumSizeHint() const override;

protected:
    void paintEvent(QPaintEvent *event) override;
    void enterEvent(QEnterEvent *event) override;
    void leaveEvent(QEvent *event) override;
    void changeEvent(QEvent *event) override;

private:
    Theme::Glyph m_glyph;
    QString m_label;
};

class NavTabs : public QWidget {
    Q_OBJECT

public:
    explicit NavTabs(QWidget *parent = nullptr);

    void addTab(const QString &text, Theme::Glyph glyph, QWidget *page);
    void setTabText(int index, const QString &text);
    void setTabGlyph(int index, Theme::Glyph glyph);

    int currentIndex() const;
    void setCurrentIndex(int index);
    int count() const;
    QStackedWidget *stack() const { return m_stack; }
    void refresh();

signals:
    void currentChanged(int index);

protected:
    void resizeEvent(QResizeEvent *event) override;

private:
    void updateRowMargins();

    QHBoxLayout *m_row = nullptr;
    QButtonGroup *m_group = nullptr;
    QList<TabButton *> m_buttons;
    QStackedWidget *m_stack = nullptr;
    int m_current = 0;
};

Card *card(Theme::Glyph glyph, const QString &title, QWidget *parent = nullptr);

QLabel *label(const QString &text, const QString &role = QString(), bool wrap = false);
QLabel *caption(const QString &text, bool wrap = false);
QLabel *headline(const QString &text, bool wrap = false);
QLabel *valueLabel(bool leadingEdge = false);

QPushButton *button(const QString &text, const QString &variant = QString());
QPushButton *button(const QString &text, Theme::Glyph glyph,
                    const QString &variant = QString());
QToolButton *iconButton(Theme::Glyph glyph, const QString &tooltip, int size = 18);
QComboBox *comboBox();
QAbstractButton *toggle(bool checked = false);

StatTile statTile(const QString &caption, Theme::Glyph glyph = Theme::Glyph::Count);
Badge *stepBadge(const QString &text, QWidget *parent = nullptr);
QFrame *divider();
QFrame *separator(bool vertical = false);

void repolish(QWidget *widget);
void setVariant(QWidget *widget, const QString &variant);

}
