// Orbiscreen - Qt Connect page (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#pragma once

#include <QImage>
#include <QWidget>

#include "report.h"

class QLabel;
class QLineEdit;
class QPushButton;
class QGroupBox;

class ConnectPage : public QWidget {
    Q_OBJECT

public:
    explicit ConnectPage(QWidget *parent = nullptr);

    void update(const DaemonStatus &status, const QImage &qr, const QString &url);
    void setQr(const QImage &qr);
    void retranslate();

signals:
    void startStopRequested(bool running);
    void openRequested(const QString &url);
    void openUsbRequested();

private:
    QGroupBox *m_statusGroup = nullptr;
    QGroupBox *m_wifiGroup = nullptr;
    QGroupBox *m_usbGroup = nullptr;
    QGroupBox *m_guideGroup = nullptr;

    QLabel *m_headline = nullptr;
    QLabel *m_subtitle = nullptr;
    QLabel *m_state = nullptr;
    QLabel *m_resolutionLabel = nullptr;
    QLabel *m_encoderLabel = nullptr;
    QLabel *m_devicesLabel = nullptr;
    QLabel *m_resolution = nullptr;
    QLabel *m_encoder = nullptr;
    QLabel *m_clients = nullptr;
    QLabel *m_qr = nullptr;
    QLabel *m_wifiPrompt = nullptr;
    QLabel *m_usbState = nullptr;
    QLabel *m_usbDesc = nullptr;
    QLabel *m_usbDevices = nullptr;
    QLabel *m_guide[3] = {nullptr, nullptr, nullptr};
    QLineEdit *m_url = nullptr;
    QPushButton *m_copy = nullptr;
    QPushButton *m_open = nullptr;
    QPushButton *m_openUsb = nullptr;
    QPushButton *m_toggle = nullptr;
    bool m_running = false;
};
