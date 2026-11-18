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
    label->setFont(Theme::font(Theme::FontRole::Caption));
    label->setAlignment(Qt::AlignLeading | Qt::AlignVCenter);
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
    setMinimumSize(800, 600);

    m_client = new DaemonClient(this);
    m_tray = new TrayIcon(this);
    m_connect = new ConnectPage(this);
    m_display = new DisplayPage(this);
    m_doctor = new DoctorPage(this);

    createActions();
    createMenu();

    auto *header = new QWidget(this);
    header->setObjectName(QStringLiteral("Header"));
    auto *headerOuter = new QVBoxLayout(header);
    headerOuter->setContentsMargins(0, Theme::Spacing::md, 0, Theme::Spacing::md);
    headerOuter->setSpacing(0);
    auto *headerColumn = new UiKit::ContentColumn(header);
    auto *headerLayout = new QHBoxLayout;
    headerLayout->setContentsMargins(0, 0, 0, 0);
    headerLayout->setSpacing(Theme::Spacing::md);
    m_appIcon = new UiKit::LogoWidget(44, headerColumn);
    headerLayout->addWidget(m_appIcon, 0, Qt::AlignVCenter);
    auto *titleBox = new QVBoxLayout;
    titleBox->setContentsMargins(0, 0, 0, 0);
    titleBox->setSpacing(0);
    m_appTitle = new QLabel(QStringLiteral("Orbiscreen"), headerColumn);
    m_appTitle->setObjectName(QStringLiteral("AppTitle"));
    m_appTitle->setFont(Theme::font(Theme::FontRole::AppTitle));
    m_appTagline = UiKit::label(t("appTagline"), QStringLiteral("Muted"));
    m_appTagline->setFont(Theme::font(Theme::FontRole::Caption));
    titleBox->addWidget(m_appTitle);
    titleBox->addWidget(m_appTagline);
    headerLayout->addLayout(titleBox);
    headerLayout->addStretch(1);

    m_statusPill = new UiKit::StatusPill(headerColumn);
    headerLayout->addWidget(m_statusPill, 0, Qt::AlignVCenter);

    m_headerAction = new QToolButton(headerColumn);
    m_headerAction->setObjectName(QStringLiteral("HeaderAction"));
    m_headerAction->setToolButtonStyle(Qt::ToolButtonTextBesideIcon);
    m_headerAction->setIconSize(QSize(Theme::Metrics::buttonIconSize,
                                      Theme::Metrics::buttonIconSize));
    m_headerAction->setFixedHeight(Theme::Metrics::controlHeight);
    m_headerAction->setCursor(Qt::PointingHandCursor);
    headerLayout->addWidget(m_headerAction, 0, Qt::AlignVCenter);

    m_menuButton = UiKit::iconButton(Theme::Glyph::More, t("moreOptions"), 18);
    m_menuButton->setPopupMode(QToolButton::InstantPopup);
    m_menuButton->setMenu(m_overflow);
    headerLayout->addWidget(m_menuButton, 0, Qt::AlignVCenter);
    headerColumn->body()->addLayout(headerLayout);
    headerOuter->addWidget(headerColumn);

    m_updateBar = new QFrame(this);
    m_updateBar->setObjectName(QStringLiteral("UpdateBar"));
    m_updateBar->hide();
    auto *updateLayout = new QHBoxLayout(m_updateBar);
    updateLayout->setContentsMargins(Theme::Spacing::md, Theme::Spacing::sm,
                                     Theme::Spacing::md, Theme::Spacing::sm);
    updateLayout->setSpacing(Theme::Spacing::md);
    auto *updateIcon = new UiKit::GlyphWidget(Theme::Glyph::Download, 20,
                                              Theme::colors().accent, m_updateBar);
    updateLayout->addWidget(updateIcon, 0, Qt::AlignVCenter);
    m_updateText = UiKit::label(QString(), QString(), true);
    updateLayout->addWidget(m_updateText, 1);
    m_updateDownload = UiKit::button(t("download"), QStringLiteral("primary"));
    m_updateDismiss = UiKit::button(t("later"), QStringLiteral("ghost"));
    updateLayout->addWidget(m_updateDownload, 0, Qt::AlignVCenter);
    updateLayout->addWidget(m_updateDismiss, 0, Qt::AlignVCenter);

    auto *updateWrap = new UiKit::ContentColumn(this);
    m_updateBar->setParent(updateWrap);
    updateWrap->body()->addWidget(m_updateBar);

    m_nav = new UiKit::NavTabs(this);
    m_nav->addTab(t("tabConnect"), Theme::Glyph::Phone, m_connect);
    m_nav->addTab(t("tabDisplay"), Theme::Glyph::Monitor, m_display);
    m_nav->addTab(t("tabDoctor"), Theme::Glyph::Doctor, m_doctor);

    auto *body = new QWidget(this);
    auto *bodyOuter = new QVBoxLayout(body);
    bodyOuter->setContentsMargins(0, Theme::Spacing::md, 0, 0);
    bodyOuter->setSpacing(0);
    bodyOuter->addWidget(m_nav, 1);

    auto *central = new QWidget(this);
    central->setObjectName(QStringLiteral("Central"));
    auto *centralLayout = new QVBoxLayout(central);
    centralLayout->setContentsMargins(0, 0, 0, 0);
    centralLayout->setSpacing(0);
    centralLayout->addWidget(header);
    centralLayout->addWidget(updateWrap);
    centralLayout->addWidget(body, 1);
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
            [this]() { m_nav->setCurrentIndex(0); });
    m_viewDisplay = new QAction(this);
    connect(m_viewDisplay, &QAction::triggered, this,
            [this]() { m_nav->setCurrentIndex(1); });
    m_viewDoctor = new QAction(this);
    connect(m_viewDoctor, &QAction::triggered, this,
            [this]() { m_nav->setCurrentIndex(2); });
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
    m_resolutionLabel = statusLabel(QStringLiteral("StatusText"));
    m_portsLabel = statusLabel(QStringLiteral("StatusText"));
    m_clientsLabel = statusLabel(QStringLiteral("StatusText"));
    m_resolutionLabel->setFont(Theme::tabular(m_resolutionLabel->font()));
    m_portsLabel->setFont(Theme::tabular(m_portsLabel->font()));
    m_clientsLabel->setFont(Theme::tabular(m_clientsLabel->font()));

    // The status bar uses the same centred content column as the header and the cards, so the
    // metrics below line up with the card edges instead of hugging the window border.
    auto *row = new UiKit::ContentColumn(this);
    row->setObjectName(QStringLiteral("StatusBarRow"));
    auto *layout = new QHBoxLayout;
    layout->setContentsMargins(0, 0, 0, 0);
    layout->setSpacing(Theme::Spacing::md);
    layout->addStretch(1);
    layout->addWidget(m_resolutionLabel, 0, Qt::AlignVCenter);
    layout->addWidget(UiKit::separator(true), 0, Qt::AlignVCenter);
    layout->addWidget(m_portsLabel, 0, Qt::AlignVCenter);
    layout->addWidget(UiKit::separator(true), 0, Qt::AlignVCenter);
    layout->addWidget(m_clientsLabel, 0, Qt::AlignVCenter);
    row->body()->addLayout(layout);

    statusBar()->setSizeGripEnabled(false);
    statusBar()->setFixedHeight(Theme::Metrics::statusBarHeight + 8);
    // QStatusBar adds its own frame margins, which pushed the metrics past the card edge; the
    // content column already provides the gutter.
    statusBar()->setContentsMargins(0, 0, 0, 0);
    statusBar()->addPermanentWidget(row, 1);
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
    connect(m_nav, &UiKit::NavTabs::currentChanged, this, [this](int index) {
        Q_UNUSED(index);
        m_nav->refresh();
    });

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
    m_updateDownload->setText(t("download"));
    m_updateDismiss->setText(t("later"));
    m_menuButton->setToolTip(t("moreOptions"));

    m_nav->setTabText(0, t("tabConnect"));
    m_nav->setTabText(1, t("tabDisplay"));
    m_nav->setTabText(2, t("tabDoctor"));

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
    const QString resolution = QStringLiteral("%1 × %2 @ %3 Hz")
                                   .arg(m_status.report.displayWidth)
                                   .arg(m_status.report.displayHeight)
                                   .arg(m_status.report.displayFps);
    m_resolutionLabel->setText(resolution);
    m_portsLabel->setText(QStringLiteral("TCP %1 · UDP %2")
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
        return t("streaming");
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
    m_lastHeaderActionKey = -1;
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
    m_lastHeaderActionKey = -1;
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
    m_lastHeaderActionKey = -1;
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
    m_lastHeaderActionKey = -1;
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
