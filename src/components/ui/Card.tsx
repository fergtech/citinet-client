import React from "react";

interface CardProps {
  children: React.ReactNode;
  className?: string;
}

export function Card({ children, className = "" }: CardProps) {
  return (
    <div
      className={`bg-[var(--bg-card)] border border-[var(--border-color)] rounded-xl p-6 transition-colors duration-200 card-glow ${className}`}
    >
      {children}
    </div>
  );
}
