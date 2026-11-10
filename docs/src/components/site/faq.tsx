"use client";

import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion";
import { Container, Section } from "./section";
import { Reveal } from "./reveal";
import { useI18n } from "@/lib/i18n";

export function Faq() {
  const { t } = useI18n();

  return (
    <Section id="sss" className="relative border-b-0 bg-transparent">
      <Container>
        <Reveal>
          <div className="max-w-[720px]">
            <p className="font-mono text-[11px] font-semibold tracking-widest text-brand-strong uppercase">
              05 / {t.faq.badge}
            </p>
            <h2 className="mt-3 font-heading font-heading-expanded text-[1.95rem] font-bold tracking-tight text-foreground sm:text-[2.35rem]">
              {t.faq.title}
            </h2>
            <p className="mt-3 text-[15px] leading-relaxed text-muted-foreground sm:text-base">
              {t.faq.subtitle}
            </p>
          </div>
        </Reveal>
        <div className="mt-10">
          <Accordion
            type="single"
            collapsible
            className="border-t border-border"
          >
            {t.faq.items.map((item, i) => (
              <AccordionItem
                key={item.q}
                value={`faq-${i}`}
                className="border-b border-border"
              >
                <AccordionTrigger className="py-4 text-left font-mono text-[14px] font-semibold text-foreground hover:text-brand-strong transition-colors hover:no-underline [&>svg]:text-muted-foreground">
                  <span className="flex items-baseline gap-3">
                    <span className="font-mono text-[11px] font-normal text-brand-strong">
                      {String(i + 1).padStart(2, "0")}
                    </span>
                    <span>{item.q}</span>
                  </span>
                </AccordionTrigger>
                <AccordionContent className="pb-4 pl-7 text-[13.5px] leading-relaxed text-muted-foreground font-sans">
                  {item.a}
                </AccordionContent>
              </AccordionItem>
            ))}
          </Accordion>
        </div>
      </Container>
    </Section>
  );
}
