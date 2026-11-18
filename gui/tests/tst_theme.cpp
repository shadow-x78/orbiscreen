// Orbiscreen - Qt theme and icon tests (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#include "theme.h"
#include "uikit.h"

#include <QApplication>
#include <QDir>
#include <QFile>
#include <QImage>
#include <QPainter>
#include <QPixmap>
#include <QTest>

namespace {

// Fraction of the canvas the drawn ink covers, measured on the alpha channel. The mark is a ring
// plus a dot on the ring's path, so this is the diameter of the outer edge.
qreal coverage(const QImage &image)
{
    int minX = image.width();
    int minY = image.height();
    int maxX = -1;
    int maxY = -1;
    for (int y = 0; y < image.height(); ++y) {
        for (int x = 0; x < image.width(); ++x) {
            if (qAlpha(image.pixel(x, y)) > 8) {
                minX = qMin(minX, x);
                minY = qMin(minY, y);
                maxX = qMax(maxX, x);
                maxY = qMax(maxY, y);
            }
        }
    }
    if (maxX < minX || maxY < minY) {
        return 0.0;
    }
    const int longest = qMax(maxX - minX + 1, maxY - minY + 1);
    return qreal(longest) / qMax(1, qMax(image.width(), image.height()));
}

QImage toImage(const QPixmap &pixmap)
{
    return pixmap.toImage().convertToFormat(QImage::Format_ARGB32);
}

}

class ThemeTest : public QObject {
    Q_OBJECT

private slots:
    void glyphsRenderAtEveryScaleAndDpr_data();
    void glyphsRenderAtEveryScaleAndDpr();
    void paintedLogoKeepsItsOpticalBox_data();
    void paintedLogoKeepsItsOpticalBox();
    void logoIconIsVectorAtEverySize();
    void committedIconsAreSquareAndFilled();
    void committedLogoSvgsStayIdentical();
};

void ThemeTest::glyphsRenderAtEveryScaleAndDpr_data()
{
    QTest::addColumn<int>("glyph");
    QTest::addColumn<int>("size");
    QTest::addColumn<qreal>("dpr");

    for (int glyph = 0; glyph < int(Theme::Glyph::Count); ++glyph) {
        for (int size : {16, 20, 24, 32, 48}) {
            for (qreal dpr : {1.0, 1.25, 2.0}) {
                QTest::newRow(qPrintable(QStringLiteral("glyph %1 at %2 dpr %3")
                                             .arg(glyph)
                                             .arg(size)
                                             .arg(dpr)))
                    << glyph << size << dpr;
            }
        }
    }
}

void ThemeTest::glyphsRenderAtEveryScaleAndDpr()
{
    QFETCH(int, glyph);
    QFETCH(int, size);
    QFETCH(qreal, dpr);

    const QPixmap pixmap = Theme::glyphPixmap(Theme::Glyph(glyph), size, Qt::white, dpr);
    QVERIFY2(!pixmap.isNull(), "a glyph must render to a pixmap");
    QCOMPARE(pixmap.devicePixelRatio(), dpr);
    // The engine allocates size*dpr physical pixels, so a hi-DPI request must not be a 1x bitmap.
    QCOMPARE(pixmap.width(), qRound(size * dpr));
    QCOMPARE(pixmap.height(), qRound(size * dpr));

    // Every glyph except the Count sentinel must draw something visible.
    if (Theme::Glyph(glyph) == Theme::Glyph::Count) {
        return;
    }
    QVERIFY2(coverage(toImage(pixmap)) > 0.15, "a glyph must actually paint ink");
}

void ThemeTest::paintedLogoKeepsItsOpticalBox_data()
{
    QTest::addColumn<int>("size");
    QTest::addColumn<bool>("compact");

    for (int size : {16, 24, 44, 64, 128, 256, 512}) {
        QTest::newRow(qPrintable(QStringLiteral("%1px normal").arg(size)))
            << size << false;
        if (size <= 32) {
            QTest::newRow(qPrintable(QStringLiteral("%1px compact").arg(size)))
                << size << true;
        }
    }
}

