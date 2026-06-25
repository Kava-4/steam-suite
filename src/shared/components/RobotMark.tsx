interface RobotMarkProps {
  className?: string;
}

/** Minimal line-art robot — transparent, uses currentColor for theme accent. */
export function RobotMark({ className = "h-8 w-8" }: RobotMarkProps) {
  return (
    <svg
      viewBox="0 0 32 32"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      className={className}
      aria-hidden
    >
      <rect
        x="7"
        y="10"
        width="18"
        height="14"
        rx="3.5"
        stroke="currentColor"
        strokeWidth="1.75"
      />
      <circle cx="12.5" cy="17" r="1.35" fill="currentColor" />
      <circle cx="19.5" cy="17" r="1.35" fill="currentColor" />
      <path
        d="M16 10V7"
        stroke="currentColor"
        strokeWidth="1.75"
        strokeLinecap="round"
      />
      <circle cx="16" cy="5.5" r="1.35" fill="currentColor" />
      <path
        d="M7 15H5.5"
        stroke="currentColor"
        strokeWidth="1.75"
        strokeLinecap="round"
      />
      <path
        d="M25 15h1.5"
        stroke="currentColor"
        strokeWidth="1.75"
        strokeLinecap="round"
      />
    </svg>
  );
}
