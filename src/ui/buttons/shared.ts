export type Variant =
  | "standard"
  | "interactive"
  | "filled"
  | "filled-dark"
  | "outline";

export const getTheme = (variant: Variant, isSelected?: boolean) => {
  if (isSelected) {
    return "bg-sky-500 text-black";
  }

  switch (variant) {
    case "filled":
      return "bg-neutral-800 text-white hover:bg-neutral-700";
    case "filled-dark":
      return "bg-neutral-950 text-white hover:bg-neutral-800";
    case "interactive":
      return "text-white bg-neutral-900 hover:bg-neutral-800";
    case "outline":
      return "border border-neutral-600 text-white hover:bg-neutral-800";
    default:
      return "";
  }
};
