// Orbiscreen - Qt main window (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#pragma once

#include <QImage>
#include <QMainWindow>
#include <QString>

#include "i18n.h"
#include "report.h"

class QAction;
class QCloseEvent;
class QLabel;
class QMenu;
class QTabWidget;
class ConnectPage;
class DisplayPage;
class DoctorPage;
class DaemonClient;
class TrayIcon;

class MainWindow : public QMainWindow {
    Q_OBJECT

public:
    explicit MainWindow(QWidget *parent = nullptr);

protected:
    void closeEvent(QCloseEvent *event) override;

private:
    void buildUi();
    void createActions();
    void createMenus();
    void createToolbar();
    void createStatusBar();
    void connectSignals();
    void retranslate();
    void applyState();
    QString stateText() const;
    void toggleService();
    void restartService();
    void applyResolution(int width, int height, int fps);
    void applyAutostart(bool enabled);
    void scan();
    void fix();
    void openUrl(const QString &url);
    void openUsb();
    void handleStatus(const DaemonStatus &status);
    void handleQr(const QImage &image);
    void handleActionFinished(const QString &message);
    void handleActionFailed(const QString &error);
    void handleDoctorReady(const DoctorReport &report);
    void handleDoctorFailed(const QString &error);
    void setLanguage(Lang lang);
    void quitApplication();
    void showFromTray();
    void showAbout();

    DaemonClient *m_client = nullptr;
    TrayIcon *m_tray = nullptr;
    ConnectPage *m_connect = nullptr;
    DisplayPage *m_display = nullptr;
    DoctorPage *m_doctor = nullptr;
    QTabWidget *m_tabs = nullptr;

    QAction *m_startStop = nullptr;
    QAction *m_restart = nullptr;
    QAction *m_refresh = nullptr;
    QAction *m_quit = nullptr;
    QAction *m_about = nullptr;
    QAction *m_viewConnect = nullptr;
    QAction *m_viewDisplay = nullptr;
    QAction *m_viewDoctor = nullptr;
    QAction *m_langEn = nullptr;
    QAction *m_langAr = nullptr;

    QMenu *m_serviceMenu = nullptr;
    QMenu *m_viewMenu = nullptr;
    QMenu *m_languageMenu = nullptr;
    QMenu *m_helpMenu = nullptr;

    QLabel *m_toolbarState = nullptr;
    QLabel *m_stateLabel = nullptr;
    QLabel *m_resolutionLabel = nullptr;
    QLabel *m_portsLabel = nullptr;
    QLabel *m_clientsLabel = nullptr;

    DaemonStatus m_status;
    DoctorReport m_doctorReport;
    bool m_hasDoctor = false;
    bool m_autostart = false;
    bool m_quitting = false;
    bool m_trayNotice = false;
    QString m_url;
    QString m_qrUrl;
    QImage m_qr;
};
