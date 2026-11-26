// ─────────────────────────────────────────────
// Orbiscreen - Changelog, license, security, privacy
// https://github.com/shadow-x78/orbiscreen
// ─────────────────────────────────────────────

import { SECURITY_EMAIL, type Lang } from '../lib/site';

export interface SimpleSection {
  title: string;
  body: string[];
  list?: string[];
}

export interface SimplePage {
  eyebrow: string;
  title: string;
  lede: string;
  sections: SimpleSection[];
}

export const changelogPage: Record<
  Lang,
  { eyebrow: string; title: string; lede: string; older: string; olderBody: string; olderCta: string }
> = {
  en: {
    eyebrow: 'CHANGELOG',
    title: 'Every release, in order.',
    lede: 'The newest releases with their fixes and changes. The full history, back to the first release, is in CHANGELOG.md next to the source.',
    older: 'Looking for an older release?',
    olderBody:
      'The complete changelog covers every version since the first release, including the reasoning behind each change and the corrections to earlier entries.',
    olderCta: 'Read the full changelog',
  },
  ar: {
    eyebrow: 'سجل التغييرات',
    title: 'كل إصدار بالترتيب.',
    lede: 'أحدث الإصدارات بإصلاحاتها وتغييراتها. والتاريخ الكامل حتى أول إصدار في CHANGELOG.md بجانب الشيفرة.',
    older: 'تبحث عن إصدار أقدم؟',
    olderBody: 'يغطي السجل الكامل كل إصدار منذ البداية، مع سبب كل تغيير وتصحيحات المدخلات السابقة.',
    olderCta: 'اقرأ السجل الكامل',
  },
};

