// Orbiscreen - Qt system tray (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#include "trayicon.h"

#include "i18n.h"
#include "theme.h"

#include <QAction>
#include <QMenu>
#include <QSystemTrayIcon>

TrayIcon::TrayIcon(QObject *parent)
    : QObject(parent)
{
    if (!QSystemTrayIcon::isSystemTrayAvailable()) {
        return;
    }

    m_menu = new QMenu;
    m_open = m_menu->addAction(QString());
    m_menu->addSeparator();
    m_start = m_menu->addAction(QString());
    m_stop = m_menu->addAction(QString());
    m_menu->addSeparator();
    m_quit = m_menu->addAction(QString());

    connect(m_open, &QAction::triggered, this, &TrayIcon::openRequested);
    connect(m_start, &QAction::triggered, this, &TrayIcon::startRequested);
    connect(m_stop, &QAction::triggered, this, &TrayIcon::stopRequested);
    connect(m_quit, &QAction::triggered, this, &TrayIcon::quitRequested);

    m_tray = new QSystemTrayIcon(this);
    m_tray->setIcon(Theme::icon(
        {QStringLiteral("orbiscreen-gui"), QStringLiteral("orbiscreen"),
         QStringLiteral("video-display")},
        QStyle::SP_ComputerIcon));
    m_tray->setContextMenu(m_menu);
    connect(m_tray, &QSystemTrayIcon::activated, this,
            [this](QSystemTrayIcon::ActivationReason reason) {
                if (reason == QSystemTrayIcon::Trigger
                    || reason == QSystemTrayIcon::DoubleClick) {
                    emit openRequested();
                }
            });
    m_tray->show();
    retranslate();
}

TrayIcon::~TrayIcon()
{
    delete m_menu;
}

bool TrayIcon::isAvailable() const
{
    return m_tray != nullptr;
}

void TrayIcon::setState(bool running, int clients)
{
    m_running = running;
    m_clients = clients;
    if (!m_tray) {
        return;
    }

    QString description;
    if (!running) {
        description = I18n::instance().t(QStringLiteral("stopped"));
    } else if (clients > 0) {
        description = I18n::instance().format(QStringLiteral("devicesConnected"), clients);
    } else {
        description = I18n::instance().t(QStringLiteral("ready"));
    }
    m_tray->setToolTip(QStringLiteral("Orbiscreen\n") + description);
    if (m_start) {
        m_start->setEnabled(!running);
    }
    if (m_stop) {
        m_stop->setEnabled(running);
    }
}

void TrayIcon::retranslate()
{
    if (!m_menu) {
        return;
    }
    m_open->setText(I18n::instance().t(QStringLiteral("actionOpenDashboard")));
    m_start->setText(I18n::instance().t(QStringLiteral("startDisplay")));
    m_stop->setText(I18n::instance().t(QStringLiteral("stopDisplay")));
    m_quit->setText(I18n::instance().t(QStringLiteral("quit")));
    setState(m_running, m_clients);
}

void TrayIcon::showMessage(const QString &title, const QString &message)
{
    if (m_tray) {
        m_tray->showMessage(title, message, QSystemTrayIcon::Information, 4000);
    }
}
