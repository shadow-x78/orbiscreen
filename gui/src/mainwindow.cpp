// Orbiscreen - Qt main window (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#include "mainwindow.h"

#include "daemonclient.h"
#include "i18n.h"
#include "pages/connectpage.h"
#include "pages/displaypage.h"
#include "pages/doctorpage.h"
#include "theme.h"
#include "trayicon.h"
#include "uikit.h"

#include <QAction>
#include <QActionGroup>
#include <QApplication>
#include <QCloseEvent>
#include <QFrame>
#include <QKeySequence>
#include <QLabel>
#include <QMenu>
#include <QMenuBar>
#include <QMessageBox>
#include <QSizePolicy>
#include <QStatusBar>
#include <QStyle>
#include <QTabWidget>
#include <QToolBar>
#include <QWidget>

namespace {

QString t(const char *key)
{
    return I18n::instance().t(QString::fromLatin1(key));
}

}

MainWindow::MainWindow(QWidget *parent)
    : QMainWindow(parent)
{
    buildUi();
}

void MainWindow::buildUi()
{
    setWindowTitle(QStringLiteral("Orbiscreen"));
    resize(900, 680);
    setMinimumSize(720, 520);

    m_client = new DaemonClient(this);
    m_tray = new TrayIcon(this);
    m_connect = new ConnectPage(this);
    m_display = new DisplayPage(this);
    m_doctor = new DoctorPage(this);

    m_tabs = new QTabWidget(this);
    m_tabs->setDocumentMode(true);
    m_tabs->addTab(m_connect, QString());
    m_tabs->addTab(m_display, QString());
    m_tabs->addTab(m_doctor, QString());
    setCentralWidget(m_tabs);

    createActions();
    createMenus();
    createToolbar();
    createStatusBar();
    connectSignals();

    m_autostart = DaemonClient::isAutostartEnabled();
    m_display->setAutostart(m_autostart);

    retranslate();
    applyState();
    m_client->refreshStatus();
}

void MainWindow::createActions()
{
    m_startStop = new QAction(this);
    connect(m_startStop, &QAction::triggered, this, &MainWindow::toggleService);

    m_restart = new QAction(this);
    connect(m_restart, &QAction::triggered, this, &MainWindow::restartService);

    m_refresh = new QAction(this);
    m_refresh->setShortcut(QKeySequence::Refresh);
    connect(m_refresh, &QAction::triggered, this, [this]() { m_client->refreshStatus(); });

    m_quit = new QAction(this);
    m_quit->setShortcut(QKeySequence::Quit);
    connect(m_quit, &QAction::triggered, this, &MainWindow::quitApplication);

    m_about = new QAction(this);
    connect(m_about, &QAction::triggered, this, &MainWindow::showAbout);

    m_viewConnect = new QAction(this);
    connect(m_viewConnect, &QAction::triggered, this,
            [this]() { m_tabs->setCurrentIndex(0); });
    m_viewDisplay = new QAction(this);
    connect(m_viewDisplay, &QAction::triggered, this,
            [this]() { m_tabs->setCurrentIndex(1); });
    m_viewDoctor = new QAction(this);
    connect(m_viewDoctor, &QAction::triggered, this,
            [this]() { m_tabs->setCurrentIndex(2); });

    m_langEn = new QAction(QStringLiteral("English"), this);
    m_langEn->setCheckable(true);
    m_langAr = new QAction(QStringLiteral("العربية"), this);
    m_langAr->setCheckable(true);
    auto *languages = new QActionGroup(this);
    languages->setExclusive(true);
    languages->addAction(m_langEn);
    languages->addAction(m_langAr);
    connect(m_langEn, &QAction::triggered, this, [this]() { setLanguage(Lang::En); });
    connect(m_langAr, &QAction::triggered, this, [this]() { setLanguage(Lang::Ar); });
}

