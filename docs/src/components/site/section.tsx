import { cn } from "@/lib/utils";

export function Container({
  className,
  children,
}: {
  className?: string;
  children: React.ReactNode;
}) {
  return (
    <div className={cn("mx-auto w-full max-w-[1120px] px-5 sm:px-8", className)}>
      {children}
    </div>
  );
}

export function SectionHead({
  index,
  label,
  title,
  desc,
  dark = false,
  className,
}: {
  index: string;
  label: string;
  title: React.ReactNode;
  desc?: React.ReactNode;
  dark?: boolean;
  className?: string;
}) {
  return (
    <div className={cn("max-w-[680px]", className)}>
      <p className="font-mono text-[11px] font-medium tracking-[0.22em]">
        <span className={dark ? "text-brand-strong" : "text-brand-strong"}>
          {index}
        </span>
        <span className={dark ? "text-white/50" : "text-muted-foreground"}>
          {" "}
          / {label}
        </span>
      </p>
      <h2
        className={cn(
          "mt-4 font-heading font-heading-expanded text-[1.9rem] font-semibold leading-[1.08] tracking-[-0.015em] sm:text-[2.35rem]",
          dark ? "text-white" : "text-foreground"
        )}
      >
        {title}
      </h2>
      {desc ? (
        <p
          className={cn(
            "mt-4 text-[15.5px] leading-relaxed sm:text-base",
            dark ? "text-white/65" : "text-muted-foreground"
          )}
        >
          {desc}
        </p>
      ) : null}
    </div>
  );
}

export function Section({
  id,
  className,
  children,
  border = true,
}: {
  id?: string;
  className?: string;
  children: React.ReactNode;
  border?: boolean;
}) {
  return (
    <section
      id={id}
      className={cn(
        "scroll-mt-24 py-16 sm:py-20 lg:py-24",
        border && "border-t border-border",
        className
      )}
    >
      {children}
    </section>
  );
}
