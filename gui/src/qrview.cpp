// Orbiscreen - Qt crisp QR renderer (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#include "qrview.h"

#include "theme.h"

#include <QPainter>
#include <QSizePolicy>

QrView::QrView(QWidget *parent)
    : QWidget(parent)
{
    setSizePolicy(QSizePolicy::Fixed, QSizePolicy::Fixed);
}

void QrView::setMatrix(const QImage &matrix)
{
    m_matrix = matrix;
    update();
}

void QrView::clear()
{
    m_matrix = QImage();
    update();
}

QSize QrView::sizeHint() const
{
    return QSize(210, 210);
}

QSize QrView::minimumSizeHint() const
{
    return QSize(150, 150);
}

void QrView::paintEvent(QPaintEvent *)
{
    const Theme::Colors &c = Theme::colors();
    QPainter painter(this);
    painter.setRenderHint(QPainter::Antialiasing, true);
    const QRectF card = QRectF(rect()).adjusted(0.5, 0.5, -0.5, -0.5);
    painter.setPen(QPen(c.border, 1.0));
    painter.setBrush(QColor(QStringLiteral("#FFFFFF")));
    painter.drawRoundedRect(card, 14.0, 14.0);

    if (m_matrix.isNull() || m_matrix.width() <= 0) {
        painter.setPen(QPen(c.muted, 1.4, Qt::SolidLine, Qt::RoundCap, Qt::RoundJoin));
        painter.drawEllipse(QRectF(card.center().x() - 12, card.center().y() - 12, 24, 24));
        return;
    }

    const int modules = m_matrix.width();
    const int quiet = 4;
    const int padded = modules + quiet * 2;
    const qreal inner = qMin(card.width(), card.height()) - 24.0;
    const qreal scale = inner / padded;
    const QPointF origin(card.center().x() - (padded * scale) / 2.0 + quiet * scale,
                         card.center().y() - (padded * scale) / 2.0 + quiet * scale);

    painter.setRenderHint(QPainter::Antialiasing, false);
    painter.setPen(Qt::NoPen);
    painter.setBrush(c.qrDark);
    for (int y = 0; y < modules; ++y) {
        const uchar *row = m_matrix.constScanLine(y);
        int run = -1;
        for (int x = 0; x <= modules; ++x) {
            const bool dark = x < modules && row[x] < 128;
            if (dark && run < 0) {
                run = x;
            } else if (!dark && run >= 0) {
                painter.drawRect(QRectF(origin.x() + run * scale, origin.y() + y * scale,
                                        (x - run) * scale, scale));
                run = -1;
            }
        }
    }
}
