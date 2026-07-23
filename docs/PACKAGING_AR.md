# دليل الحزم والتوزيع لمختلف التوزيعات - Orbiscreen

---

## 🌐 اللغة

<a href="PACKAGING.md">🇬🇧 English</a> · <a href="PACKAGING_AR.md">🇸🇦 العربية</a>

---

## 📦 نظرة عامة

يوفر Orbiscreen إعدادات وبناء وتوزيع لكافة توزيعات اللينكس الرئيسية ونظام أندرويد:

- **AppImage:** حزمة عالمية تعمل على أي توزيعة لينكس فوراً بدون تثبيت.
- **Flatpak:** حزمة معزولة متوافقة مع متجر Flathub.
- **Debian / Ubuntu (.deb):** حزمة ديبيان الأصلية لتوزيعات أوبونتو وديبيان ومينت.
- **Fedora / RHEL (.rpm):** حزمة RPM الأصلية لتوزيعات فيدورا وريدهات وسوزي.
- **الأرشيف المستقل (.tar.gz):** ملفات تنفيذية جاهزة مع سكربت التثبيت التلقائي بنقرة واحدة.
- **تطبيق الأندرويد (.apk):** تطبيق العميل للأجهزة اللوحية والهواتف الذكية.

---

## 🔨 بناء الحزم محلياً

### 1. الأرشيف المستقل وسكربت التثبيت التلقائي
```bash
cargo build --release --workspace
./scripts/install.sh
```

### 2. حزمة ديبيان وأوبونتو (`.deb`)
```bash
cargo install cargo-deb
cargo deb -p orbiscreen-daemon
```

### 3. حزمة فيدورا وريدهات (`.rpm`)
```bash
cargo install cargo-generate-rpm
cargo generate-rpm -p orbiscreen-daemon
```

### 4. تطبيق الأندرويد (`app-debug.apk`)
```bash
cd clients/android
./gradlew assembleDebug
```
مسار ملف الـ APK الناتج: `clients/android/app/build/outputs/apk/debug/app-debug.apk`

---

## 🚀 النشر التلقائي في GitHub Releases

عند رفع تاغ إصدار جديد (مثل `git tag v0.4.2 && git push origin v0.4.2`)، يشتغل أكشن `.github/workflows/release.yml` تلقائياً لتجميع كافة الحزم ورفعها مباشرة في صفحة الإصدارات **GitHub Releases**.
