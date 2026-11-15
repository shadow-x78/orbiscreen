// Orbiscreen - Qt system tray (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#pragma once

#include <QObject>

class QAction;
class QMenu;
class QSystemTrayIcon;

class TrayIcon : public QObject {
    Q_OBJECT

public:
    explicit TrayIcon(QObject *parent = nullptr);
    ~TrayIcon() override;

    bool isAvailable() const;
    void setState(bool running, int clients);
    void retranslate();
    void showMessage(const QString &title, const QString &message);

signals:
    void openRequested();
    void startRequested();
    void stopRequested();
    void quitRequested();

private:
    QSystemTrayIcon *m_tray = nullptr;
    QMenu *m_menu = nullptr;
    QAction *m_open = nullptr;
    QAction *m_start = nullptr;
    QAction *m_stop = nullptr;
    QAction *m_quit = nullptr;
    bool m_running = false;
    int m_clients = 0;
};
