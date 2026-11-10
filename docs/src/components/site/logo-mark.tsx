export function LogoMark({ className }: { className?: string }) {
  return (
    <svg
      viewBox="0 0 1000 1000"
      role="img"
      aria-label="Konvert logosu"
      className={className}
      suppressHydrationWarning
    >
      <path
        d="M287.686 517.213 C287.686 517.213 287.686 156.554 287.686 109.168 C287.686 61.782 253.024 98.199 253.024 98.199 L93.754 257.469 L93.754 705.002"
        fill="#007ebf"
      />
      <polygon
        points="93.754,762.918 246.004,915.167 902.825,253.081 765.932,116.188"
        fill="#007ebf"
      />
      <path
        d="M466.699 749.755 L623.774 597.945 L902.825 876.995 C902.825 876.995 917.742 908.586 885.274 915.167 L623.774 915.167 Z"
        fill="#007ebf"
      />
      <polygon points="765.932,257.469 765.932,116.188 902.825,253.081" fill="#7ecaf6" />
    </svg>
  );
}
