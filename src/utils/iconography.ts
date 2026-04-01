interface StrokeIconOptions {
  size?: number;
  strokeWidth?: number;
  className?: string;
  fill?: string;
}

interface FillIconOptions {
  size?: number;
  className?: string;
}

function classAttr(className?: string) {
  return className ? ` class="${className}"` : '';
}

export function strokeIcon(
  paths: string,
  { size = 18, strokeWidth = 1.9, className, fill = 'none' }: StrokeIconOptions = {},
) {
  return `<svg${classAttr(className)} width="${size}" height="${size}" viewBox="0 0 24 24" fill="${fill}" stroke="currentColor" stroke-width="${strokeWidth}" stroke-linecap="round" stroke-linejoin="round">${paths}</svg>`;
}

export function fillIcon(paths: string, { size = 18, className }: FillIconOptions = {}) {
  return `<svg${classAttr(className)} width="${size}" height="${size}" viewBox="0 0 24 24" fill="currentColor">${paths}</svg>`;
}
