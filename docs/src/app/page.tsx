import { SiteHeader } from "@/components/site/site-header";
import { Hero } from "@/components/site/hero";
import { Problem } from "@/components/site/problem";
import { TrustMetrics } from "@/components/site/trust-metrics";
import { Formats } from "@/components/site/formats";
import { Install } from "@/components/site/install";
import { Faq } from "@/components/site/faq";
import { SiteFooter } from "@/components/site/site-footer";
import { MobileCta } from "@/components/site/mobile-cta";

export default function Home() {
  return (
    <>
      <SiteHeader />
      <main id="main" className="relative flex-1">
        <div className="pointer-events-none absolute inset-0 -z-10 overflow-hidden">
          <div className="absolute top-0 left-1/2 -translate-x-1/2 h-[760px] w-full max-w-[1240px] bg-[radial-gradient(ellipse_75%_50%_at_50%_0%,rgba(0,126,191,0.12),rgba(107,74,255,0.04)_50%,transparent_75%)] dark:bg-[radial-gradient(ellipse_75%_50%_at_50%_0%,rgba(0,126,191,0.22),rgba(107,74,255,0.07)_50%,transparent_75%)]" />

          <div className="absolute inset-0 [background-image:linear-gradient(to_right,rgba(0,0,0,0.035)_1px,transparent_1px),linear-gradient(to_bottom,rgba(0,0,0,0.035)_1px,transparent_1px)] dark:[background-image:linear-gradient(to_right,rgba(255,255,255,0.025)_1px,transparent_1px),linear-gradient(to_bottom,rgba(255,255,255,0.025)_1px,transparent_1px)] [background-size:48px_48px] [mask-image:radial-gradient(ellipse_85%_75%_at_50%_35%,black_40%,transparent_90%)]" />

          <div className="absolute inset-0 [background-image:radial-gradient(rgba(0,126,191,0.12)_1px,transparent_1px)] dark:[background-image:radial-gradient(rgba(0,126,191,0.18)_1px,transparent_1px)] [background-size:24px_24px] [mask-image:radial-gradient(ellipse_60%_40%_at_50%_15%,black_25%,transparent_80%)]" />

          <div className="absolute top-[38%] left-1/2 -translate-x-1/2 h-[600px] w-full max-w-[1000px] bg-[radial-gradient(ellipse_60%_40%_at_50%_50%,rgba(0,126,191,0.05),transparent_70%)] dark:bg-[radial-gradient(ellipse_60%_40%_at_50%_50%,rgba(0,126,191,0.09),transparent_70%)]" />
        </div>

        <Hero />
        <Problem />
        <TrustMetrics />
        <Formats />
        <Install />
        <Faq />
      </main>
      <SiteFooter />
      <MobileCta />
    </>
  );
}