void MainWindow::createMenus()
{
    m_serviceMenu = menuBar()->addMenu(QString());
    m_serviceMenu->addAction(m_startStop);
    m_serviceMenu->addAction(m_restart);
    m_serviceMenu->addSeparator();
    m_serviceMenu->addAction(m_quit);

    m_viewMenu = menuBar()->addMenu(QString());
    m_viewMenu->addAction(m_viewConnect);
    m_viewMenu->addAction(m_viewDisplay);
    m_viewMenu->addAction(m_viewDoctor);
    m_viewMenu->addSeparator();
    m_viewMenu->addAction(m_refresh);

    m_languageMenu = menuBar()->addMenu(QString());
    m_languageMenu->addAction(m_langEn);
    m_languageMenu->addAction(m_langAr);

    m_helpMenu = menuBar()->addMenu(QString());
    m_helpMenu->addAction(m_about);
}

void MainWindow::createToolbar()
{
    auto *toolbar = addToolBar(QStringLiteral("Main"));
    toolbar->setObjectName(QStringLiteral("MainToolbar"));
    toolbar->setMovable(false);
    toolbar->setToolButtonStyle(Qt::ToolButtonTextBesideIcon);
    toolbar->addAction(m_startStop);
    toolbar->addAction(m_restart);
    toolbar->addAction(m_refresh);
    auto *spacer = new QWidget(toolbar);
    spacer->setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Preferred);
    toolbar->addWidget(spacer);
    m_toolbarState = new QLabel(toolbar);
    m_toolbarState->setContentsMargins(0, 0, 8, 0);
    toolbar->addWidget(m_toolbarState);
}

void MainWindow::createStatusBar()
{
    m_stateLabel = new QLabel(this);
    m_resolutionLabel = new QLabel(this);
    m_portsLabel = new QLabel(this);
    m_clientsLabel = new QLabel(this);
    statusBar()->addPermanentWidget(m_stateLabel);
    statusBar()->addPermanentWidget(m_resolutionLabel);
    statusBar()->addPermanentWidget(m_portsLabel);
    statusBar()->addPermanentWidget(m_clientsLabel);
}

void MainWindow::connectSignals()
{
    connect(m_client, &DaemonClient::statusReady, this, &MainWindow::handleStatus);
    connect(m_client, &DaemonClient::statusFailed, this, [this](const QString &error) {
        statusBar()->showMessage(error, 5000);
    });
    connect(m_client, &DaemonClient::actionFinished, this, &MainWindow::handleActionFinished);
    connect(m_client, &DaemonClient::actionFailed, this, &MainWindow::handleActionFailed);
    connect(m_client, &DaemonClient::doctorReady, this, &MainWindow::handleDoctorReady);
    connect(m_client, &DaemonClient::doctorFailed, this, &MainWindow::handleDoctorFailed);
    connect(m_client, &DaemonClient::qrReady, this, &MainWindow::handleQr);

    connect(m_connect, &ConnectPage::startStopRequested, this,
            [this](bool) { toggleService(); });
    connect(m_connect, &ConnectPage::openRequested, this, &MainWindow::openUrl);
    connect(m_connect, &ConnectPage::openUsbRequested, this, &MainWindow::openUsb);
    connect(m_display, &DisplayPage::resolutionApplyRequested, this,
            &MainWindow::applyResolution);
    connect(m_display, &DisplayPage::autostartChanged, this, &MainWindow::applyAutostart);
    connect(m_doctor, &DoctorPage::scanRequested, this, &MainWindow::scan);
    connect(m_doctor, &DoctorPage::fixRequested, this, &MainWindow::fix);

    connect(m_tray, &TrayIcon::openRequested, this, &MainWindow::showFromTray);
    connect(m_tray, &TrayIcon::startRequested, this, [this]() {
        if (!m_status.report.running) {
            toggleService();
        }
    });
    connect(m_tray, &TrayIcon::stopRequested, this, [this]() {
        if (m_status.report.running) {
            toggleService();
        }
    });
    connect(m_tray, &TrayIcon::quitRequested, this, &MainWindow::quitApplication);
}

