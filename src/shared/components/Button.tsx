import { Button as HeroButton } from "@heroui/react";
import type { ButtonHTMLAttributes, ComponentProps, MouseEvent } from "react";

const variantMap = {
  primary: "primary",
  secondary: "secondary",
  danger: "danger",
  ghost: "ghost",
} as const;

type HeroButtonProps = ComponentProps<typeof HeroButton>;

export function Button({
  variant = "secondary",
  size = "md",
  className = "",
  onClick,
  disabled,
  children,
  type = "button",
  title,
  ...rest
}: ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: keyof typeof variantMap;
  size?: "sm" | "md";
}) {
  const heroVariant = variantMap[variant];
  const heroSize = size === "sm" ? "sm" : "md";
  const gradient = variant === "primary" ? "btn-primary" : "";

  const button = (
    <HeroButton
      variant={heroVariant}
      size={heroSize}
      isDisabled={disabled}
      className={`${gradient} ${className}`.trim()}
      onPress={
        onClick
          ? () => {
              onClick({} as MouseEvent<HTMLButtonElement>);
            }
          : undefined
      }
      {...(type !== "button" ? { type } : {})}
      {...(rest as Partial<HeroButtonProps>)}
    >
      {children}
    </HeroButton>
  );

  if (title) {
    return <span title={title}>{button}</span>;
  }

  return button;
}
