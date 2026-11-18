// Orbiscreen - Qt main window (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#pragma once

#include <QImage>
#include <QMainWindow>
#include <QString>

#include "i18n.h"
#include "report.h"

class QAction;
class QCloseEvent;
class QFrame;
class QLabel;
class QMenu;
class QPushButton;
class QToolButton;
class ConnectPage;
class DisplayPage;
class DoctorPage;
class DaemonClient;
class TrayIcon;

namespace UiKit {
class LogoWidget;
class NavTabs;
class StatusPill;
}

class MainWindow : public QMainWindow {
    Q_OBJECT

public:
    explicit MainWindow(QWidget *parent = nullptr);

protected:
    void closeEvent(QCloseEvent *event) override;

private:
    void buildUi();
    void createActions();
    void createMenu();
    void createStatusBar();
    void connectSignals();
    void retranslate();
    void syncHeaderAction();
    void applyState();
    QString stateText() const;
    QString stateKey() const;
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
    void checkForUpdates();
    void handleUpdate(const QString &current, const QString &latest, const QString &url,
                      bool available);
    void refreshUpdateBanner();

    DaemonClient *m_client = nullptr;
    TrayIcon *m_tray = nullptr;
    ConnectPage *m_connect = nullptr;
    DisplayPage *m_display = nullptr;
    DoctorPage *m_doctor = nullptr;
    UiKit::NavTabs *m_nav = nullptr;

    QAction *m_startStop = nullptr;
    QAction *m_restart = nullptr;
    QAction *m_refresh = nullptr;
    QAction *m_quit = nullptr;
    QAction *m_about = nullptr;
    QAction *m_checkUpdates = nullptr;
    QAction *m_openDashboard = nullptr;
    QAction *m_viewConnect = nullptr;
    QAction *m_viewDisplay = nullptr;
    QAction *m_viewDoctor = nullptr;
    QAction *m_langEn = nullptr;
    QAction *m_langAr = nullptr;

    QMenu *m_overflow = nullptr;
    QMenu *m_viewMenu = nullptr;
    QMenu *m_languageMenu = nullptr;

    UiKit::LogoWidget *m_appIcon = nullptr;
    QLabel *m_appTitle = nullptr;
    QLabel *m_appTagline = nullptr;
    QLabel *m_resolutionLabel = nullptr;
    QLabel *m_portsLabel = nullptr;
    QLabel *m_clientsLabel = nullptr;
    UiKit::StatusPill *m_statusPill = nullptr;
    QToolButton *m_headerAction = nullptr;
    QToolButton *m_menuButton = nullptr;

    QFrame *m_updateBar = nullptr;
    QLabel *m_updateText = nullptr;
    QPushButton *m_updateDownload = nullptr;
    QPushButton *m_updateDismiss = nullptr;

    DaemonStatus m_status;
    DoctorReport m_doctorReport;
    bool m_hasDoctor = false;
    bool m_autostart = false;
    bool m_quitting = false;
    bool m_trayNotice = false;
    QString m_url;
    QString m_qrUrl;
    QString m_updateVersion;
    QString m_updateUrl;
    QImage m_qr;
    int m_lastHeaderActionKey = -1;
};
