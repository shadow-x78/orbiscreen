// Orbiscreen - Qt Doctor page (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#pragma once

#include <QWidget>

#include "report.h"
#include "uikit.h"

class QLabel;
class QProgressBar;
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
    UiKit::GlyphWidget *m_backendIcon = nullptr;
    UiKit::GlyphWidget *m_uinputIcon = nullptr;
    UiKit::GlyphWidget *m_encoderIcon = nullptr;
    UiKit::GlyphWidget *m_usbIcon = nullptr;
    QPushButton *m_scan = nullptr;
    QPushButton *m_fix = nullptr;
    QProgressBar *m_progress = nullptr;
    bool m_scanning = false;
    bool m_fixing = false;
};
