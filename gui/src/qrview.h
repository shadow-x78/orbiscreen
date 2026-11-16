// Orbiscreen - Qt crisp QR renderer (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#pragma once

#include <QImage>
#include <QSize>
#include <QWidget>

class QrView : public QWidget {
    Q_OBJECT

public:
    explicit QrView(QWidget *parent = nullptr);

    void setMatrix(const QImage &matrix);
    void clear();

    QSize sizeHint() const override;
    QSize minimumSizeHint() const override;

protected:
    void paintEvent(QPaintEvent *event) override;

private:
    QImage m_matrix;
};
