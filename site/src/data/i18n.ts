// ─────────────────────────────────────────────
// Orbiscreen - Interface strings (EN / AR)
// https://github.com/shadow-x78/orbiscreen
// ─────────────────────────────────────────────

import type { Lang } from '../lib/site';

export interface UiStrings {
  nav: {
    features: string;
    install: string;
    docs: string;
    compare: string;
    faq: string;
    changelog: string;
    github: string;
    cta: string;
    menu: string;
    close: string;
    language: string;
    primary: string;
    dialog: string;
  };
  actions: {
    readDocs: string;
    androidApk: string;
    copy: string;
    copied: string;
    copyFailed: string;
    backToTop: string;
  };
  footer: {
    tagline: string;
    product: string;
    docs: string;
    community: string;
    legal: string;
    copyright: string;
    features: string;
    setup: string;
    releases: string;
    changelog: string;
    architecture: string;
    udp: string;
    deSupport: string;
    troubleshooting: string;
    github: string;
    issues: string;
    contributing: string;
    license: string;
    security: string;
    conduct: string;
    privacy: string;
    compare: string;
    faq: string;
  };
  labels: {
    breadcrumbHome: string;
    breadcrumb: string;
    docsNav: string;
    docs: string;
    onThisPage: string;
    previous: string;
    next: string;
    pagination: string;
    updated: string;
    source: string;
    readingTime: string;
    minutes: string;
    allDocs: string;
  };
}

export const ui: Record<Lang, UiStrings> = {
  en: {
    nav: {
      features: 'Features',
      install: 'Setup',
      docs: 'Docs',
      compare: 'Compare',
      faq: 'FAQ',
      changelog: 'Changelog',
      github: 'GitHub',
      cta: 'Install Orbiscreen',
      menu: 'Open menu',
      close: 'Close menu',
      language: 'Language',
      primary: 'Main navigation',
      dialog: 'Navigation menu',
    },
    actions: {
      readDocs: 'Read the docs',
      androidApk: 'Android APK',
      copy: 'Copy',
      copied: 'Copied',
      copyFailed: 'Copy failed',
      backToTop: 'Back to top',
    },
    footer: {
      tagline:
        'Your tablet is your second monitor. Open source under GPL-3.0, written in Rust.',
      product: 'Product',
      docs: 'Docs',
      community: 'Community',
      legal: 'Legal',
      copyright: '© 2026 Orbiscreen · Built by shadow-x78 · GPL-3.0',
      features: 'Features',
      setup: 'Setup',
      releases: 'Releases',
      changelog: 'Changelog',
      architecture: 'Architecture',
      udp: 'UDP transport',
      deSupport: 'DE support',
      troubleshooting: 'Troubleshooting',
      github: 'GitHub',
      issues: 'Issues',
      contributing: 'Contributing',
      license: 'GPL-3.0',
      security: 'Security',
      conduct: 'Code of Conduct',
      privacy: 'Privacy',
      compare: 'Compare',
      faq: 'FAQ',
    },
    labels: {
      breadcrumbHome: 'Home',
      breadcrumb: 'Breadcrumb',
      docsNav: 'Documentation navigation',
      docs: 'Docs',
      onThisPage: 'On this page',
      previous: 'Previous',
      next: 'Next',
      pagination: 'Documentation pagination',
      updated: 'Updated',
      source: 'Source',
      readingTime: 'Read time',
      minutes: 'min',
      allDocs: 'All documentation',
    },
  },
  ar: {
    nav: {
      features: 'المزايا',
      install: 'التثبيت',
      docs: 'التوثيق',
      compare: 'المقارنة',
      faq: 'الأسئلة',
      changelog: 'سجل التغييرات',
      github: 'GitHub',
      cta: 'تثبيت Orbiscreen',
      menu: 'افتح القائمة',
      close: 'أغلق القائمة',
      language: 'اللغة',
      primary: 'التنقل الرئيسي',
      dialog: 'قائمة التنقل',
    },
    actions: {
      readDocs: 'اقرأ التوثيق',
      androidApk: 'Android APK',
      copy: 'انسخ',
      copied: 'نسخ',
      copyFailed: 'فشل النسخ',
      backToTop: 'إلى الأعلى',
    },
    footer: {
      tagline: 'تابلتك، شاشتك الثانية. مفتوح المصدر بترخيص GPL-3.0، مكتوب بـ Rust.',
      product: 'المنتج',
      docs: 'التوثيق',
      community: 'المجتمع',
      legal: 'قانوني',
      copyright: '© 2026 Orbiscreen · بناه shadow-x78 · GPL-3.0',
      features: 'المزايا',
      setup: 'التثبيت',
      releases: 'الإصدارات',
      changelog: 'سجل التغييرات',
      architecture: 'المعمارية',
      udp: 'نقل UDP',
      deSupport: 'دعم بيئات سطح المكتب',
      troubleshooting: 'حل الأعطال',
      github: 'GitHub',
      issues: 'المشكلات',
      contributing: 'المساهمة',
      license: 'GPL-3.0',
      security: 'الأمان',
      conduct: 'مدونة السلوك',
      privacy: 'الخصوصية',
      compare: 'المقارنة',
      faq: 'الأسئلة',
    },
    labels: {
      breadcrumbHome: 'الرئيسية',
      breadcrumb: 'مسار التنقل',
      docsNav: 'التنقل في التوثيق',
      docs: 'التوثيق',
      onThisPage: 'في هذه الصفحة',
      previous: 'السابق',
      next: 'التالي',
      pagination: 'التنقل بين صفحات التوثيق',
      updated: 'حدث',
      source: 'المصدر',
      readingTime: 'زمن القراءة',
      minutes: 'دقيقة',
      allDocs: 'كل التوثيق',
    },
  },
};
