"use client";

import * as React from "react";
import {
  FileAudio,
  FileImage,
  FileText,
  FileVideo,
  ArrowRight,
  Cpu,
  Layers,
} from "lucide-react";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Container, Section } from "./section";
import { Reveal } from "./reveal";
import { useI18n } from "@/lib/i18n";

type CategoryKey = "image" | "video" | "audio" | "doc";

export function Formats() {
  const { t } = useI18n();

  const categories: Array<{
    id: CategoryKey;
    icon: React.ComponentType<{ className?: string }>;
    engineTitle: string;
    engineDetails: string;
    conversions: Array<{
      from: string;
      to: string;
      engine: string;
      speed: string;
      param: string;
    }>;
  }> = [
    {
      id: "image",
      icon: FileImage,
      engineTitle: "Rust image-rs & resvg",
      engineDetails: t.formats.engineDetails.image,
      conversions: [
        { from: "WEBP", to: "PNG", engine: "image-rs", speed: "~14ms", param: t.formats.params.losslessRgba },
        { from: "PNG", to: "WEBP", engine: "libwebp", speed: "~22ms", param: t.formats.params.webOptimize },
        { from: "JPG", to: "WEBP", engine: "libwebp", speed: "~18ms", param: t.formats.params.saving60 },
        { from: "PNG", to: "JPG", engine: "mozjpeg", speed: "~16ms", param: t.formats.params.exifStrip },
        { from: "SVG", to: "PNG", engine: "resvg", speed: "~11ms", param: t.formats.params.dpiRender },
        { from: "HEIC", to: "JPG", engine: "libheif", speed: "~45ms", param: t.formats.params.appleDecode },
        { from: "AVIF", to: "PNG", engine: "libdav1d", speed: "~38ms", param: t.formats.params.hdrDecode },
        { from: "GIF", to: "WEBP", engine: "image-rs", speed: "~32ms", param: t.formats.params.animatedFrames },
        { from: "ICO", to: "PNG", engine: "image-rs", speed: "~6ms", param: t.formats.params.extractMultiRes },
        { from: "BMP", to: "PNG", engine: "image-rs", speed: "~9ms", param: t.formats.params.losslessCompress },
      ],
    },
    {
      id: "video",
      icon: FileVideo,
      engineTitle: "FFmpeg & libavcodec",
      engineDetails: t.formats.engineDetails.video,
      conversions: [
        { from: "MP4", to: "WEBM", engine: "libvpx-vp9", speed: "~1.2s", param: t.formats.params.crf30Html5 },
        { from: "MKV", to: "MP4", engine: "remux / copy", speed: "~180ms", param: t.formats.params.losslessStreamCopy },
        { from: "MOV", to: "MP4", engine: "libx264", speed: "~850ms", param: t.formats.params.universalH264 },
        { from: "MP4", to: "GIF", engine: "palettegen", speed: "~620ms", param: t.formats.params.fpsLoop },
        { from: "AVI", to: "MP4", engine: "libx264", speed: "~950ms", param: t.formats.params.modernMp4Container },
        { from: "WEBM", to: "MP4", engine: "libx264", speed: "~780ms", param: t.formats.params.h264Baseline },
        { from: "MP4", to: "MP3", engine: "libmp3lame", speed: "~95ms", param: t.formats.params.extractAudio320 },
        { from: "MKV", to: "WEBM", engine: "librav1e", speed: "~1.8s", param: t.formats.params.av1Opus },
      ],
    },
    {
      id: "audio",
      icon: FileAudio,
      engineTitle: "FFmpeg & libopus / LAME",
      engineDetails: t.formats.engineDetails.audio,
      conversions: [
        { from: "MP4", to: "MP3", engine: "libmp3lame", speed: "~110ms", param: t.formats.params.cbr320 },
        { from: "WAV", to: "MP3", engine: "libmp3lame", speed: "~85ms", param: t.formats.params.vbrV0 },
        { from: "FLAC", to: "MP3", engine: "libmp3lame", speed: "~120ms", param: t.formats.params.mobileCompatible },
        { from: "WAV", to: "FLAC", engine: "flac", speed: "~90ms", param: t.formats.params.bitPerfect },
        { from: "M4A", to: "MP3", engine: "libmp3lame", speed: "~75ms", param: t.formats.params.aacToMp3 },
        { from: "OGG", to: "MP3", engine: "libmp3lame", speed: "~65ms", param: t.formats.params.universalPlayer },
        { from: "FLAC", to: "WAV", engine: "pcm_s16le", speed: "~45ms", param: t.formats.params.studioDaw },
        { from: "MP3", to: "OGG", engine: "libopus", speed: "~55ms", param: t.formats.params.opusUltraClear },
      ],
    },
    {
      id: "doc",
      icon: FileText,
      engineTitle: "Poppler & Pandoc",
      engineDetails: t.formats.engineDetails.doc,
      conversions: [
        { from: "PDF", to: "PNG", engine: "pdftoppm", speed: "~65ms/sayfa", param: t.formats.params.antiAliased300 },
        { from: "PDF", to: "JPG", engine: "pdftoppm", speed: "~50ms/sayfa", param: t.formats.params.pageByPage },
        { from: "DOCX", to: "PDF", engine: "pandoc", speed: "~180ms", param: t.formats.params.layoutPreserve },
        { from: "MD", to: "PDF", engine: "pandoc + wkhtml", speed: "~140ms", param: t.formats.params.githubCssTemplate },
        { from: "UDF", to: "PDF", engine: "udf2pdf / poppler", speed: "~90ms", param: t.formats.params.udfToPdf },
        { from: "PDF", to: "TXT", engine: "pdftotext", speed: "~15ms", param: t.formats.params.plainTextUtf8 },
      ],
    },
  ];

  return (
    <Section id="formatlar" className="relative border-b border-border bg-transparent">
      <Container>
        <Reveal>
          <div className="max-w-[720px]">
            <p className="font-mono text-[11px] font-semibold tracking-widest text-brand-strong uppercase">
              03 / {t.formats.badge}
            </p>
            <h2 className="mt-3 font-heading font-heading-expanded text-[1.95rem] font-bold tracking-tight text-foreground sm:text-[2.35rem]">
              {t.formats.title}
            </h2>
            <p className="mt-3 text-[15px] leading-relaxed text-muted-foreground sm:text-base">
              {t.formats.subtitle}
            </p>
          </div>
        </Reveal>
        <div className="mt-10">
          <Tabs defaultValue="image" className="w-full">
            <Reveal delay={0.05}>
              <TabsList className="flex h-auto w-full flex-wrap justify-start gap-1 rounded-[2px] border border-border bg-muted/40 p-1">
                {categories.map((cat) => {
                  const Icon = cat.icon;
                  return (
                    <TabsTrigger
                      key={cat.id}
                      value={cat.id}
                      className="flex items-center gap-2 rounded-[2px] px-3.5 py-1.5 font-mono text-[12px] font-medium transition-colors data-[state=active]:bg-brand data-[state=active]:text-primary-foreground"
                    >
                      <Icon className="h-3.5 w-3.5" aria-hidden="true" />
                      <span>{t.formats.tabs[cat.id]}</span>
                    </TabsTrigger>
                  );
                })}
              </TabsList>
            </Reveal>

            {categories.map((cat) => (
              <TabsContent key={cat.id} value={cat.id} className="mt-5 focus-visible:outline-none">
                <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 border border-border bg-muted/40 px-3.5 py-2.5 font-mono text-[11.5px]">
                  <div className="flex items-center gap-2 text-foreground font-semibold">
                    <Cpu className="h-3.5 w-3.5 text-brand" aria-hidden="true" />
                    <span>{t.formats.engineTag}: {cat.engineTitle}</span>
                  </div>
                  <span className="text-muted-foreground">
                    {cat.engineDetails}
                  </span>
                </div>
                <div className="mt-3 grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-2 gap-2">
                  {cat.conversions.map((conv) => (
                    <div
                      key={`${conv.from}-${conv.to}`}
                      className="flex items-center justify-between border border-border bg-card px-3.5 py-2.5 font-mono text-[12px] transition-colors hover:border-brand"
                    >
                      <div className="flex items-center gap-2 font-semibold">
                        <span className="text-foreground">{conv.from}</span>
                        <ArrowRight className="h-3 w-3 text-brand" aria-hidden="true" />
                        <span className="text-brand-strong">{conv.to}</span>
                      </div>
                      <div className="flex items-center gap-3 text-[11px] text-muted-foreground">
                        <span>{conv.param}</span>
                        <span className="rounded-[2px] border border-border bg-muted px-1.5 py-0.5 text-[10px] text-foreground/80 font-mono">
                          {conv.speed}
                        </span>
                      </div>
                    </div>
                  ))}
                </div>
              </TabsContent>
            ))}
          </Tabs>
        </div>
        <div className="mt-12 border-t border-border pt-8">
          <div className="flex items-center justify-between font-mono text-[11px] font-semibold text-brand-strong uppercase">
            <span className="flex items-center gap-1.5">
              <Layers className="h-3.5 w-3.5" aria-hidden="true" />
              {t.formats.profilesTitle}
            </span>
            <span className="text-muted-foreground/70 font-normal">
              CLI: --preset &lt;web|balanced|lossless&gt;
            </span>
          </div>

          <div className="mt-4 grid grid-cols-1 gap-4 md:grid-cols-3">
            {t.formats.profiles.map((profile) => (
              <div key={profile.name} className="flex flex-col justify-between rounded-[2px] border border-border bg-card p-5">
                <div>
                  <span className="font-mono text-[10.5px] font-semibold text-brand-strong uppercase tracking-wider">
                    {profile.label}
                  </span>
                  <h3 className="mt-2 font-heading font-heading-expanded text-[15px] font-bold text-foreground">
                    {profile.name}
                  </h3>
                  <p className="mt-2 text-[13px] leading-relaxed text-muted-foreground">
                    {profile.desc}
                  </p>
                </div>
                <div className="mt-4 border-t border-border pt-2 font-mono text-[10px] text-muted-foreground/60">
                  {t.formats.defaultRayon}
                </div>
              </div>
            ))}
          </div>
        </div>
      </Container>
    </Section>
  );
}
