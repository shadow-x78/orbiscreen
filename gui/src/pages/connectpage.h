// Orbiscreen - Qt Connect page (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#pragma once

#include <QImage>
#include <QWidget>

#include "report.h"
#include "uikit.h"

class QLabel;
class QLineEdit;
class QPushButton;
class QToolButton;
class QrView;

class ConnectPage : public QWidget {
    Q_OBJECT

public:
    explicit ConnectPage(QWidget *parent = nullptr);

    void update(const DaemonStatus &status, const QImage &qr, const QString &url);
    void setQr(const QImage &qr);
    void retranslate();

signals:
    void openRequested(const QString &url);
    void openUsbRequested();

private:
    UiKit::Card *m_statusCard = nullptr;
    UiKit::Card *m_wifiCard = nullptr;
    UiKit::Card *m_usbCard = nullptr;
    UiKit::Card *m_guideCard = nullptr;

    QLabel *m_headline = nullptr;
    QLabel *m_subtitle = nullptr;
    UiKit::StatTile m_resolution;
    UiKit::StatTile m_encoder;
    UiKit::StatTile m_devices;

    QrView *m_qr = nullptr;
    QLabel *m_wifiPrompt = nullptr;
    QLineEdit *m_url = nullptr;
    QToolButton *m_copy = nullptr;
    QPushButton *m_open = nullptr;

    QLabel *m_usbState = nullptr;
    QLabel *m_usbDesc = nullptr;
    QLabel *m_usbDevices = nullptr;
    QPushButton *m_openUsb = nullptr;

    QLabel *m_guide[3] = {nullptr, nullptr, nullptr};
};
