"use client";

import { Cpu, ShieldCheck, Code2, Github, ArrowUpRight } from "lucide-react";
import { Container, Section } from "./section";
import { Reveal } from "./reveal";
import { useI18n } from "@/lib/i18n";

export function TrustMetrics() {
  const { t } = useI18n();

  const cardIcons = [Cpu, ShieldCheck, Code2];
  const technicalSpecs = [
    "POSIX EXEC · DAEMON YOK · 0 SYSTEMD SERVICE",
    "SIFIR AĞ İSTEĞİ · SIFIR TELEMETRİ · AIR-GAP UYUMLU",
    "GPL-3.0-or-later · BAĞIMSIZ DENETLENEBİLİR DERLEME",
  ];

  return (
    <Section id="metrikler" className="relative border-b border-border bg-transparent">
      <Container>
        <Reveal>
          <div className="max-w-[720px]">
            <p className="font-mono text-[11px] font-semibold tracking-widest text-brand-strong uppercase">
              02 / {t.trustMetrics.badge}
            </p>
            <h2 className="mt-3 font-heading font-heading-expanded text-[1.95rem] font-bold tracking-tight text-foreground sm:text-[2.35rem]">
              {t.trustMetrics.title}
            </h2>
            <p className="mt-3 text-[15px] leading-relaxed text-muted-foreground sm:text-base">
              {t.trustMetrics.subtitle}
            </p>
          </div>
        </Reveal>

        <div className="mt-10 grid grid-cols-1 gap-5 md:grid-cols-3">
          {t.trustMetrics.cards.map((card, i) => {
            const Icon = cardIcons[i] || ShieldCheck;
            const spec = technicalSpecs[i];
            const isThirdCard = i === 2;

            return (
              <Reveal key={card.label} delay={0.06 * i}>
                <div className="flex h-full flex-col justify-between rounded-[2px] border border-border bg-card p-6">
                  <div>
                    <div className="flex items-center justify-between border-b border-border pb-3">
                      <span className="font-mono text-[11px] font-semibold text-brand-strong uppercase">
                        {card.tag}
                      </span>
                      <Icon className="h-4 w-4 text-muted-foreground" aria-hidden="true" />
                    </div>

                    <div className="mt-5">
                      <p className="font-heading font-heading-expanded text-[2.6rem] font-bold leading-none tracking-tight text-foreground">
                        {card.value}
                      </p>
                      <h3 className="mt-2 font-mono text-[14px] font-semibold text-foreground/90">
                        {card.label}
                      </h3>
                    </div>

                    <p className="mt-3 text-[13.5px] leading-relaxed text-muted-foreground">
                      {card.desc}
                    </p>

                    {isThirdCard && (
                      <div className="mt-4 pt-3 border-t border-border">
                        <a
                          href="https://github.com/enesehs/konvert"
                          target="_blank"
                          rel="noopener noreferrer"
                          className="inline-flex items-center gap-1.5 font-mono text-[12px] font-medium text-brand-strong hover:underline transition-colors"
                        >
                          <Github className="h-3.5 w-3.5" aria-hidden="true" />
                          <span>github.com/enesehs/konvert</span>
                          <ArrowUpRight className="h-3 w-3 opacity-60" aria-hidden="true" />
                        </a>
                      </div>
                    )}
                  </div>

                  <div className="mt-6 border-t border-border pt-3 font-mono text-[10px] text-muted-foreground/70">
                    {spec}
                  </div>
                </div>
              </Reveal>
            );
          })}
        </div>
      </Container>
    </Section>
  );
}
