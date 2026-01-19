import prettyMilliseconds from "pretty-ms";
import React, { useEffect, useState } from "react";
import { AppState, LayoutType, UniqueId } from "./types";

const THIRTY_SECONDS = 30 * 1000;
const ONE_WEEK = 7 * 24 * 60 * 60 * 1000;

const convertTimestampToHuman = (when: number) => {
  const calc = Date.now() - when;

  if (calc < THIRTY_SECONDS) {
    return "Just now";
  } else if (calc > ONE_WEEK) {
    return new Date(when).toLocaleDateString();
  } else {
    const str = prettyMilliseconds(calc, {
      compact: true,
      separateMilliseconds: true,
      unitCount: 1,
      verbose: true,
    });
    return str + " ago";
  }
};

export const useHumanizedTime = (when: number) => {
  const [display, setDisplay] = useState(convertTimestampToHuman(when));

  useEffect(() => {
    const interval = setInterval(() => {
      setDisplay(convertTimestampToHuman(when));
    }, 1000);
    return () => {
      clearInterval(interval);
    };
  }, [when]);

  return display;
};

export const useAsync = <T>(
  asyncFn: () => Promise<T>,
  deps: React.DependencyList = [],
) => {
  useEffect(() => {
    const runAsync = async () => {
      try {
        await asyncFn();
      } catch (err) {
        console.log(err);
      }
    };

    runAsync();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [asyncFn, ...deps]);
};

export const getLayoutName = (state: AppState, layoutId: UniqueId) => {
  const layout = state.score.layouts.by_id[layoutId];

  if (layout.title) {
    return layout.title;
  }

  if (layout.layout_type === LayoutType.FullScore) {
    return "Full Score";
  }

  const instrumentNames = state.score.players.order
    .filter((playerId) => layout.player_ids.includes(playerId))
    .flatMap((playerId) => {
      const player = state.score.players.by_id[playerId];
      return player.instrument_ids.map((instrumentId) => {
        const instrument = state.score.instruments.by_id[instrumentId];
        return (
          instrument.long_name +
          (instrument.count ? ` ${instrument.count}` : "")
        );
      });
    });

  if (instrumentNames.length === 0) {
    return "Empty Part";
  }

  if (instrumentNames.length === 1) {
    return instrumentNames[0];
  }

  const last = instrumentNames.pop();

  return `${instrumentNames.join(", ")} & ${last}`;
};

export const getFlowName = (
  state: AppState,
  index: number,
  flowId: UniqueId,
) => {
  const flow = state.score.flows.by_id[flowId];

  return `${index + 1}. ${flow.title || "Untitled Flow"}`;
};

export const parseTimeSignature = (input: string) => {
  const validBeatTypes = [1, 2, 4, 8, 16, 32, 64, 128];
  const match = input.match(/^(\d{1,2})\/(\d{1,3})$/);

  if (!match) return null;

  const beats = parseInt(match[1], 10);
  const beatType = parseInt(match[2], 10);

  if (!validBeatTypes.includes(beatType)) return null;

  return { beats, beatType };
};

/**
 * Generates a rainbow of colors for the given count (minimum 7).
 * Returns an array of hex color strings evenly distributed across the HSL color spectrum.
 */
export const generateRainbowColors = (count: number): string[] => {
  const minColors = 7;
  const actualCount = Math.max(count, minColors);

  const colors: string[] = [];
  const saturation = 70; // Vibrant but not too bright
  const lightness = 50; // Medium lightness for good contrast

  for (let i = 0; i < actualCount; i++) {
    // Distribute hues evenly across the spectrum (0-360 degrees)
    const hue = (i * 360) / actualCount;

    // Convert HSL to RGB, then to hex
    const hslToRgb = (h: number, s: number, l: number) => {
      h /= 360;
      s /= 100;
      l /= 100;

      const hue2rgb = (p: number, q: number, t: number) => {
        if (t < 0) t += 1;
        if (t > 1) t -= 1;
        if (t < 1 / 6) return p + (q - p) * 6 * t;
        if (t < 1 / 2) return q;
        if (t < 2 / 3) return p + (q - p) * (2 / 3 - t) * 6;
        return p;
      };

      let r, g, b;
      if (s === 0) {
        r = g = b = l;
      } else {
        const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
        const p = 2 * l - q;
        r = hue2rgb(p, q, h + 1 / 3);
        g = hue2rgb(p, q, h);
        b = hue2rgb(p, q, h - 1 / 3);
      }

      const toHex = (c: number) => {
        const hex = Math.round(c * 255).toString(16);
        return hex.length === 1 ? "0" + hex : hex;
      };

      return `#${toHex(r)}${toHex(g)}${toHex(b)}`;
    };

    colors.push(hslToRgb(hue, saturation, lightness));
  }

  return colors;
};

/**
 * Desaturates and darkens a hex color by adjusting saturation and lightness in HSL space.
 * @param hexColor - The hex color string (e.g., "#ff0000")
 * @param saturationFactor - Factor to multiply saturation by (0-1)
 * @param lightnessFactor - Factor to multiply lightness by (0-1)
 * @returns The desaturated and darkened hex color string
 */
export const desaturateColor = (
  hexColor: string,
  saturationFactor: number,
  lightnessFactor: number,
): string => {
  // Remove # if present
  const hex = hexColor.replace("#", "");

  // Convert hex to RGB
  const r = parseInt(hex.substr(0, 2), 16) / 255;
  const g = parseInt(hex.substr(2, 2), 16) / 255;
  const b = parseInt(hex.substr(4, 2), 16) / 255;

  // Find min and max values
  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  let h = 0,
    s = 0,
    l = (max + min) / 2;

  if (max !== min) {
    const d = max - min;
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min);

    switch (max) {
      case r:
        h = (g - b) / d + (g < b ? 6 : 0);
        break;
      case g:
        h = (b - r) / d + 2;
        break;
      case b:
        h = (r - g) / d + 4;
        break;
    }
    h /= 6;
  }

  // Reduce saturation
  s *= saturationFactor;

  // Reduce lightness to match neutral-800 darkness
  l *= lightnessFactor;

  // Convert back to RGB
  const hue2rgb = (p: number, q: number, t: number) => {
    if (t < 0) t += 1;
    if (t > 1) t -= 1;
    if (t < 1 / 6) return p + (q - p) * 6 * t;
    if (t < 1 / 2) return q;
    if (t < 2 / 3) return p + (q - p) * (2 / 3 - t) * 6;
    return p;
  };

  let r2, g2, b2;
  if (s === 0) {
    r2 = g2 = b2 = l;
  } else {
    const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
    const p = 2 * l - q;
    r2 = hue2rgb(p, q, h + 1 / 3);
    g2 = hue2rgb(p, q, h);
    b2 = hue2rgb(p, q, h - 1 / 3);
  }

  // Convert back to hex
  const toHex = (c: number) => {
    const hex = Math.round(c * 255).toString(16);
    return hex.length === 1 ? "0" + hex : hex;
  };

  return `#${toHex(r2)}${toHex(g2)}${toHex(b2)}`;
};

