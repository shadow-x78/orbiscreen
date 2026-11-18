// Orbiscreen - Qt GUI widget helpers (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#include "uikit.h"

#include <QAbstractButton>
#include <QButtonGroup>
#include <QComboBox>
#include <QEnterEvent>
#include <QHBoxLayout>
#include <QPainter>
#include <QPushButton>
#include <QResizeEvent>
#include <QScrollArea>
#include <QScrollBar>
#include <QStackedWidget>
#include <QStyle>
#include <QToolButton>
#include <QVBoxLayout>

namespace {

class ChevronComboBox : public QComboBox {
public:
    explicit ChevronComboBox(QWidget *parent = nullptr)
        : QComboBox(parent)
    {
    }

protected:
    void paintEvent(QPaintEvent *event) override
    {
        QComboBox::paintEvent(event);
        QPainter painter(this);
        painter.setRenderHint(QPainter::Antialiasing, true);
        const Theme::Colors &c = Theme::colors();
        const int side = 16;
        const bool rtl = layoutDirection() == Qt::RightToLeft;
        const qreal x = rtl ? 12.0 : width() - 12.0 - side;
        Theme::paintGlyph(&painter, Theme::Glyph::Chevron,
                          QRectF(x, (height() - side) / 2.0, side, side),
                          isEnabled() ? c.muted : c.borderStrong);
    }
};

class Switch : public QAbstractButton {
public:
    explicit Switch(QWidget *parent = nullptr)
        : QAbstractButton(parent)
    {
        setCheckable(true);
        setCursor(Qt::PointingHandCursor);
        setFocusPolicy(Qt::StrongFocus);
        setSizePolicy(QSizePolicy::Fixed, QSizePolicy::Fixed);
    }

    QSize sizeHint() const override { return QSize(44, 30); }
    QSize minimumSizeHint() const override { return QSize(44, 30); }

protected:
    void paintEvent(QPaintEvent *) override
    {
        const Theme::Colors &c = Theme::colors();
        QPainter painter(this);
        painter.setRenderHint(QPainter::Antialiasing, true);
        const qreal trackWidth = 42.0;
        const qreal trackHeight = 24.0;
        QRectF track(0.5, (height() - trackHeight) / 2.0 + 0.5, trackWidth, trackHeight);
        if (layoutDirection() == Qt::RightToLeft) {
            track.moveLeft(width() - trackWidth - 0.5);
        }
        const bool on = isChecked();
        QColor fill = on ? c.accent : c.surfaceAlt;
        QColor border = on ? c.accent : c.borderStrong;
        if (!isEnabled()) {
            fill = c.surfaceAlt;
            border = c.border;
        }
        painter.setPen(QPen(border, 1.0));
        painter.setBrush(fill);
        painter.drawRoundedRect(track, trackHeight / 2.0, trackHeight / 2.0);
        const qreal knob = 18.0;
        const qreal margin = 3.0;
        const qreal knobX = on ? track.right() - margin - knob : track.left() + margin;
        painter.setPen(Qt::NoPen);
        painter.setBrush(on ? QColor(QStringLiteral("#FFFFFF"))
                            : (isEnabled() ? c.muted : c.borderStrong));
        painter.drawEllipse(QRectF(knobX, track.center().y() - knob / 2.0, knob, knob));
        if (hasFocus()) {
            painter.setPen(QPen(c.accent, 1.0));
            painter.setBrush(Qt::NoBrush);
            painter.drawRoundedRect(track.adjusted(-3.0, -3.0, 3.0, 3.0), 15.0, 15.0);
        }
    }
};

// QGridLayout (used by ResponsiveColumns) does not propagate height-for-width, so a wrapped
// paragraph inside a card keeps a one-line height and gets clipped. A label that pins its own
// wrapped height as its minimum fixes every parent layout at once.
class WrappedLabel : public QLabel {
public:
    using QLabel::QLabel;

protected:
    void resizeEvent(QResizeEvent *event) override
    {
        QLabel::resizeEvent(event);
        updateMinimumHeight();
    }

