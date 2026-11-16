// Orbiscreen - Qt Doctor page (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#pragma once

#include <QWidget>

#include "report.h"
#include "uikit.h"

class QLabel;
class QPushButton;
class QFrame;

class DoctorPage : public QWidget {
    Q_OBJECT

public:
    explicit DoctorPage(QWidget *parent = nullptr);

    void update(const DaemonStatus &status, const DoctorReport &report, bool hasReport);
    void setBusy(bool scanning, bool fixing);
    void retranslate();

signals:
    void scanRequested();
    void fixRequested();

private:
    UiKit::Card *m_headerCard = nullptr;
    UiKit::Card *m_resultsCard = nullptr;
    UiKit::Card *m_notesCard = nullptr;

    QLabel *m_description = nullptr;
    QLabel *m_notesText = nullptr;
    QLabel *m_empty = nullptr;
    QLabel *m_rowTitles[6] = {nullptr, nullptr, nullptr, nullptr, nullptr, nullptr};
    QWidget *m_rowWidgets[6] = {nullptr, nullptr, nullptr, nullptr, nullptr, nullptr};
    QFrame *m_dividers[6] = {nullptr, nullptr, nullptr, nullptr, nullptr, nullptr};
    QLabel *m_compositor = nullptr;
    QLabel *m_backend = nullptr;
    QLabel *m_uinput = nullptr;
    QLabel *m_encoder = nullptr;
    QLabel *m_ports = nullptr;
    QLabel *m_usb = nullptr;
    QLabel *m_backendIcon = nullptr;
    QLabel *m_uinputIcon = nullptr;
    QLabel *m_encoderIcon = nullptr;
    QLabel *m_usbIcon = nullptr;
    QPushButton *m_scan = nullptr;
    QPushButton *m_fix = nullptr;
    bool m_scanning = false;
    bool m_fixing = false;
};
