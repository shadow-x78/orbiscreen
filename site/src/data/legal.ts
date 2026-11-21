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
    title: 'Every release, in order',
    lede: 'The newest releases with their fixes and changes. The full history, back to the first release, is in CHANGELOG.md next to the source.',
    older: 'Looking for an older release?',
    olderBody:
      'The complete changelog covers every version since the first release, including the reasoning behind each change and the corrections to earlier entries.',
    olderCta: 'Read the full changelog',
  },
  ar: {
    eyebrow: 'سجل التغييرات',
    title: 'كل إصدار بالترتيب',
    lede: 'أحدث الإصدارات بإصلاحاتها وتغييراتها. والتاريخ الكامل حتى أول إصدار في CHANGELOG.md بجانب الشيفرة.',
    older: 'تبحث عن إصدار أقدم؟',
    olderBody: 'يغطي السجل الكامل كل إصدار منذ البداية، مع سبب كل تغيير وتصحيحات المدخلات السابقة.',
    olderCta: 'اقرأ السجل الكامل',
  },
};

export const licensePage: Record<Lang, SimplePage> = {
  en: {
    eyebrow: 'License',
    title: 'GPL-3.0-or-later',
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
    ],
  },
  ar: {
    eyebrow: 'الرخصة',
    title: 'GPL-3.0-or-later',
    lede: 'Orbiscreen برنامج حر. يمكنك تشغيله ودراسته ومشاركته وتعديله، بشرط بقاء الأعمال المشتقة تحت الرخصة نفسها.',
    sections: [
      {
        title: 'ما تمنحك الرخصة',
        body: ['تمنحك رخصة GNU العامة الإصدار الثالث أربع حريات:'],
        list: [
          'تشغيل البرنامج لأي غرض.',
          'دراسة طريقة عمله وتغييره مع الوصول إلى المصدر.',
          'إعادة توزيع النسخ، بما فيها الملفات الثنائية المنشورة في الإصدارات.',
          'توزيع نسخ معدّلة بشرط حملها الرخصة نفسها.',
        ],
      },
      {
        title: 'ما تطلبه منك',
        body: [
          'إن وزّعت نسخة معدّلة فعليك إتاحة المصدر المقابل تحت GPL-3.0-or-later والحفاظ على الإشعارات. أما توزيع الملفات الثنائية غير المعدّلة من الإصدارات فلا يحتاج أكثر من نص الرخصة.',
          'يُوزَّع Orbiscreen على أمل أن يكون مفيداً، لكن دون أي ضمان، حتى الضمان الضمني لقابلية التسويق أو الملاءمة لغرض معين.',
        ],
      },
      {
        title: 'مكوّنات مضمّنة',
        body: [
          'مساعد رمز QR في gui/ui/qrcode.js مرخّص بـ MIT ويحتفظ بإشعاره الأصلي. ذلك الإشعار التزام ترخيصي لا توثيق، فيبقى في مكانه داخل شجرة GPL-3.0-or-later.',
        ],
      },
    ],
  },
};

export const securityPage: Record<Lang, SimplePage> = {
  en: {
    eyebrow: 'Security',
    title: 'Report a vulnerability privately',
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
    ],
  },
  ar: {
    eyebrow: 'الأمان',
    title: 'أبلغ عن ثغرة بخصوصية',
    lede: 'يعمل Orbiscreen على شبكة محلية ويحقن الإدخال في سطح مكتبك، لذا تُؤخذ بلاغات الأمان بجدية وتُجاب بخصوصية.',
    sections: [
      {
        title: 'كيف تبلّغ',
        body: [
          'افتح استشارة أمان خاصة على GitHub، فتُبقي التفاصيل بعيدة عن العرض العام حتى يصبح الإصلاح جاهزاً. ويتوفر البريد كبديل إن تعذّر استخدام GitHub.',
        ],
        list: [
          'GitHub: Security ← Advisories ← New draft security advisory',
          `البريد: ${SECURITY_EMAIL}`,
          'أدرج الإصدار المتأثر والمكوّن وخطوات إعادة الإنتاج والأثر الذي لاحظته.',
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
          'لا يتلقى تحديثات الأمان إلا أحدث إصدار ثانوي. ابنِ من main قبل الإبلاغ كي لا تكون المشكلة مُصلحة سلفاً.',
          'كل إصدارات Linux وAndroid موقّعة تشفيرياً، وتُنشر التوقيعات مع الإصدار.',
        ],
      },
    ],
  },
};

export const privacyPage: Record<Lang, SimplePage> = {
  en: {
    eyebrow: 'Privacy',
    title: 'No accounts, no cookies, no tracking',
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
        title: 'Contact',
        body: [`Privacy questions can go to ${SECURITY_EMAIL}.`],
      },
    ],
  },
  ar: {
    eyebrow: 'الخصوصية',
    title: 'بلا حسابات ولا كوكيز ولا تتبع',
    lede: 'هذا الموقع توثيق وتسويق ثابت لأداة تعمل على الشبكة المحلية. لا يضع كوكيز ولا يطلب بيانات شخصية.',
    sections: [
      {
        title: 'ما يجمعه هذا الموقع',
        body: [
          'يقدّم Cloudflare هذا الموقع ويسجّل بيانات طلب مجمّعة مثل المسار المطلوب وحالة الاستجابة والبلد ووكيل المستخدم. ويضيف Cloudflare Web Analytics عدّاً مجمّعاً للمشاهدات بلا كوكيز وبلا معرّفات عبر المواقع وبلا بصمة.',
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
          'تفتح روابط GitHub وLaunchpad وFedora COPR مواقع طرف ثالث لها سياساتها. ولا يضمّن هذا الموقع سكربتات أو خطوطاً أو بكسلات تتبع من طرف ثالث.',
        ],
      },
      {
        title: 'التواصل',
        body: [`يمكن إرسال أسئلة الخصوصية إلى ${SECURITY_EMAIL}.`],
      },
    ],
  },
};
