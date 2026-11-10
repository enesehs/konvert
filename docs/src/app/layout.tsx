import type { Metadata } from "next";
import "./globals.css";
import { ThemeProvider } from "@/components/theme-provider";
import { I18nProvider } from "@/lib/i18n";

export const metadata: Metadata = {
  title: "Konvert — Native Right-Click File Converter for Linux",
  description:
    "Stop memorizing terminal flags and uploading files to ad-filled websites. Convert 50+ formats offline directly from Dolphin, Nautilus, Nemo, and Thunar with 0 MB idle RAM.",
  keywords: [
    "Konvert",
    "Linux",
    "right click file converter",
    "file converter",
    "Rust",
    "Dolphin",
    "Nautilus",
    "Nemo",
    "Thunar",
    "COSMIC",
    "context menu",
    "WebP",
    "AVIF",
    "JXL",
    "offline",
    "privacy",
    "open source",
  ],
  authors: [{ name: "enesehs" }],
  icons: {
    icon: "/konvert.svg",
    apple: "/konvert.svg",
  },
  openGraph: {
    title: "Konvert — Native Right-Click File Converter for Linux",
    description:
      "Native, lightweight, 100% offline file converter integrated into your Linux file manager's context menu. Zero idle RAM.",
    siteName: "Konvert",
    type: "website",
    locale: "en_US",
    alternateLocale: ["tr_TR", "de_DE"],
  },
  twitter: {
    card: "summary_large_image",
    title: "Konvert — Native Right-Click File Converter for Linux",
    description:
      "Native, lightweight, 100% offline file converter integrated into your Linux file manager's context menu. Zero idle RAM.",
  },
  other: {
    "darkreader-lock": "true",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <meta name="darkreader-lock" content="true" />
        <link
          rel="preload"
          href="/MonaSansVF.woff2"
          as="font"
          type="font/woff2"
          crossOrigin="anonymous"
        />
      </head>
      <body
        suppressHydrationWarning
        className="font-sans antialiased bg-background text-foreground min-h-screen flex flex-col overflow-x-clip"
      >
        <a
          href="#main"
          className="skip-link bg-brand px-4 py-2 text-sm font-medium text-primary-foreground"
        >
          Skip to content
        </a>
        <ThemeProvider>
          <I18nProvider>{children}</I18nProvider>
        </ThemeProvider>
      </body>
    </html>
  );
}
