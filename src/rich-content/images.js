export const MINIMUM_IMAGE_WIDTH = 32;
export const MAXIMUM_IMAGE_WIDTH = 2048;

export function imageDisplayWidth( value ) {
  return Number.isInteger( value ) && value >= MINIMUM_IMAGE_WIDTH && value <= MAXIMUM_IMAGE_WIDTH
    ? value
    : null;
}
