"use client";

import * as React from "react";
import { Check, Copy } from "lucide-react";

export function CodeBlock({
  label,
  code,
  caption,
  children,
}: {
  label: string;
  code: string;
  caption?: string;
  children?: React.ReactNode;
}) {
  const [copied, setCopied] = React.useState(false);

  const copy = async () => {
    let ok = false;
    try {
      await navigator.clipboard.writeText(code);
      ok = true;
    } catch {
      try {
        const ta = document.createElement("textarea");
        ta.value = code;
        ta.setAttribute("readonly", "");
        ta.style.position = "fixed";
        ta.style.opacity = "0";
        document.body.appendChild(ta);
        ta.select();
        ok = document.execCommand("copy");
        document.body.removeChild(ta);
      } catch {
        ok = false;
      }
    }
    if (ok) {
      setCopied(true);
      setTimeout(() => setCopied(false), 1800);
    }
  };

  return (
    <figure className="term rounded-[3px]">
      <div className="flex items-center justify-between border-b border-[var(--term-line)] px-4 py-2">
        <figcaption className="font-mono text-[10px] tracking-[0.18em] text-[var(--term-cmt)]">
          {label}
        </figcaption>
        <button
          type="button"
          onClick={copy}
          aria-label={`${label} içeriğini kopyala`}
          className="inline-flex h-8 w-8 items-center justify-center text-[var(--term-cmt)] transition-colors hover:text-white"
        >
          {copied ? (
            <Check className="h-[15px] w-[15px]" aria-hidden="true" />
          ) : (
            <Copy className="h-[15px] w-[15px]" aria-hidden="true" />
          )}
        </button>
      </div>
      <pre className="scroll-thin overflow-x-auto px-4 py-3.5 font-mono text-[12.5px] leading-[1.75]">
        {children ?? <code>{code}</code>}
      </pre>
      {caption ? (
        <p className="border-t border-[var(--term-line)] px-4 py-2 text-[11.5px] leading-snug text-[var(--term-cmt)]">
          {caption}
        </p>
      ) : null}
    </figure>
  );
}
