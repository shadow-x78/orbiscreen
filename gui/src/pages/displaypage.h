// Orbiscreen - Qt Display page (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#pragma once

#include <QWidget>

#include "report.h"

class QCheckBox;
class QComboBox;
class QGroupBox;
class QLabel;
class QPushButton;

class DisplayPage : public QWidget {
    Q_OBJECT

public:
    explicit DisplayPage(QWidget *parent = nullptr);

    void update(const DaemonStatus &status, bool autostart);
    void setAutostart(bool enabled);
    void resolutionApplied();
    void retranslate();

signals:
    void resolutionApplyRequested(int width, int height, int fps);
    void autostartChanged(bool enabled);

private:
    QGroupBox *m_geometryGroup = nullptr;
    QGroupBox *m_encoderGroup = nullptr;
    QGroupBox *m_inputGroup = nullptr;
    QGroupBox *m_systemGroup = nullptr;
    QLabel *m_resolutionLabel = nullptr;
    QLabel *m_fpsLabel = nullptr;
    QLabel *m_encoderLabel = nullptr;
    QLabel *m_touchTitle = nullptr;
    QLabel *m_touchDesc = nullptr;
    QLabel *m_stylusTitle = nullptr;
    QLabel *m_stylusDesc = nullptr;
    QLabel *m_autostartDesc = nullptr;
    QComboBox *m_resolution = nullptr;
    QComboBox *m_fps = nullptr;
    QPushButton *m_apply = nullptr;
    QLabel *m_encoder = nullptr;
    QCheckBox *m_autostart = nullptr;
    bool m_updating = false;
    bool m_pendingResolution = false;
    bool m_pendingAutostart = false;
};