    void changeEvent(QEvent *event) override
    {
        QLabel::changeEvent(event);
        if (event->type() == QEvent::FontChange || event->type() == QEvent::StyleChange
            || event->type() == QEvent::LayoutDirectionChange) {
            updateMinimumHeight();
        }
    }

private:
    void updateMinimumHeight()
    {
        const int needed = heightForWidth(qMax(1, width()));
        if (needed > 0 && minimumHeight() != needed) {
            setMinimumHeight(needed);
        }
    }
};

QLabel *makeLabel(const QString &text, Theme::FontRole role, const QString &roleName,
                  bool wrap)
{
    QLabel *result = wrap ? new WrappedLabel(text) : new QLabel(text);
    result->setFont(Theme::font(role));
    if (!roleName.isEmpty()) {
        result->setObjectName(roleName);
    }
    result->setWordWrap(wrap);
    result->setTextInteractionFlags(Qt::TextSelectableByMouse);
    if (wrap) {
        result->setSizePolicy(QSizePolicy::Preferred, QSizePolicy::Minimum);
    }
    return result;
}

}

namespace UiKit {

GlyphWidget::GlyphWidget(Theme::Glyph glyph, int size, const QColor &color, QWidget *parent)
    : QWidget(parent)
    , m_glyph(glyph)
    , m_color(color)
    , m_size(size)
{
    setAttribute(Qt::WA_TransparentForMouseEvents, true);
    setSizePolicy(QSizePolicy::Fixed, QSizePolicy::Fixed);
    setFocusPolicy(Qt::NoFocus);
}

void GlyphWidget::setGlyph(Theme::Glyph glyph)
{
    if (m_glyph == glyph) {
        return;
    }
    m_glyph = glyph;
    update();
}

void GlyphWidget::setColor(const QColor &color)
{
    m_color = color;
    update();
}

void GlyphWidget::setGlyphSize(int size)
{
    m_size = size;
    updateGeometry();
    update();
}

QSize GlyphWidget::sizeHint() const
{
    return QSize(m_size, m_size);
}

QSize GlyphWidget::minimumSizeHint() const
{
    return QSize(m_size, m_size);
}

void GlyphWidget::paintEvent(QPaintEvent *)
{
    QPainter painter(this);
    painter.setRenderHint(QPainter::Antialiasing, true);
    const qreal side = qMin(width(), height());
    const QRectF box((width() - side) / 2.0, (height() - side) / 2.0, side, side);
    Theme::paintGlyph(&painter, m_glyph, box, resolvedColor());
}

QColor GlyphWidget::resolvedColor() const
{
    if (m_color.isValid()) {
        return m_color;
    }
    return isEnabled() ? Theme::colors().muted : Theme::colors().borderStrong;
}

LogoWidget::LogoWidget(int size, QWidget *parent)
    : QWidget(parent)
    , m_size(size)
{
    setAttribute(Qt::WA_TransparentForMouseEvents, true);
    setSizePolicy(QSizePolicy::Fixed, QSizePolicy::Fixed);
}

void LogoWidget::setLogoSize(int size)
{
    m_size = size;
    updateGeometry();
    update();
}

QSize LogoWidget::sizeHint() const
{
    return QSize(m_size, m_size);
}

QSize LogoWidget::minimumSizeHint() const
{
    return QSize(m_size, m_size);
}

void LogoWidget::paintEvent(QPaintEvent *)
{
    QPainter painter(this);
    painter.setRenderHint(QPainter::Antialiasing, true);
    const qreal side = qMin(width(), height());
    const QRectF box((width() - side) / 2.0, (height() - side) / 2.0, side, side);
    Theme::paintLogo(&painter, box, Theme::colors().brand, m_size <= 28);
}

Card::Card(Theme::Glyph glyph, const QString &title, QWidget *parent)
    : QFrame(parent)
{
    setObjectName(QStringLiteral("Card"));
    auto *root = new QVBoxLayout(this);
    root->setContentsMargins(Theme::Spacing::xl, Theme::Spacing::lg, Theme::Spacing::xl,
                             Theme::Spacing::xl);
    root->setSpacing(Theme::Spacing::lg);

    auto *header = new QHBoxLayout;
    header->setContentsMargins(0, 0, 0, 0);
    header->setSpacing(Theme::Spacing::sm);
    m_icon = new GlyphWidget(glyph, Theme::Metrics::cardIconSize, Theme::colors().muted,
                             this);
    header->addWidget(m_icon, 0, Qt::AlignVCenter);
    m_title = makeLabel(title, Theme::FontRole::SectionTitle, QStringLiteral("CardTitle"),
                        false);
    header->addWidget(m_title, 1, Qt::AlignVCenter);
    root->addLayout(header);

    m_body = new QVBoxLayout;
    m_body->setContentsMargins(0, 0, 0, 0);
    m_body->setSpacing(Theme::Spacing::md);
    root->addLayout(m_body);
    // Cards in a row share the tallest height; keep their content packed at the top.
    root->addStretch(1);
}

void Card::setTitle(const QString &title)
{
    m_title->setText(title);
}

void Card::setBodySpacing(int spacing)
{
    m_body->setSpacing(spacing);
}

StatusPill::StatusPill(QWidget *parent)
    : QFrame(parent)
{
    setObjectName(QStringLiteral("StatusPill"));
    setProperty("state", QStringLiteral("stopped"));
    setSizePolicy(QSizePolicy::Fixed, QSizePolicy::Fixed);
    auto *row = new QHBoxLayout(this);
    row->setContentsMargins(Theme::Spacing::md, 0, Theme::Spacing::md, 0);
    row->setSpacing(Theme::Spacing::sm);
    m_dot = new QLabel(this);
    m_dot->setFixedSize(8, 8);
    row->addWidget(m_dot, 0, Qt::AlignVCenter);
    m_text = makeLabel(QString(), Theme::FontRole::Caption, QStringLiteral("StatusText"),
                       false);
    row->addWidget(m_text, 0, Qt::AlignVCenter);
    setFixedHeight(Theme::Metrics::controlHeight);
    setStatus(QStringLiteral("stopped"), QString());
}

void StatusPill::setStatus(const QString &state, const QString &text)
{
    m_state = state;
    m_text->setText(text);
    const Theme::Colors &c = Theme::colors();
    QColor dot = c.danger;
    if (state == QStringLiteral("ready")) {
        dot = c.success;
    } else if (state == QStringLiteral("streaming")) {
        dot = c.accent;
    }
    m_dot->setStyleSheet(QStringLiteral("background:%1;border-radius:4px;").arg(dot.name()));
    setProperty("state", state);
    // The colour rules are scoped to the pill's state property, so the child labels have to be
    // polished again; repolishing only the frame left them with the previous state's colour.
    repolish(this);
    repolish(m_text);
    repolish(m_dot);
    adjustSize();
}

PageScroll::PageScroll(QWidget *parent)
    : QScrollArea(parent)
    , m_gutter(Theme::Metrics::pageGutter)
    , m_maxWidth(Theme::Metrics::contentMaxWidth)
{
    setWidgetResizable(true);
    setFrameShape(QFrame::NoFrame);
    setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
    setVerticalScrollBarPolicy(Qt::ScrollBarAsNeeded);
    setObjectName(QStringLiteral("PageSurface"));
    viewport()->setAutoFillBackground(false);

    m_content = new QWidget(this);
    m_content->setObjectName(QStringLiteral("PageSurface"));
    m_content->setAutoFillBackground(false);
    m_layout = new QVBoxLayout(m_content);
    m_layout->setContentsMargins(m_gutter, Theme::Spacing::sm, m_gutter, Theme::Spacing::xxl);
    m_layout->setSpacing(Theme::Metrics::cardGap);
    m_layout->setSizeConstraint(QLayout::SetDefaultConstraint);
    m_content->installEventFilter(this);
    setWidget(m_content);
}

void PageScroll::setGutter(int gutter)
{
    m_gutter = qMax(0, gutter);
    updateColumnWidth();
}

void PageScroll::setMaxContentWidth(int width)
{
    m_maxWidth = width;
    updateColumnWidth();
}

bool PageScroll::eventFilter(QObject *watched, QEvent *event)
{
    if (watched == m_content && event->type() == QEvent::Resize) {
        updateColumnWidth();
    }
    return QScrollArea::eventFilter(watched, event);
}

void PageScroll::updateColumnWidth()
{
    const int side = qMax(m_gutter, (m_content->width() - m_maxWidth) / 2);
    const QMargins wanted(side, Theme::Spacing::sm, side, Theme::Spacing::xxl);
    if (m_layout->contentsMargins() != wanted) {
        m_layout->setContentsMargins(wanted);
    }
    // A scroll area pins its widget to the viewport height, so the layout would compress cards
    // below their minimum. The minimum height also has to follow the wrapped text, which only
    // becomes known once the width is.
    m_layout->invalidate();
    const int needed = m_layout->minimumSize().height();
    if (needed > 0 && m_content->minimumHeight() != needed) {
        m_content->setMinimumHeight(needed);
    }
}

ContentColumn::ContentColumn(QWidget *parent)
    : QWidget(parent)
    , m_maxWidth(Theme::Metrics::contentMaxWidth)
    , m_gutter(Theme::Metrics::pageGutter)
{
    m_layout = new QVBoxLayout(this);
    m_layout->setContentsMargins(m_gutter, 0, m_gutter, 0);
    m_layout->setSpacing(0);
}

void ContentColumn::setMaxContentWidth(int width)
{
    m_maxWidth = width;
    updateWidth();
}

void ContentColumn::setGutter(int gutter)
{
    m_gutter = qMax(0, gutter);
    updateWidth();
}

void ContentColumn::resizeEvent(QResizeEvent *event)
{
    QWidget::resizeEvent(event);
    updateWidth();
}

void ContentColumn::updateWidth()
{
    const int side = qMax(m_gutter, (width() - m_maxWidth) / 2);
    m_layout->setContentsMargins(side, 0, side, 0);
}

ResponsiveColumns::ResponsiveColumns(int columns, QWidget *parent)
    : QWidget(parent)
    , m_maxColumns(qMax(1, columns))
    , m_breakpoint(Theme::Metrics::breakpointTwoColumns)
{
    m_grid = new QGridLayout(this);
    m_grid->setContentsMargins(0, 0, 0, 0);
    m_grid->setHorizontalSpacing(Theme::Metrics::cardGap);
    m_grid->setVerticalSpacing(Theme::Metrics::cardGap);
}

void ResponsiveColumns::addWidget(QWidget *widget, int weight)
{
    m_widgets.append(widget);
    m_weights.append(qMax(1, weight));
    relayout(m_activeColumns > 0 ? m_activeColumns : m_maxColumns);
}

void ResponsiveColumns::setBreakpoint(int width)
{
    m_breakpoint = width;
    relayout(m_activeColumns > 0 ? m_activeColumns : m_maxColumns);
}

void ResponsiveColumns::resizeEvent(QResizeEvent *event)
{
    QWidget::resizeEvent(event);
    const int wanted = width() >= m_breakpoint && m_maxColumns > 1 ? m_maxColumns : 1;
    if (wanted != m_activeColumns) {
        relayout(wanted);
    }
}

void ResponsiveColumns::relayout(int columns)
{
    if (m_relayouting) {
        return;
    }
    m_relayouting = true;
    columns = qBound(1, columns, m_maxColumns);
    const bool changed = columns != m_activeColumns;
    m_activeColumns = columns;
    while (QLayoutItem *item = m_grid->takeAt(0)) {
        delete item;
    }
    for (int index = 0; index < m_widgets.size(); ++index) {
        m_grid->addWidget(m_widgets.at(index), index / columns, index % columns);
    }
    for (int column = 0; column < columns; ++column) {
        int sum = 0;
        for (int index = column; index < m_widgets.size(); index += columns) {
            sum += m_weights.at(index);
        }
        m_grid->setColumnStretch(column, sum > 0 ? sum : 1);
    }
    for (int column = columns; column < m_grid->columnCount(); ++column) {
        m_grid->setColumnStretch(column, 0);
    }
    m_grid->invalidate();
    m_relayouting = false;
    if (changed) {
        emit columnsChanged(columns);
    }
}

TabButton::TabButton(const QString &text, Theme::Glyph glyph, QWidget *parent)
    : QToolButton(parent)
    , m_glyph(glyph)
    , m_label(text)
{
    setObjectName(QStringLiteral("NavTab"));
    setCheckable(true);
    setCursor(Qt::PointingHandCursor);
    setFixedHeight(Theme::Metrics::controlHeight);
    setSizePolicy(QSizePolicy::Fixed, QSizePolicy::Fixed);
    setFocusPolicy(Qt::StrongFocus);
    setToolTip(text);
    setAccessibleName(text);
}

void TabButton::setLabel(const QString &text)
{
    if (m_label == text) {
        return;
    }
    m_label = text;
    setToolTip(text);
    setAccessibleName(text);
    updateGeometry();
    update();
}

void TabButton::setGlyph(Theme::Glyph glyph)
{
    m_glyph = glyph;
    update();
}

QSize TabButton::sizeHint() const
{
    const QFontMetrics metrics(Theme::font(Theme::FontRole::Label));
    const int textWidth = metrics.horizontalAdvance(m_label);
    // Matches the padding declared for QToolButton#NavTab: padding + icon + gap + text + padding.
    return QSize(2 * Theme::Metrics::tabPadding + Theme::Metrics::tabIconSize + Theme::Spacing::sm
                     + textWidth,
                 Theme::Metrics::controlHeight);
}

QSize TabButton::minimumSizeHint() const
{
    return sizeHint();
}

void TabButton::paintEvent(QPaintEvent *)
{
    const Theme::Colors &c = Theme::colors();
    QPainter painter(this);
    painter.setRenderHint(QPainter::Antialiasing, true);
    const bool on = isChecked();
    const bool rtl = layoutDirection() == Qt::RightToLeft;
    const QRectF box = QRectF(rect()).adjusted(0.5, 0.5, -0.5, -0.5);

    QColor background = QColor(0, 0, 0, 0);
    QColor border = QColor(0, 0, 0, 0);
    if (on) {
        background = c.accent;
        background.setAlpha(c.dark ? 46 : 34);
        border = c.accent;
        border.setAlpha(c.dark ? 96 : 78);
    } else if (underMouse()) {
        background = c.surfaceAlt;
        border = c.border;
    }
    painter.setPen(border.alpha() == 0 ? QPen(Qt::NoPen) : QPen(border, 1.0));
    painter.setBrush(background);
    painter.drawRoundedRect(box, Theme::Radius::control, Theme::Radius::control);

    const QColor content = on ? c.accent : (underMouse() ? c.text : c.muted);
    const QFontMetrics metrics(Theme::font(Theme::FontRole::Label));
    const int iconSide = Theme::Metrics::tabIconSize;
    const int gap = Theme::Spacing::sm;
    const int textWidth = metrics.horizontalAdvance(m_label);
    const qreal iconX = rtl ? width() - Theme::Metrics::tabPadding - iconSide
                            : Theme::Metrics::tabPadding;
    const qreal iconY = (height() - iconSide) / 2.0;
    Theme::paintGlyph(&painter, m_glyph, QRectF(iconX, iconY, iconSide, iconSide), content);
    const qreal textX = rtl ? iconX - gap - textWidth : iconX + iconSide + gap;
    painter.setFont(Theme::font(Theme::FontRole::Label));
    painter.setPen(content);
    painter.drawText(QRectF(textX, 0, textWidth + 2, height()),
                     Qt::AlignVCenter | Qt::AlignLeading, m_label);

    if (hasFocus()) {
        painter.setPen(QPen(c.accent, 1.0));
        painter.setBrush(Qt::NoBrush);
        painter.drawRoundedRect(box.adjusted(1.0, 1.0, -1.0, -1.0), Theme::Radius::control - 1.0,
                                Theme::Radius::control - 1.0);
    }
}

void TabButton::enterEvent(QEnterEvent *event)
{
    QToolButton::enterEvent(event);
    update();
}

void TabButton::leaveEvent(QEvent *event)
{
    QToolButton::leaveEvent(event);
    update();
}

void TabButton::changeEvent(QEvent *event)
{
    QToolButton::changeEvent(event);
    if (event->type() == QEvent::LayoutDirectionChange
        || event->type() == QEvent::EnabledChange) {
        updateGeometry();
        update();
    }
}

NavTabs::NavTabs(QWidget *parent)
    : QWidget(parent)
{
    setObjectName(QStringLiteral("NavBar"));
    auto *root = new QVBoxLayout(this);
    root->setContentsMargins(0, 0, 0, 0);
    root->setSpacing(Theme::Spacing::lg);

    auto *rowHost = new QWidget(this);
    rowHost->setObjectName(QStringLiteral("NavBar"));
    m_row = new QHBoxLayout(rowHost);
    m_row->setContentsMargins(0, 0, 0, 0);
    m_row->setSpacing(Theme::Spacing::xs);
    m_group = new QButtonGroup(this);
    m_group->setExclusive(true);
    m_row->addStretch(1);
    root->addWidget(rowHost);

    m_stack = new QStackedWidget(this);
    m_stack->setObjectName(QStringLiteral("PageSurface"));
    root->addWidget(m_stack, 1);

    connect(m_group, &QButtonGroup::idClicked, this, [this](int id) {
        setCurrentIndex(id);
    });
}

void NavTabs::resizeEvent(QResizeEvent *event)
{
    QWidget::resizeEvent(event);
    updateRowMargins();
}

void NavTabs::updateRowMargins()
{
    const int side = qMax(Theme::Metrics::pageGutter,
                          (width() - Theme::Metrics::contentMaxWidth) / 2);
    m_row->setContentsMargins(side, 0, side, 0);
}

void NavTabs::addTab(const QString &text, Theme::Glyph glyph, QWidget *page)
{
    auto *tab = new TabButton(text, glyph, this);
    const int index = m_buttons.size();
    m_buttons.append(tab);
    m_group->addButton(tab, index);
    m_row->insertWidget(m_row->count() - 1, tab, 0);
    m_stack->addWidget(page);
    tab->setChecked(index == 0);
}

void NavTabs::setTabText(int index, const QString &text)
{
    if (index < 0 || index >= m_buttons.size()) {
        return;
    }
    m_buttons[index]->setLabel(text);
}

void NavTabs::setTabGlyph(int index, Theme::Glyph glyph)
{
    if (index < 0 || index >= m_buttons.size()) {
        return;
    }
    m_buttons[index]->setGlyph(glyph);
}

int NavTabs::currentIndex() const
{
    return m_current;
}

void NavTabs::setCurrentIndex(int index)
{
    if (index < 0 || index >= m_buttons.size()) {
        return;
    }
    if (m_current == index && m_buttons[index]->isChecked()) {
        return;
    }
    m_current = index;
    m_buttons[index]->setChecked(true);
    m_stack->setCurrentIndex(index);
    refresh();
    emit currentChanged(index);
}

int NavTabs::count() const
{
    return m_buttons.size();
}

void NavTabs::refresh()
{
    for (int index = 0; index < m_buttons.size(); ++index) {
        m_buttons[index]->setChecked(index == m_current);
        m_buttons[index]->update();
    }
}

Card *card(Theme::Glyph glyph, const QString &title, QWidget *parent)
{
    return new Card(glyph, title, parent);
}

QLabel *label(const QString &text, const QString &role, bool wrap)
{
    Theme::FontRole fontRole = Theme::FontRole::Body;
    QString roleName;
    if (role == QStringLiteral("Muted")) {
        roleName = QStringLiteral("Muted");
    } else if (role == QStringLiteral("Value")) {
        roleName = QStringLiteral("Value");
        fontRole = Theme::FontRole::Label;
    } else if (role == QStringLiteral("CardTitle")) {
        roleName = QStringLiteral("CardTitle");
        fontRole = Theme::FontRole::SectionTitle;
    } else if (role == QStringLiteral("Caption")) {
        roleName = QStringLiteral("Caption");
        fontRole = Theme::FontRole::Caption;
    }
    QLabel *result = makeLabel(text, fontRole, roleName, wrap);
    result->setAlignment(Qt::AlignLeading | Qt::AlignVCenter);
    return result;
}

QLabel *caption(const QString &text, bool wrap)
{
    return makeLabel(text, Theme::FontRole::Caption, QStringLiteral("Caption"), wrap);
}

QLabel *headline(const QString &text, bool wrap)
{
    QLabel *result = makeLabel(text, Theme::FontRole::Headline, QString(), wrap);
    result->setAlignment(Qt::AlignLeading | Qt::AlignVCenter);
    return result;
}

QLabel *valueLabel(bool leadingEdge)
{
    QLabel *result = makeLabel(QString(), Theme::FontRole::Label, QStringLiteral("Value"),
                               false);
    result->setFont(Theme::tabular(result->font()));
    result->setAlignment((leadingEdge ? Qt::AlignLeading : Qt::AlignTrailing)
                         | Qt::AlignVCenter);
    return result;
}

QPushButton *button(const QString &text, const QString &variant)
{
    auto *result = new QPushButton(text);
    result->setCursor(Qt::PointingHandCursor);
    result->setMinimumHeight(Theme::Metrics::controlHeight);
    result->setSizePolicy(QSizePolicy::Fixed, QSizePolicy::Fixed);
    if (!variant.isEmpty()) {
        setVariant(result, variant);
    }
    return result;
}

QPushButton *button(const QString &text, Theme::Glyph glyph, const QString &variant)
{
    QPushButton *result = button(text, variant);
    result->setIcon(Theme::glyph(glyph));
    result->setIconSize(QSize(16, 16));
    return result;
}

QToolButton *iconButton(Theme::Glyph glyph, const QString &tooltip, int size)
{
    auto *result = new QToolButton;
    result->setObjectName(QStringLiteral("IconButton"));
    result->setCursor(Qt::PointingHandCursor);
    result->setIcon(Theme::glyph(glyph));
    result->setIconSize(QSize(size, size));
    result->setToolTip(tooltip);
    result->setFixedSize(Theme::Metrics::iconButtonSize, Theme::Metrics::iconButtonSize);
    result->setSizePolicy(QSizePolicy::Fixed, QSizePolicy::Fixed);
    return result;
}

QComboBox *comboBox()
{
    auto *result = new ChevronComboBox;
    result->setCursor(Qt::PointingHandCursor);
    result->setMinimumHeight(Theme::Metrics::controlHeight);
    result->setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Fixed);
    return result;
}