export const licensePage: Record<Lang, SimplePage> = {
  en: {
    eyebrow: 'License',
    title: 'GPL-3.0-or-later.',
    lede: 'Orbiscreen is free software. You can run it, study it, share it, and modify it, provided derivative works stay under the same license.',
    sections: [
      {
        title: 'What the license gives you',
        body: ['The GNU General Public License version 3 grants four freedoms:'],
        list: [
          'Run the program for any purpose.',
          'Study how it works and change it, with access to the source.',
          'Redistribute copies, including the binaries published in the releases.',
          'Distribute modified versions, provided they carry the same license.',
        ],
      },
      {
        title: 'What it asks of you',
        body: [
          'If you distribute a modified version, you must make the corresponding source available under GPL-3.0-or-later and keep the notices intact. Distributing the unmodified binaries from the releases needs no further action beyond the license text.',
          'Orbiscreen is distributed in the hope that it will be useful, but without any warranty, without even the implied warranty of merchantability or fitness for a particular purpose.',
        ],
      },
      {
        title: 'Bundled components',
        body: [
          'The QR code helper in gui/ui/qrcode.js is MIT licensed and keeps its original notice. That notice is a license obligation, not documentation, so it stays in place inside the GPL-3.0-or-later source tree.',
        ],
      },
      {
        title: 'Which license covers which part',
        body: ['Every first-party component is GPL-3.0-or-later unless a file says otherwise:'],
        list: [
          'The Rust workspace in crates/ and the orbiscreen daemon.',
          'The Qt GUI in gui/, except gui/ui/qrcode.js, which is MIT.',
          'The Android client in clients/android/ and the web client in clients/web/.',
          'The site code in site/, and the packaging scripts in scripts/.',
          'Third-party crates and libraries keep their own licenses, recorded in Cargo.lock and in the package metadata.',
        ],
      },
      {
        title: 'Redistributing a build',
        body: [
          'You may redistribute the unmodified AppImage, deb, rpm, tarball, or APK. Keep the license text and the copyright notices, and do not suggest that the project endorses your build.',
          'If you modify the code, publish the corresponding source under GPL-3.0-or-later. The release tag in the artifacts and the source in the repository identify the exact revision.',
        ],
        list: [
          'Ship LICENSE and the notices that come with the artifact.',
          'Link back to the source tag or commit used for the build.',
          'Do not add restrictions that GPL-3.0 forbids.',
          'State the license on any store or download page.',
        ],
      },
      {
        title: 'Commercial use',
        body: [
          'GPL-3.0 does not forbid selling Orbiscreen, bundling it with hardware, or offering paid support. It only requires that people who receive the software also receive the same freedoms.',
          'You do not need a separate commercial license from the project. If your organization cannot accept the GPL-3.0 obligations, use Orbiscreen internally instead of distributing it, or contact the maintainer to discuss a separate arrangement.',
        ],
      },
      {
        title: 'Reporting a licensing problem',
        body: [
          'If a file is missing a notice, or a bundled component is under a different license than this page says, open an issue with the path and the license you expected. The maintainer will correct the file or this page.',
        ],
      },
    ],
  },
  ar: {
    eyebrow: 'الرخصة',
    title: 'GPL-3.0-or-later.',
    lede: 'Orbiscreen برنامج حر. يمكنك تشغيله ودراسته ومشاركته وتعديله، بشرط بقاء الأعمال المشتقة تحت الرخصة نفسها.',
    sections: [
      {
        title: 'ما تمنحك الرخصة',
        body: ['تمنحك رخصة GNU العامة الإصدار الثالث أربع حريات:'],
        list: [
          'تشغيل البرنامج لأي غرض.',
          'دراسة طريقة عمله وتغييره مع الوصول إلى المصدر.',
          'إعادة توزيع النسخ، بما فيها الملفات الثنائية المنشورة في الإصدارات.',
          'توزيع نسخ معدلة بشرط حملها الرخصة نفسها.',
        ],
      },
      {
        title: 'ما تطلبه منك',
        body: [
          'إن وزعت نسخة معدلة فعليك إتاحة المصدر المقابل تحت GPL-3.0-or-later والحفاظ على الإشعارات. أما توزيع الملفات الثنائية غير المعدلة من الإصدارات فلا يحتاج أكثر من نص الرخصة.',
          'يوزع Orbiscreen على أمل أن يكون مفيدا، لكن دون أي ضمان، حتى الضمان الضمني لقابلية التسويق أو الملاءمة لغرض معين.',
        ],
      },
      {
        title: 'مكونات مضمنة',
        body: [
          'مساعد رمز QR في gui/ui/qrcode.js مرخص بـ MIT ويحتفظ بإشعاره الأصلي. ذلك الإشعار التزام ترخيصي لا توثيق، فيبقى في مكانه داخل شجرة GPL-3.0-or-later.',
        ],
      },
      {
        title: 'أي جزء تغطيه أي رخصة',
        body: ['كل مكون من مكونات المشروع تحت GPL-3.0-or-later ما لم يذكر الملف غير ذلك:'],
        list: [
          'مساحة عمل Rust في crates/ وخادم orbiscreen.',
          'واجهة Qt في gui/، باستثناء gui/ui/qrcode.js المرخص بـ MIT.',
          'عميل Android في clients/android/ وعميل الويب في clients/web/.',
          'شيفرة الموقع في site/، وسكربتات التغليف في scripts/.',
          'تحتفظ حزم Rust والمكتبات الخارجية برخصها، وهي مسجلة في Cargo.lock وفي بيانات الحزم.',
        ],
      },
      {
        title: 'إعادة توزيع نسخة مبنية',
        body: [
          'يمكنك إعادة توزيع AppImage أو deb أو rpm أو tarball أو APK دون تعديل. حافظ على نص الرخصة وإشعارات حقوق النشر، ولا توحِ بأن المشروع يتبنى نسختك.',
          'إذا عدلت الشيفرة فعليك نشر المصدر المقابل تحت GPL-3.0-or-later. وسم الإصدار في الملفات والمصدر في المستودع يحددان المراجعة بدقة.',
        ],
        list: [
          'أرفق LICENSE والإشعارات التي تأتي مع الملف.',
          'اربط بوسم المصدر أو الالتزام المستخدم في البناء.',
          'لا تضف قيودا تحظرها GPL-3.0.',
          'اذكر الرخصة في أي صفحة متجر أو تحميل.',
        ],
      },
      {
        title: 'الاستخدام التجاري',
        body: [
          'لا تمنع GPL-3.0 بيع Orbiscreen أو ضمه إلى عتاد أو تقديم دعم مدفوع. كل ما تطلبه أن يحصل متلقو البرنامج على الحريات نفسها.',
          'لا تحتاج رخصة تجارية منفصلة من المشروع. وإذا تعذر على مؤسستك قبول التزامات GPL-3.0 فاستخدم Orbiscreen داخليا بدل توزيعه، أو تواصل مع المشرف لمناقشة ترتيب منفصل.',
        ],
      },
      {
        title: 'الإبلاغ عن مشكلة ترخيص',
        body: [
          'إذا كان ملف يفتقد إشعارا، أو كان مكون مضمّن تحت رخصة مختلفة عما تقوله هذه الصفحة، فافتح بلاغا بالمسار والرخصة المتوقعة. سيصحح المشرف الملف أو هذه الصفحة.',
        ],
      },
    ],
  },
};

