export type Locale = "tr" | "en" | "de";

export interface Translations {
  meta: {
    title: string;
    description: string;
    keywords: string[];
  };
  nav: {
    why: string;
    formats: string;
    metrics: string;
    install: string;
    faq: string;
    github: string;
    installNow: string;
    openMenu: string;
    mobileMenu: string;
    skipLink: string;
  };
  hero: {
    seoH1: string;
    displayTitle1: string;
    displayTitle2: string;
    valueProposition: string;
    oneLinerLabel: string;
    oneLinerCommand: string;
    copied: string;
    copy: string;
    exploreBtn: string;
    installBtn: string;
    allInstallOptions: string;
    supportedManagersTitle: string;
    supportedDesktops: string[];
    offlineBadge: string;
    openSourceBadge: string;
    interactiveBadge: string;
  };
  demo: {
    step1: string;
    step2: string;
    step3: string;
    windowTitle: string;
    pathUser: string;
    pathFolder: string;
    colName: string;
    colSize: string;
    contextMenuLabel: string;
    notificationTitle: string;
    notificationSuccess: string;
    notificationDetail: string;
    replay: string;
    statusIdle: string;
    statusConverting: string;
    statusDone: string;
    reset: string;
    localDisk: string;
    hint: string;
    interactiveHint: string;
  };
  comparison: {
    badge: string;
    title: string;
    subtitle: string;
    oldWayTitle: string;
    oldWaySubtitle: string;
    oldWayTag: string;
    oldWayPoints: string[];
    oldWayResult: string;
    newWayTitle: string;
    newWaySubtitle: string;
    newWayPoints: string[];
  };
  workflow: {
    folder: string;
    fileCount1: string;
    fileCount2: string;
    sourceTag: string;
    newTag: string;
    converting: string;
    doneToastTitle: string;
    doneToastDetail: string;
    menuOpen: string;
    menuCopy: string;
    bottomStatus: string;
    replay: string;
    summaryTitle: string;
    summaryItems: string[];
  };
  trustMetrics: {
    badge: string;
    title: string;
    subtitle: string;
    cards: Array<{
      value: string;
      label: string;
      desc: string;
      tag: string;
    }>;
  };
  formats: {
    badge: string;
    title: string;
    subtitle: string;
    tabs: {
      image: string;
      video: string;
      audio: string;
      doc: string;
    };
    engineTag: string;
    engineDetails: {
      image: string;
      video: string;
      audio: string;
      doc: string;
    };
    profilesTitle: string;
    profilesDesc: string;
    profiles: Array<{
      label: string;
      name: string;
      desc: string;
    }>;
    defaultRayon: string;
    params: Record<string, string>;
  };
  install: {
    badge: string;
    title: string;
    subtitle: string;
    tabs: {
      script: string;
      arch: string;
      debian: string;
      fedora: string;
      uninstall: string;
    };
    comingSoonBadge: string;
    commands: {
      script: string;
      arch: string;
      debian: string;
      fedora: string;
      uninstall: string;
    };
    captions: {
      script: string;
      arch: string;
      debian: string;
      fedora: string;
      uninstall: string;
    };
    integrationTitle: string;
    integrationDesc: string;
    integrationCmd: string;
    viewRepo: string;
    sourceContribTitle: string;
    licenseTag: string;
    sourceContribDesc: string;
  };
  faq: {
    badge: string;
    title: string;
    subtitle: string;
    items: Array<{
      q: string;
      a: string;
    }>;
  };
  footer: {
    tagline: string;
    copyright: string;
    navTitle: string;
    devTitle: string;
    standards: string;
    chips: string[];
    links: Array<{
      href: string;
      label: string;
    }>;
  };
  mobileCta: {
    tagline: string;
    download: string;
    dismiss: string;
  };
}