QAbstractButton *toggle(bool checked)
{
    auto *result = new Switch;
    result->setChecked(checked);
    return result;
}

Badge::Badge(const QString &text, QWidget *parent)
    : QWidget(parent)
    , m_text(text)
    , m_side(Theme::Metrics::badgeSize)
{
    setSizePolicy(QSizePolicy::Fixed, QSizePolicy::Fixed);
    setAttribute(Qt::WA_TransparentForMouseEvents);
}

void Badge::setText(const QString &text)
{
    if (m_text == text) {
        return;
    }
    m_text = text;
    update();
}

QSize Badge::sizeHint() const
{
    return QSize(m_side, m_side);
}

void Badge::changeEvent(QEvent *event)
{
    QWidget::changeEvent(event);
    if (event->type() == QEvent::FontChange || event->type() == QEvent::StyleChange) {
        update();
    }
}

void Badge::paintEvent(QPaintEvent *)
{
    const Theme::Colors &c = Theme::colors();
    QPainter painter(this);
    painter.setRenderHint(QPainter::Antialiasing, true);

    QColor disc = c.accent;
    disc.setAlpha(c.dark ? 46 : 34);
    painter.setPen(Qt::NoPen);
    painter.setBrush(disc);
    painter.drawEllipse(QRectF(rect()));

    QFont font = Theme::font(Theme::FontRole::Caption);
    font.setBold(true);
    painter.setFont(font);
    painter.setPen(c.accent);
    // Draw the glyph on the widget rect with AlignCenter: the same rect the disc used, so the
    // number is centred on the circle rather than on a font-metric-derived padding box.
    painter.drawText(rect(), Qt::AlignCenter, m_text);
}