void MainWindow::retranslate()
{
    m_serviceMenu->setTitle(t("menuService"));
    m_viewMenu->setTitle(t("menuView"));
    m_languageMenu->setTitle(t("menuLanguage"));
    m_helpMenu->setTitle(t("menuHelp"));

    m_startStop->setText(m_status.report.running ? t("stopDisplay") : t("startDisplay"));
    m_startStop->setIcon(Theme::icon(
        {m_status.report.running ? QStringLiteral("media-playback-stop")
                                 : QStringLiteral("media-playback-start")},
        m_status.report.running ? QStyle::SP_MediaStop : QStyle::SP_MediaPlay));
    m_restart->setText(t("restartTitle"));
    m_restart->setIcon(Theme::icon({QStringLiteral("view-refresh")},
                                   QStyle::SP_BrowserReload));
    m_refresh->setText(t("actionRefresh"));
    m_refresh->setIcon(Theme::icon({QStringLiteral("view-refresh")},
                                   QStyle::SP_BrowserReload));
    m_quit->setText(t("quit"));
    m_about->setText(t("about"));
    m_viewConnect->setText(t("tabConnect"));
    m_viewDisplay->setText(t("tabDisplay"));
    m_viewDoctor->setText(t("tabDoctor"));

    m_tabs->setTabText(0, t("tabConnect"));
    m_tabs->setTabText(1, t("tabDisplay"));
    m_tabs->setTabText(2, t("tabDoctor"));

    m_langEn->setChecked(I18n::instance().lang() == Lang::En);
    m_langAr->setChecked(I18n::instance().lang() == Lang::Ar);

    QApplication::setLayoutDirection(I18n::instance().lang() == Lang::Ar
                                         ? Qt::RightToLeft
                                         : Qt::LeftToRight);

    m_connect->retranslate();
    m_display->retranslate();
    m_doctor->retranslate();
    m_tray->retranslate();
}

void MainWindow::applyState()
{
    m_connect->update(m_status, m_qr, m_url);
    m_display->update(m_status, m_autostart);
    m_doctor->update(m_status, m_doctorReport, m_hasDoctor);
    m_tray->setState(m_status.report.running, m_status.report.activeClients);

    const bool running = m_status.report.running;
    m_startStop->setText(running ? t("stopDisplay") : t("startDisplay"));
    m_startStop->setIcon(Theme::icon(
        {running ? QStringLiteral("media-playback-stop")
                 : QStringLiteral("media-playback-start")},
        running ? QStyle::SP_MediaStop : QStyle::SP_MediaPlay));

    m_toolbarState->setText(stateText());
    m_stateLabel->setText(stateText());
    m_resolutionLabel->setText(QStringLiteral("%1 × %2 @ %3 Hz")
                                   .arg(m_status.report.displayWidth)
                                   .arg(m_status.report.displayHeight)
                                   .arg(m_status.report.displayFps));
    m_portsLabel->setText(QStringLiteral("%1 / %2")
                              .arg(m_status.report.signalingPort)
                              .arg(m_status.udpPort));
    const int clients = m_status.report.activeClients;
    m_clientsLabel->setText(clients == 0
                                ? t("noDevices")
                                : clients == 1
                                      ? t("deviceConnected")
                                      : I18n::instance().format(
                                            QStringLiteral("devicesConnected"), clients));
}

QString MainWindow::stateText() const
{
    if (!m_status.report.running) {
        return t("stopped");
    }
    if (m_status.report.activeClients > 0) {
        return QStringLiteral("%1 · %2")
            .arg(t("streaming"),
                 I18n::instance().format(QStringLiteral("devicesConnected"),
                                         m_status.report.activeClients));
    }
    return t("ready");
}

void MainWindow::toggleService()
{
    m_startStop->setEnabled(false);
    m_restart->setEnabled(false);
    if (m_status.report.running) {
        statusBar()->showMessage(t("toastStopped"), 4000);
        m_client->stopService();
    } else {
        statusBar()->showMessage(t("toastStarted"), 4000);
        m_client->startService();
    }
}