export const securityPage: Record<Lang, SimplePage> = {
  en: {
    eyebrow: 'Security',
    title: 'Report a vulnerability privately.',
    lede: 'Orbiscreen runs on a local network and injects input into your desktop, so security reports are taken seriously and answered privately.',
    sections: [
      {
        title: 'How to report',
        body: [
          'Open a private security advisory on GitHub, which keeps the details out of public view until a fix is ready. Email is available as a fallback if you cannot use GitHub.',
        ],
        list: [
          'GitHub: Security → Advisories → New draft security advisory',
          `Email: ${SECURITY_EMAIL}`,
          'Include the affected version, the component, reproduction steps, and the impact you observed.',
        ],
      },
      {
        title: 'What is in scope',
        body: [
          'The host daemon, the transport layer, the input injection path, the Android client, the web client, the packaging scripts, and the release workflow.',
        ],
        list: [
          'Authentication bypass on the HTTP, UDP, or WebTransport paths',
          'Session ownership errors that let one client attach to another client\'s stream',
          'Input injection outside the virtual display or across sessions',
          'Privilege escalation through packaging or the EVDI helper',
          'Secrets exposure in build artifacts or workflow logs',
        ],
      },
      {
        title: 'Supported versions',
        body: [
          'Only the latest minor release receives security updates. Build from main before reporting, so the issue is not already fixed.',
          'Every Linux and Android release artifact is cryptographically signed, and the signatures are published with the release.',
        ],
      },
      {
        title: 'What happens after you report',
        body: ['Reports are handled privately until a fix and a release are ready. The usual sequence is:'],
        list: [
          'The maintainer acknowledges the report and confirms the affected versions.',
          'Reproduction and scope confirmation with you.',
          'A private fix, a signed release, and a GitHub advisory.',
          'Coordinated disclosure on a date agreed with you, with credit under the name you choose.',
        ],
      },
      {
        title: 'Handling and credit',
        body: [
          'Reports stay private until a fix and a release are ready. The advisory credits the reporter under the name they choose, unless they ask to stay anonymous. Please give the maintainer a reasonable window to ship a fix before publishing details.',
        ],
      },
      {
        title: 'Hardening checklist',
        body: ['Orbiscreen listens on the local network and authenticates clients with a token. These defaults reduce exposure:'],
        list: [
          'Keep the daemon on a trusted LAN; do not expose the signaling port to the internet.',
          'Pair clients explicitly and revoke them when a device is lost.',
          'Prefer per-session tokens over the legacy shared token for browser clients.',
          'Run the daemon as your user; root is only needed for EVDI on some desktops.',
          'Keep the host and the Android client on current releases.',
        ],
      },
      {
        title: 'Verifying a release',
        body: ['Every release publishes SHA-256 sums and GitHub build attestations. Check an artifact before running it:'],
        list: [
          'sha256sum -c orbiscreen-x86_64.AppImage.sha256',
          'gh attestation verify orbiscreen-x86_64.AppImage -R shadow-x78/orbiscreen',
          'Compare the tag in the release notes with the source you audit.',
        ],
      },
      {
        title: 'Out of scope',
        body: [
          'Reports that require physical access, a compromised host, a malicious compositor, or a browser zero-day are out of scope. So are missing headers on third-party mirrors and issues in dependencies that are already fixed upstream.',
        ],
      },
    ],
  },
  ar: {
    eyebrow: 'الأمان',
    title: 'أبلغ عن ثغرة بخصوصية.',
    lede: 'يعمل Orbiscreen على شبكة محلية ويحقن الإدخال في سطح مكتبك، لذا تؤخذ بلاغات الأمان بجدية وتجاب بخصوصية.',
    sections: [
      {
        title: 'كيف تبلغ',
        body: [
          'افتح استشارة أمان خاصة على GitHub، فتبقي التفاصيل بعيدة عن العرض العام حتى يصبح الإصلاح جاهزا. ويتوفر البريد كبديل إن تعذر استخدام GitHub.',
        ],
        list: [
          'GitHub: Security ← Advisories ← New draft security advisory',
          `البريد: ${SECURITY_EMAIL}`,
          'أدرج الإصدار المتأثر والمكون وخطوات إعادة الإنتاج والأثر الذي لاحظته.',
        ],
      },
      {
        title: 'ما يشمله النطاق',
        body: [
          'خادم المضيف وطبقة النقل ومسار حقن الإدخال وعميل Android وعميل الويب وسكربتات التغليف ومسار الإصدار.',
        ],
        list: [
          'تجاوز المصادقة على مسارات HTTP أو UDP أو WebTransport',
          'أخطاء ملكية الجلسة التي تسمح لعميل بالارتباط ببث عميل آخر',
          'حقن إدخال خارج الشاشة الافتراضية أو عبر الجلسات',
          'تصعيد صلاحيات عبر التغليف أو مساعد EVDI',
          'تسريب أسرار في مخرجات البناء أو سجلات مسار العمل',
        ],
      },
      {
        title: 'الإصدارات المدعومة',
        body: [
          'لا يتلقى تحديثات الأمان إلا أحدث إصدار ثانوي. ابن من main قبل الإبلاغ كي لا تكون المشكلة مصلحة سلفا.',
          'كل إصدارات Linux وAndroid موقعة تشفيريا، وتنشر التوقيعات مع الإصدار.',
        ],
      },
      {
        title: 'ما يحدث بعد الإبلاغ',
        body: ['تدار البلاغات بخصوصية حتى يصبح الإصلاح والإصدار جاهزين. والتسلسل المعتاد:'],
        list: [
          'يقر المشرف بالبلاغ ويؤكد الإصدارات المتأثرة.',
          'إعادة إنتاج وتأكيد النطاق معك.',
          'إصلاح خاص وإصدار موقّع واستشارة على GitHub.',
          'كشف منسق في موعد تتفقان عليه، والفضل باسم تختاره.',
        ],
      },
      {
        title: 'التعامل والفضل',
        body: [
          'تبقى البلاغات خاصة حتى يصبح الإصلاح والإصدار جاهزين. وتنسب الاستشارة الفضل للباحث بالاسم الذي يختاره، ما لم يطلب البقاء مجهولا. يرجى منح المشرف وقتا معقولا لإصدار الإصلاح قبل نشر التفاصيل.',
        ],
      },
      {
        title: 'قائمة تقوية',
        body: ['يستمع Orbiscreen على الشبكة المحلية ويوثق العملاء برمز. هذه الإعدادات تقلل التعرض:'],
        list: [
          'أبق الخادم على شبكة محلية موثوقة، ولا تعرض منفذ الإشارات إلى الإنترنت.',
          'اقرن العملاء صراحة وألغ اقترانهم عند فقد الجهاز.',
          'فضل رموز الجلسات على الرمز المشترك القديم لعملاء المتصفح.',
          'شغّل الخادم بمستخدمك؛ ولا تحتاج root إلا لـ EVDI على بعض البيئات.',
          'أبق المضيف وعميل Android على أحدث الإصدارات.',
        ],
      },
      {
        title: 'التحقق من الإصدار',
        body: ['ينشر كل إصدار مجاميع SHA-256 وإثباتات بناء GitHub. تحقق من الملف قبل تشغيله:'],
        list: [
          'sha256sum -c orbiscreen-x86_64.AppImage.sha256',
          'gh attestation verify orbiscreen-x86_64.AppImage -R shadow-x78/orbiscreen',
          'قارن الوسم في ملاحظات الإصدار بالمصدر الذي تدققه.',
        ],
      },
      {
        title: 'خارج النطاق',
        body: [
          'البلاغات التي تتطلب وصولا ماديا أو مضيفا مخترقا أو مدير نوافذ خبيثا أو ثغرة يوم صفر في المتصفح خارج النطاق. وكذلك الترويسات الناقصة على مرايا طرف ثالث والمشكلات في اعتماديات أصلحتها المنابع.',
        ],
      },
    ],
  },
};