Badge *stepBadge(const QString &text, QWidget *parent)
{
    return new Badge(text, parent);
}

StatTile statTile(const QString &captionText, Theme::Glyph glyph)
{
    StatTile tile;
    tile.frame = new QFrame;
    tile.frame->setObjectName(QStringLiteral("StatTile"));
    tile.frame->setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Fixed);
    auto *layout = new QVBoxLayout(tile.frame);
    layout->setContentsMargins(Theme::Spacing::md, Theme::Spacing::sm + 2, Theme::Spacing::md,
                               Theme::Spacing::sm + 2);
    layout->setSpacing(Theme::Spacing::xs - 1);

    // A glyph on the trailing edge marks what the tile measures, so a row of tiles is scannable
    // without reading every caption. It is decorative, so it is muted and ignores the mouse.
    auto *row = new QHBoxLayout;
    row->setContentsMargins(0, 0, 0, 0);
    row->setSpacing(Theme::Spacing::sm);
    auto *text = new QVBoxLayout;
    text->setContentsMargins(0, 0, 0, 0);
    text->setSpacing(Theme::Spacing::xs - 1);

    tile.caption = makeLabel(captionText, Theme::FontRole::Caption, QStringLiteral("Caption"),
                             false);
    tile.value = makeLabel(QStringLiteral("-"), Theme::FontRole::Label,
                           QStringLiteral("Value"), false);
    tile.value->setFont(Theme::tabular(tile.value->font()));
    tile.value->setAlignment(Qt::AlignLeading | Qt::AlignVCenter);
    text->addWidget(tile.caption);
    text->addWidget(tile.value);
    row->addLayout(text, 1);

    if (glyph != Theme::Glyph::Count) {
        tile.icon = new GlyphWidget(glyph, 16, Theme::colors().muted, tile.frame);
        row->addWidget(tile.icon, 0, Qt::AlignVCenter);
    }
    layout->addLayout(row);
    // A tile is a fixed-height strip: inside a scroll area a word-wrapped sibling makes the
    // layout's height-for-width estimate drift, which used to squeeze these rows. Polish first so
    // the fonts (and therefore the line heights) are final before the height is measured.
    tile.caption->ensurePolished();
    tile.value->ensurePolished();
    const int height = layout->contentsMargins().top() + layout->contentsMargins().bottom()
        + tile.caption->sizeHint().height() + layout->spacing() + tile.value->sizeHint().height();
    tile.frame->setFixedHeight(qMax(height, layout->sizeHint().height()));
    return tile;
}

QFrame *divider()
{
    auto *line = new QFrame;
    line->setObjectName(QStringLiteral("Divider"));
    line->setFixedHeight(1);
    line->setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Fixed);
    return line;
}

QFrame *separator(bool vertical)
{
    auto *line = new QFrame;
    line->setObjectName(QStringLiteral("Divider"));
    if (vertical) {
        line->setFixedSize(1, 14);
        line->setSizePolicy(QSizePolicy::Fixed, QSizePolicy::Fixed);
    } else {
        line->setFixedHeight(1);
        line->setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Fixed);
    }
    return line;
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
    widget->setProperty("variant", variant);
    repolish(widget);
}

}