void MainWindow::restartService()
{
    m_startStop->setEnabled(false);
    m_restart->setEnabled(false);
    statusBar()->showMessage(t("toastRestart"), 4000);
    m_client->restartService();
}

void MainWindow::applyResolution(int width, int height, int fps)
{
    statusBar()->showMessage(QStringLiteral("%1 × %2 @ %3 Hz")
                                 .arg(width)
                                 .arg(height)
                                 .arg(fps),
                             4000);
    m_client->setResolution(width, height, fps);
}

void MainWindow::applyAutostart(bool enabled)
{
    QString error;
    if (!DaemonClient::setAutostart(enabled, &error)) {
        m_display->setAutostart(m_autostart);
        statusBar()->showMessage(error.isEmpty() ? t("toastAutostartFailed") : error, 5000);
        return;
    }
    m_autostart = enabled;
}

void MainWindow::scan()
{
    m_doctor->setBusy(true, false);
    m_client->runDoctor();
}

void MainWindow::fix()
{
    m_doctor->setBusy(false, true);
    m_client->runDoctorFix();
}

void MainWindow::openUrl(const QString &url)
{
    QString error;
    if (!DaemonClient::openUrl(url, &error)) {
        statusBar()->showMessage(error, 5000);
    }
}

void MainWindow::openUsb()
{
    openUrl(DaemonClient::usbUrl(m_status, DaemonClient::token()));
}

void MainWindow::handleStatus(const DaemonStatus &status)
{
    m_status = status;
    const QString url = DaemonClient::sessionUrl(status, DaemonClient::token());
    if (url != m_qrUrl || m_qr.isNull()) {
        m_qrUrl = url;
        m_qr = QImage();
        m_client->renderQr(url);
    }
    m_url = url;
    applyState();
}

void MainWindow::handleQr(const QImage &image)
{
    m_qr = image;
    m_connect->setQr(image);
}

void MainWindow::handleActionFinished(const QString &message)
{
    m_startStop->setEnabled(true);
    m_restart->setEnabled(true);
    m_doctor->setBusy(false, false);
    statusBar()->showMessage(message, 5000);
    m_client->refreshStatus();
}

void MainWindow::handleActionFailed(const QString &error)
{
    m_startStop->setEnabled(true);
    m_restart->setEnabled(true);
    m_doctor->setBusy(false, false);
    statusBar()->showMessage(error, 5000);
}

void MainWindow::handleDoctorReady(const DoctorReport &report)
{
    m_doctorReport = report;
    m_hasDoctor = true;
    m_doctor->setBusy(false, false);
    m_doctor->update(m_status, m_doctorReport, true);
}

void MainWindow::handleDoctorFailed(const QString &error)
{
    m_doctor->setBusy(false, false);
    statusBar()->showMessage(error.isEmpty() ? t("toastDoctorFailed") : error, 5000);
}

void MainWindow::setLanguage(Lang lang)
{
    I18n::instance().setLang(lang);
    retranslate();
    applyState();
}

void MainWindow::quitApplication()
{
    m_quitting = true;
    close();
    QApplication::quit();
}

void MainWindow::showFromTray()
{
    showNormal();
    raise();
    activateWindow();
}

void MainWindow::showAbout()
{
    QMessageBox::about(
        this, t("about"),
        QStringLiteral("<h3>Orbiscreen %1</h3>"
                       "<p>Stream a virtual display to Android devices.</p>"
                       "<p>GPL-3.0-or-later · "
                       "<a href=\"https://github.com/shadow-x78/orbiscreen\">"
                       "github.com/shadow-x78/orbiscreen</a></p>")
            .arg(QApplication::applicationVersion()));
}

void MainWindow::closeEvent(QCloseEvent *event)
{
    if (!m_quitting && m_tray && m_tray->isAvailable()) {
        hide();
        if (!m_trayNotice) {
            m_tray->showMessage(QStringLiteral("Orbiscreen"), t("statusReadyHeadline"));
            m_trayNotice = true;
        }
        event->ignore();
        return;
    }
    QMainWindow::closeEvent(event);
}
