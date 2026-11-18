// Orbiscreen - Qt Display page (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#pragma once

#include <QSize>
#include <QWidget>

#include "report.h"
#include "uikit.h"

class QAbstractButton;
class QComboBox;
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
    void applySelection();
    void updateApplyState();

    UiKit::Card *m_geometryCard = nullptr;
    UiKit::Card *m_encoderCard = nullptr;
    UiKit::Card *m_inputCard = nullptr;
    UiKit::Card *m_systemCard = nullptr;

    QLabel *m_resolutionCaption = nullptr;
    QLabel *m_fpsCaption = nullptr;
    QLabel *m_encoder = nullptr;
    QLabel *m_touchTitle = nullptr;
    QLabel *m_touchDesc = nullptr;
    QLabel *m_stylusTitle = nullptr;
    QLabel *m_stylusDesc = nullptr;
    QLabel *m_autostartTitle = nullptr;
    QLabel *m_autostartDesc = nullptr;
    QComboBox *m_resolution = nullptr;
    QComboBox *m_fps = nullptr;
    QPushButton *m_apply = nullptr;
    QAbstractButton *m_autostart = nullptr;
    QSize m_appliedResolution;
    int m_appliedFps = 0;
    bool m_updating = false;
    bool m_pendingResolution = false;
    bool m_pendingAutostart = false;
};