void ThemeTest::paintedLogoKeepsItsOpticalBox()
{
    QFETCH(int, size);
    QFETCH(bool, compact);

    QImage image(size, size, QImage::Format_ARGB32);
    image.fill(Qt::transparent);
    {
        QPainter painter(&image);
        painter.setRenderHint(QPainter::Antialiasing, true);
        Theme::paintLogo(&painter, QRectF(0, 0, size, size), Qt::white, compact);
    }

    const qreal box = coverage(image);
    // The masters are tuned to 82% (normal) and 84% (compact). Small sizes lose a little to
    // pixel rounding, so the floor is lower there, but the mark must never shrink back to the
    // old ~74% or spill past the canvas edge.
    QVERIFY2(box >= 0.70, qPrintable(QStringLiteral("logo covers %1%").arg(box * 100.0)));
    QVERIFY2(box <= 0.93, qPrintable(QStringLiteral("logo covers %1%").arg(box * 100.0)));
}

void ThemeTest::logoIconIsVectorAtEverySize()
{
    // The tray icon and the notification icon both come from Theme::logoIcon(), which is a
    // QIconEngine and must therefore render the mark itself at whatever size the host asks for
    // (tray panels ask for 16-24, notifications for 32-64).
    const QIcon icon = Theme::logoIcon();
    QVERIFY(!icon.isNull());

    for (int size : {16, 22, 24, 32, 48, 64}) {
        const QPixmap pixmap = icon.pixmap(QSize(size, size));
        QVERIFY2(!pixmap.isNull(), qPrintable(QStringLiteral("no pixmap at %1").arg(size)));
        QCOMPARE(pixmap.width(), size);
        QCOMPARE(pixmap.height(), size);

        const QImage image = toImage(pixmap);
        const qreal box = coverage(image);
        // A vector engine must actually draw the mark, not hand back an empty or
        // placeholder pixmap (which is what a missing icon looks like).
        QVERIFY2(box >= 0.70, qPrintable(QStringLiteral("%1px icon covers %2%")
                                             .arg(size)
                                             .arg(box * 100.0)));
        QVERIFY2(box <= 0.93, qPrintable(QStringLiteral("%1px icon covers %2%")
                                             .arg(size)
                                             .arg(box * 100.0)));
    }
}

void ThemeTest::committedIconsAreSquareAndFilled()
{
    const QDir dir(QStringLiteral(ORBISCREEN_ICON_DIR));
    QVERIFY2(dir.exists(), "the icon directory must be reachable for the test");

    const QList<int> sizes = {16, 22, 24, 32, 48, 64, 128, 256, 512};
    for (int size : sizes) {
        const QString path = dir.filePath(QStringLiteral("%1x%1.png").arg(size));
        QImage image(path);
        QVERIFY2(!image.isNull(), qPrintable(QStringLiteral("missing %1").arg(path)));
        QCOMPARE(image.width(), size);
        QCOMPARE(image.height(), size);

        image = image.convertToFormat(QImage::Format_ARGB32);
        // A committed icon must be a rendering of the mark: ink present and centred, not a
        // blank file and not an off-canvas crop.
        const qreal box = coverage(image);
        QVERIFY2(box >= 0.70, qPrintable(QStringLiteral("%1 covers %2%")
                                             .arg(path)
                                             .arg(box * 100.0)));
        QVERIFY2(box <= 0.93, qPrintable(QStringLiteral("%1 covers %2%")
                                             .arg(path)
                                             .arg(box * 100.0)));
    }
}

void ThemeTest::committedLogoSvgsStayIdentical()
{
    const QString root = QStringLiteral(ORBISCREEN_SOURCE_DIR);
    QFile main(root + QStringLiteral("/data/orbiscreen.svg"));
    QFile mirror(root + QStringLiteral("/assets/logo/orbiscreen-logo.svg"));
    QVERIFY(main.open(QIODevice::ReadOnly));
    QVERIFY(mirror.open(QIODevice::ReadOnly));
    QCOMPARE(main.readAll(), mirror.readAll());
}

QTEST_MAIN(ThemeTest)
#include "tst_theme.moc"
