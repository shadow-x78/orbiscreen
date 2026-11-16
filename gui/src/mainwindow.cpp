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
#include <QHBoxLayout>
#include <QKeySequence>
#include <QLabel>
#include <QMenu>
#include <QMessageBox>
#include <QPushButton>
#include <QSize>
#include <QStatusBar>
#include <QTabWidget>
#include <QTimer>
#include <QToolButton>
#include <QVBoxLayout>
#include <QWidget>

namespace {

QString t(const char *key)
{
    return I18n::instance().t(QString::fromLatin1(key));
}

QString escaped(const QString &text)
{
    QString result = text;
    result.replace(QLatin1Char('&'), QStringLiteral("&&"));
    return result;
}

QLabel *statusLabel(const QString &name)
{
    auto *label = new QLabel;
    label->setObjectName(name);
    label->setAlignment(Qt::AlignLeft | Qt::AlignVCenter);
    return label;
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
    resize(1040, 780);
    setMinimumSize(840, 620);

    m_client = new DaemonClient(this);
    m_tray = new TrayIcon(this);
    m_connect = new ConnectPage(this);
    m_display = new DisplayPage(this);
    m_doctor = new DoctorPage(this);

    createActions();
    createMenu();

    auto *header = new QWidget(this);
    header->setObjectName(QStringLiteral("Header"));
    auto *headerLayout = new QHBoxLayout(header);
    headerLayout->setContentsMargins(20, 14, 20, 14);
    headerLayout->setSpacing(14);
    m_appIcon = new QLabel(header);
    m_appIcon->setPixmap(Theme::logo(42));
    headerLayout->addWidget(m_appIcon);
    auto *titleBox = new QVBoxLayout;
    titleBox->setSpacing(1);
    m_appTitle = new QLabel(QStringLiteral("Orbiscreen"), header);
    m_appTitle->setObjectName(QStringLiteral("AppTitle"));
    m_appTagline = UiKit::label(t("appTagline"), QStringLiteral("Muted"));
    titleBox->addWidget(m_appTitle);
    titleBox->addWidget(m_appTagline);
    headerLayout->addLayout(titleBox);
    headerLayout->addStretch(1);

    m_statusPill = new UiKit::StatusPill(header);
    headerLayout->addWidget(m_statusPill, 0, Qt::AlignVCenter);

    m_headerAction = new QToolButton(header);
    m_headerAction->setObjectName(QStringLiteral("HeaderAction"));
    m_headerAction->setToolButtonStyle(Qt::ToolButtonTextBesideIcon);
    m_headerAction->setIconSize(QSize(18, 18));
    m_headerAction->setMinimumHeight(38);
    m_headerAction->setCursor(Qt::PointingHandCursor);
    headerLayout->addWidget(m_headerAction, 0, Qt::AlignVCenter);

    m_restartButton = UiKit::iconButton(Theme::Glyph::Restart, t("restartTitle"), 20);
    m_restartButton->setDefaultAction(m_restart);
    m_restartButton->setToolButtonStyle(Qt::ToolButtonIconOnly);
    headerLayout->addWidget(m_restartButton, 0, Qt::AlignVCenter);

    m_refreshButton = UiKit::iconButton(Theme::Glyph::Refresh, t("actionRefresh"), 20);
    m_refreshButton->setDefaultAction(m_refresh);
    m_refreshButton->setToolButtonStyle(Qt::ToolButtonIconOnly);
    headerLayout->addWidget(m_refreshButton, 0, Qt::AlignVCenter);

    m_menuButton = UiKit::iconButton(Theme::Glyph::More, t("moreOptions"), 20);
    m_menuButton->setPopupMode(QToolButton::InstantPopup);
    m_menuButton->setMenu(m_overflow);
    headerLayout->addWidget(m_menuButton, 0, Qt::AlignVCenter);

    m_updateBar = new QFrame(this);
    m_updateBar->setObjectName(QStringLiteral("UpdateBar"));
    m_updateBar->hide();
    auto *updateLayout = new QHBoxLayout(m_updateBar);
    updateLayout->setContentsMargins(14, 10, 14, 10);
    updateLayout->setSpacing(10);
    updateLayout->addWidget(UiKit::iconLabel(Theme::Glyph::Download, 20,
                                             Theme::colors().accent));
    m_updateText = UiKit::label(QString(), QString(), true);
    updateLayout->addWidget(m_updateText, 1);
    m_updateDownload = UiKit::button(t("download"), QStringLiteral("primary"));
    m_updateDismiss = UiKit::button(t("later"), QStringLiteral("ghost"));
    updateLayout->addWidget(m_updateDownload);
    updateLayout->addWidget(m_updateDismiss);
    auto *updateWrap = new QWidget(this);
    auto *updateWrapLayout = new QHBoxLayout(updateWrap);
    updateWrapLayout->setContentsMargins(20, 12, 20, 0);
    updateWrapLayout->setSpacing(0);
    updateWrapLayout->addWidget(m_updateBar);

    m_tabs = new QTabWidget(this);
    m_tabs->setDocumentMode(true);
    m_tabs->setIconSize(QSize(18, 18));
    m_tabs->addTab(m_connect, QString());
    m_tabs->addTab(m_display, QString());
    m_tabs->addTab(m_doctor, QString());

    auto *central = new QWidget(this);
    central->setObjectName(QStringLiteral("Central"));
    auto *centralLayout = new QVBoxLayout(central);
    centralLayout->setContentsMargins(0, 0, 0, 0);
    centralLayout->setSpacing(0);
    centralLayout->addWidget(header);
    centralLayout->addWidget(updateWrap);
    centralLayout->addWidget(m_tabs, 1);
    setCentralWidget(central);

    connect(m_updateDownload, &QPushButton::clicked, this,
            [this]() { openUrl(m_updateUrl); });
    connect(m_updateDismiss, &QPushButton::clicked, this,
            [this]() { m_updateBar->hide(); });

    createStatusBar();
    connectSignals();

    m_autostart = DaemonClient::isAutostartEnabled();
    m_display->setAutostart(m_autostart);

    retranslate();
    applyState();
    refreshTabIcons();
    m_client->refreshStatus();
    QTimer::singleShot(2500, this, &MainWindow::checkForUpdates);
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

    m_checkUpdates = new QAction(this);
    connect(m_checkUpdates, &QAction::triggered, this, &MainWindow::checkForUpdates);

    m_viewConnect = new QAction(this);
    connect(m_viewConnect, &QAction::triggered, this,
            [this]() { m_tabs->setCurrentIndex(0); });
    m_viewDisplay = new QAction(this);
    connect(m_viewDisplay, &QAction::triggered, this,
            [this]() { m_tabs->setCurrentIndex(1); });
    m_viewDoctor = new QAction(this);
    connect(m_viewDoctor, &QAction::triggered, this,
            [this]() { m_tabs->setCurrentIndex(2); });
    m_openDashboard = new QAction(this);
    m_openDashboard->setShortcut(QKeySequence(QStringLiteral("Ctrl+D")));
    connect(m_openDashboard, &QAction::triggered, this,
            [this]() { openUrl(m_url); });

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

void MainWindow::createMenu()
{
    m_overflow = new QMenu(this);
    m_overflow->addAction(m_startStop);
    m_overflow->addAction(m_restart);
    m_overflow->addAction(m_refresh);
    m_overflow->addAction(m_openDashboard);
    m_overflow->addSeparator();

    m_viewMenu = m_overflow->addMenu(QString());
    m_viewMenu->addAction(m_viewConnect);
    m_viewMenu->addAction(m_viewDisplay);
    m_viewMenu->addAction(m_viewDoctor);

    m_languageMenu = m_overflow->addMenu(QString());
    m_languageMenu->addAction(m_langEn);
    m_languageMenu->addAction(m_langAr);

    m_overflow->addSeparator();
    m_overflow->addAction(m_checkUpdates);
    m_overflow->addAction(m_about);
    m_overflow->addSeparator();
    m_overflow->addAction(m_quit);
}

void MainWindow::createStatusBar()
{
    m_stateLabel = statusLabel(QStringLiteral("StatusState"));
    m_resolutionLabel = statusLabel(QStringLiteral("StatusText"));
    m_portsLabel = statusLabel(QStringLiteral("StatusText"));
    m_clientsLabel = statusLabel(QStringLiteral("StatusText"));
    const auto addSeparator = []() {
        auto *label = new QLabel(QStringLiteral("·"));
        label->setObjectName(QStringLiteral("StatusText"));
        return label;
    };
    statusBar()->setSizeGripEnabled(false);
    statusBar()->addPermanentWidget(m_stateLabel);
    statusBar()->addPermanentWidget(addSeparator());
    statusBar()->addPermanentWidget(m_resolutionLabel);
    statusBar()->addPermanentWidget(addSeparator());
    statusBar()->addPermanentWidget(m_portsLabel);
    statusBar()->addPermanentWidget(addSeparator());
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
    connect(m_client, &DaemonClient::updateReady, this, &MainWindow::handleUpdate);

    connect(m_connect, &ConnectPage::openRequested, this, &MainWindow::openUrl);
    connect(m_connect, &ConnectPage::openUsbRequested, this, &MainWindow::openUsb);
    connect(m_display, &DisplayPage::resolutionApplyRequested, this,
            &MainWindow::applyResolution);
    connect(m_display, &DisplayPage::autostartChanged, this, &MainWindow::applyAutostart);
    connect(m_doctor, &DoctorPage::scanRequested, this, &MainWindow::scan);
    connect(m_doctor, &DoctorPage::fixRequested, this, &MainWindow::fix);

    connect(m_headerAction, &QToolButton::clicked, this, &MainWindow::toggleService);

    connect(m_tabs, &QTabWidget::currentChanged, this, [this]() { refreshTabIcons(); });

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

void MainWindow::refreshTabIcons()
{
    const int index = m_tabs->currentIndex();
    if (index == m_lastTabIconIndex) {
        return;
    }
    m_lastTabIconIndex = index;
    const QColor active = Theme::colors().accent;
    const QColor idle = Theme::colors().muted;
    const Theme::Glyph glyphs[3] = {Theme::Glyph::Phone, Theme::Glyph::Monitor,
                                    Theme::Glyph::Doctor};
    for (int tab = 0; tab < 3; ++tab) {
        m_tabs->setTabIcon(tab, Theme::glyph(glyphs[tab], index == tab ? active : idle));
    }
}

void MainWindow::retranslate()
{
    m_lastHeaderActionKey = -1;
    m_viewMenu->setTitle(t("menuView"));
    m_languageMenu->setTitle(t("menuLanguage"));

    m_startStop->setText(m_status.report.running ? t("stopDisplay") : t("startDisplay"));
    m_startStop->setIcon(Theme::glyph(m_status.report.running ? Theme::Glyph::Stop
                                                              : Theme::Glyph::Play));
    m_restart->setText(t("restartTitle"));
    m_restart->setIcon(Theme::glyph(Theme::Glyph::Restart));
    m_refresh->setText(t("actionRefresh"));
    m_refresh->setIcon(Theme::glyph(Theme::Glyph::Refresh));
    m_quit->setText(t("quit"));
    m_about->setText(t("about"));
    m_about->setIcon(Theme::glyph(Theme::Glyph::Info));
    m_checkUpdates->setText(t("actionCheckUpdates"));
    m_checkUpdates->setIcon(Theme::glyph(Theme::Glyph::Download));
    m_viewConnect->setText(t("tabConnect"));
    m_viewDisplay->setText(escaped(t("tabDisplay")));
    m_viewDoctor->setText(t("tabDoctor"));
    m_openDashboard->setText(t("actionOpenDashboard"));
    m_openDashboard->setIcon(Theme::glyph(Theme::Glyph::Dashboard));

    m_appTagline->setText(t("appTagline"));
    const Qt::Alignment appTextAlign =
        (I18n::instance().lang() == Lang::Ar ? Qt::AlignRight : Qt::AlignLeft)
        | Qt::AlignVCenter;
    m_appTitle->setAlignment(appTextAlign);
    m_appTagline->setAlignment(appTextAlign);
    m_updateDownload->setText(t("download"));
    m_updateDismiss->setText(t("later"));
    m_menuButton->setToolTip(t("moreOptions"));

    m_tabs->setTabText(0, t("tabConnect"));
    m_tabs->setTabText(1, escaped(t("tabDisplay")));
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
    refreshUpdateBanner();
    applyState();
}

void MainWindow::applyState()
{
    m_connect->update(m_status, m_qr, m_url);
    m_display->update(m_status, m_autostart);
    m_doctor->update(m_status, m_doctorReport, m_hasDoctor);
    m_tray->setState(m_status.report.running, m_status.report.activeClients);

    const bool running = m_status.report.running;
    m_startStop->setText(running ? t("stopDisplay") : t("startDisplay"));
    m_startStop->setIcon(Theme::glyph(running ? Theme::Glyph::Stop : Theme::Glyph::Play));

    syncHeaderAction();

    m_statusPill->setStatus(stateKey(), stateText());
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
    refreshTabIcons();
}

void MainWindow::syncHeaderAction()
{
    if (!m_headerAction) {
        return;
    }
    const bool running = m_status.report.running;
    const int key = (running ? 2 : 0) + (m_headerAction->isEnabled() ? 1 : 0);
    if (key == m_lastHeaderActionKey) {
        return;
    }
    m_lastHeaderActionKey = key;
    QColor iconColor = running ? Theme::colors().danger : Theme::colors().accentText;
    if (!m_headerAction->isEnabled()) {
        iconColor = Theme::colors().muted;
    }
    m_headerAction->setText(running ? t("stopDisplay") : t("startDisplay"));
    m_headerAction->setIcon(Theme::glyph(running ? Theme::Glyph::Stop : Theme::Glyph::Play,
                                         iconColor));
    UiKit::setVariant(m_headerAction, running ? QStringLiteral("danger")
                                              : QStringLiteral("primary"));
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

QString MainWindow::stateKey() const
{
    if (!m_status.report.running) {
        return QStringLiteral("stopped");
    }
    return m_status.report.activeClients > 0 ? QStringLiteral("streaming")
                                             : QStringLiteral("ready");
}

void MainWindow::toggleService()
{
    m_startStop->setEnabled(false);
    m_restart->setEnabled(false);
    m_headerAction->setEnabled(false);
    syncHeaderAction();
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
    m_headerAction->setEnabled(false);
    syncHeaderAction();
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
    m_display->setAutostart(enabled);
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
    m_display->resolutionApplied();
    m_startStop->setEnabled(true);
    m_restart->setEnabled(true);
    m_headerAction->setEnabled(true);
    syncHeaderAction();
    m_doctor->setBusy(false, false);
    statusBar()->showMessage(message, 5000);
    m_client->refreshStatus();
}

void MainWindow::handleActionFailed(const QString &error)
{
    m_display->resolutionApplied();
    m_startStop->setEnabled(true);
    m_restart->setEnabled(true);
    m_headerAction->setEnabled(true);
    syncHeaderAction();
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

void MainWindow::checkForUpdates()
{
    m_client->checkForUpdates();
}

void MainWindow::handleUpdate(const QString &current, const QString &latest,
                              const QString &url, bool available)
{
    Q_UNUSED(current);
    if (!available) {
        m_updateVersion.clear();
        m_updateUrl.clear();
        m_updateBar->hide();
        return;
    }
    m_updateVersion = latest;
    m_updateUrl = url;
    refreshUpdateBanner();
}

void MainWindow::refreshUpdateBanner()
{
    if (m_updateVersion.isEmpty()) {
        m_updateBar->hide();
        return;
    }
    m_updateText->setText(t("updateAvailable")
                              .arg(m_updateVersion, QApplication::applicationVersion()));
    m_updateBar->show();
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