export const privacyPage: Record<Lang, SimplePage> = {
  en: {
    eyebrow: 'Privacy',
    title: 'No accounts, no cookies, no tracking.',
    lede: 'This site is static documentation and marketing for a local-network tool. It sets no cookies and asks for no personal data.',
    sections: [
      {
        title: 'What this site collects',
        body: [
          'Cloudflare serves this site and records aggregate request data such as the requested path, the response status, the country, and the user agent. Cloudflare Web Analytics adds aggregate page-view counts without cookies, without cross-site identifiers, and without fingerprinting.',
          'There are no accounts, no forms, no newsletter, and no comment system. Nothing on this site asks you to identify yourself.',
        ],
      },
      {
        title: 'What the application collects',
        body: [
          'The Orbiscreen daemon does not send telemetry. Video and input stay between your host and your client on your local network.',
          'The update check is a single request to the release endpoint, and it can be disabled with ORBISCREEN_NO_UPDATE_CHECK=1.',
        ],
      },
      {
        title: 'Outbound links',
        body: [
          'Links to GitHub, Launchpad, and the Fedora COPR open third-party sites with their own policies. This site does not embed third-party scripts, fonts, or tracking pixels.',
        ],
      },
      {
        title: 'Data flow at a glance',
        body: ['Three paths are involved, and none of them sends your screen or input to the project:'],
        list: [
          'Browser to the Cloudflare edge to a static asset from Cloudflare Pages. Cloudflare records the request metadata described above.',
          'Host to your LAN to the Android or browser client. No project server is involved.',
          'Host to the GitHub releases endpoint, only for the update check, unless you disable it.',
        ],
      },
      {
        title: 'Cloudflare Analytics and retention',
        body: [
          'Cloudflare Web Analytics reports aggregate counts such as page views, referrers, countries, and browsers without cookies or cross-site identifiers. The project sees only the aggregate dashboard; raw edge logs are retained by Cloudflare according to the account settings.',
        ],
      },
      {
        title: 'Android client permissions',
        body: [
          'The Android client asks for network access for streaming and LAN discovery. It also declares the install-packages permission for an update APK you choose yourself, and the battery-optimization exemption so a session can stay alive when you grant it. It does not request contacts, location, camera, microphone, storage, or notification access.',
        ],
      },
      {
        title: 'How to verify these claims',
        body: ['You do not have to trust this page:'],
        list: [
          'Open DevTools, then Network, on any page; the only third-party requests are Cloudflare\u2019s analytics script and the edge itself.',
          'Run ss -tunp while the daemon is idle; no connection leaves your LAN except the update check.',
          'Set ORBISCREEN_NO_UPDATE_CHECK=1 and confirm the host makes no outbound request.',
          'Inspect the Android manifest for the permission list.',
        ],
      },
      {
        title: 'Changes to this policy',
        body: [
          'This policy changes only when the site or the client changes what it collects. The changelog records the change, and this page keeps the current text.',
        ],
      },
      {
        title: 'Contact',
        body: [`Privacy questions can go to ${SECURITY_EMAIL}.`],
      },
    ],
  },
  ar: {
    eyebrow: 'الخصوصية',
    title: 'بلا حسابات ولا كوكيز ولا تتبع.',
    lede: 'هذا الموقع توثيق وتسويق ثابت لأداة تعمل على الشبكة المحلية. لا يضع كوكيز ولا يطلب بيانات شخصية.',
    sections: [
      {
        title: 'ما يجمعه هذا الموقع',
        body: [
          'يقدم Cloudflare هذا الموقع ويسجل بيانات طلب مجمعة مثل المسار المطلوب وحالة الاستجابة والبلد ووكيل المستخدم. ويضيف Cloudflare Web Analytics عدا مجمعا للمشاهدات بلا كوكيز وبلا معرفات عبر المواقع وبلا بصمة.',
          'لا حسابات ولا نماذج ولا نشرة بريدية ولا تعليقات. لا شيء في هذا الموقع يطلب منك تعريف نفسك.',
        ],
      },
      {
        title: 'ما يجمعه التطبيق',
        body: [
          'لا يرسل خادم Orbiscreen أي قياسات. يبقى الفيديو والإدخال بين مضيفك وعميلك على شبكتك المحلية.',
          'فحص التحديث طلب واحد إلى نقطة الإصدارات، ويمكن تعطيله بـ ORBISCREEN_NO_UPDATE_CHECK=1.',
        ],
      },
      {
        title: 'الروابط الخارجية',
        body: [
          'تفتح روابط GitHub وLaunchpad وFedora COPR مواقع طرف ثالث لها سياساتها. ولا يضمن هذا الموقع سكربتات أو خطوطا أو بكسلات تتبع من طرف ثالث.',
        ],
      },
      {
        title: 'مسار البيانات باختصار',
        body: ['هناك ثلاثة مسارات، ولا يرسل أي منها شاشتك أو إدخالك إلى المشروع:'],
        list: [
          'المتصفح إلى حافة Cloudflare إلى ملف ثابت من Cloudflare Pages. وتسجل Cloudflare بيانات الطلب المذكورة أعلاه.',
          'المضيف إلى شبكتك المحلية إلى عميل Android أو المتصفح. ولا يشارك أي خادم تابع للمشروع.',
          'المضيف إلى نقطة إصدارات GitHub، لفحص التحديث فقط، ما لم تعطله.',
        ],
      },
      {
        title: 'تحليلات Cloudflare والاحتفاظ',
        body: [
          'تعرض Cloudflare Web Analytics عدا مجمعة مثل المشاهدات والمصادر والبلدان والمتصفحات بلا كوكيز وبلا معرفات عبر المواقع. ولا يرى المشروع إلا لوحة العدا المجمعة؛ وتحتفظ Cloudflare بسجلات الحافة الخام حسب إعدادات الحساب.',
        ],
      },
      {
        title: 'أذونات عميل Android',
        body: [
          'يطلب عميل Android الوصول إلى الشبكة للبث واكتشاف الشبكة المحلية. ويعلن أيضا إذن تثبيت الحزم لتحديث APK تختاره بنفسك، وإعفاء تحسين البطارية لإبقاء الجلسة حية عند منحه. ولا يطلب جهات الاتصال أو الموقع أو الكاميرا أو الميكروفون أو التخزين أو الإشعارات.',
        ],
      },
      {
        title: 'كيف تتحقق من هذه الادعاءات',
        body: ['لا تحتاج إلى الوثوق بهذه الصفحة:'],
        list: [
          'افتح أدوات المطور ثم Network في أي صفحة؛ الطلبات الخارجية الوحيدة هي سكربت تحليلات Cloudflare والحافة نفسها.',
          'شغّل ss -tunp والخادم خامل؛ لا يغادر اتصال شبكتك المحلية إلا فحص التحديث.',
          'عيّن ORBISCREEN_NO_UPDATE_CHECK=1 وتأكد أن المضيف لا يرسل طلبا خارجيا.',
          'افحص بيان Android للحصول على قائمة الأذونات.',
        ],
      },
      {
        title: 'تغييرات هذه السياسة',
        body: [
          'تتغير هذه السياسة فقط عندما يغيّر الموقع أو العميل ما يجمعه. ويسجل سجل التغييرات التعديل، وتحتفظ الصفحة بالنص الحالي.',
        ],
      },
      {
        title: 'التواصل',
        body: [`يمكن إرسال أسئلة الخصوصية إلى ${SECURITY_EMAIL}.`],
      },
    ],
  },
};
