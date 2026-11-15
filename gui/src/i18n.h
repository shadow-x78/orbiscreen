// Orbiscreen - Qt GUI localization (GPL-3.0-or-later) https://github.com/shadow-x78/orbiscreen

#pragma once

#include <QHash>
#include <QString>
#include <QStringList>

enum class Lang {
    En,
    Ar,
};

class I18n {
public:
    static I18n &instance();

    Lang lang() const { return m_lang; }
    void setLang(Lang lang);
    void load();
    void save() const;

    QString t(const QString &key) const;
    QString format(const QString &key, int count) const;
    QStringList keys() const;
    bool has(Lang lang, const QString &key) const;

private:
    I18n();

    Lang m_lang = Lang::En;
};

inline QString tr(const QString &key)
{
    return I18n::instance().t(key);
}