/**
 * Lightens a hex color by increasing its lightness in HSL space.
 * @param hexColor - The hex color string (e.g., "#ff0000")
 * @param factor - Factor to lighten by (0-1, where 1 is maximum lightness)
 * @returns The lightened hex color string
 */
export const lightenColor = (hexColor: string, factor: number): string => {
  // Remove # if present
  const hex = hexColor.replace("#", "");

  // Convert hex to RGB
  const r = parseInt(hex.substr(0, 2), 16) / 255;
  const g = parseInt(hex.substr(2, 2), 16) / 255;
  const b = parseInt(hex.substr(4, 2), 16) / 255;

  // Convert RGB to HSL
  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  let h = 0,
    s = 0,
    l = (max + min) / 2;

  if (max !== min) {
    const d = max - min;
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min);

    switch (max) {
      case r:
        h = (g - b) / d + (g < b ? 6 : 0);
        break;
      case g:
        h = (b - r) / d + 2;
        break;
      case b:
        h = (r - g) / d + 4;
        break;
    }
    h /= 6;
  }

  // Increase lightness
  l = Math.min(1, l + factor * (1 - l));

  // Convert back to RGB
  const hue2rgb = (p: number, q: number, t: number) => {
    if (t < 0) t += 1;
    if (t > 1) t -= 1;
    if (t < 1 / 6) return p + (q - p) * 6 * t;
    if (t < 1 / 2) return q;
    if (t < 2 / 3) return p + (q - p) * (2 / 3 - t) * 6;
    return p;
  };

  let r2, g2, b2;
  if (s === 0) {
    r2 = g2 = b2 = l;
  } else {
    const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
    const p = 2 * l - q;
    r2 = hue2rgb(p, q, h + 1 / 3);
    g2 = hue2rgb(p, q, h);
    b2 = hue2rgb(p, q, h - 1 / 3);
  }

  // Convert back to hex
  const toHex = (c: number) => {
    const hex = Math.round(c * 255).toString(16);
    return hex.length === 1 ? "0" + hex : hex;
  };

  return `#${toHex(r2)}${toHex(g2)}${toHex(b2)}`;
};
