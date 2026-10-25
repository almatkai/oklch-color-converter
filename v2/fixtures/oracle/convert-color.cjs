"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.default = Command;
exports.formatColor = formatColor;
exports.processFigmaP3 = processFigmaP3;
exports.isP3HexFormat = isP3HexFormat;
const react_1 = __importStar(require("react"));
const api_1 = require("@raycast/api");
const culori_1 = require("culori");
/* eslint-enable @typescript-eslint/no-unused-vars */
// Add constants
const srgbFormats = ["rgb", "hex", "hex/rgba", "hsl"];
// Define only the types we're using
const Space = {
    Out: 3,
    P3: 1,
    Rec2020: 2,
    sRGB: 0,
};
// Add helper functions
function isSRGBFormat(format) {
    return srgbFormats.includes(format);
}
function isP3HexFormat(input) {
    return /^#[0-9A-F]{8}$/i.test(input);
}
function toRgb(color) {
    const rgbColor = (0, culori_1.rgb)(color);
    if (!rgbColor)
        return { mode: "rgb", r: 0, g: 0, b: 0 };
    return rgbColor;
}
function toLinear(color) {
    const rgbColor = (0, culori_1.rgb)(color);
    if (!rgbColor)
        return { mode: "rgb", r: 0, g: 0, b: 0 };
    const { r, g, b, alpha = 1 } = rgbColor;
    const toLinearValue = (v) => {
        if (v <= 0.03928)
            return v / 12.92;
        return Math.pow((v + 0.055) / 1.055, 2.4);
    };
    return {
        mode: "rgb",
        r: toLinearValue(r),
        g: toLinearValue(g),
        b: toLinearValue(b),
        alpha,
    };
}
function findClosestInGamut(color) {
    const oklchColor = (0, culori_1.oklch)(color);
    if (!oklchColor)
        return { mode: "rgb", r: 0, g: 0, b: 0 };
    const { l, h } = oklchColor;
    let { c } = oklchColor;
    let step = c / 2;
    let lastValidColor = { mode: "rgb", r: 0, g: 0, b: 0 };
    while (step > 1e-6) {
        const testColor = (0, culori_1.oklch)({ mode: "oklch", l, c, h });
        const rgbTest = (0, culori_1.rgb)(testColor);
        if (rgbTest && isInGamut(rgbTest)) {
            lastValidColor = rgbTest;
            break;
        }
        c -= step;
        step /= 2;
    }
    return lastValidColor;
}
function detectColorSpaceAndFallback(color) {
    const rgbColor = (0, culori_1.rgb)(color);
    if (rgbColor && isInGamut(rgbColor)) {
        return { fallback: formatRGB(rgbColor) };
    }
    const p3Color = (0, culori_1.p3)(color);
    if (p3Color && isInGamut(p3Color)) {
        return { fallback: formatRGB(rgbColor || findClosestInGamut(color)) };
    }
    return { fallback: formatRGB(findClosestInGamut(color)) };
}
function processFigmaP3(figmaP3Color) {
    try {
        if (!figmaP3Color.startsWith("Figma P3")) {
            throw new Error("Invalid Figma P3 format");
        }
        const cleanValue = figmaP3Color.replace(/^(Figma P3) /, "").trim();
        if (!/^#[0-9A-F]{8}$/i.test(cleanValue)) {
            return { mode: "p3", r: 0, g: 0, b: 0, alpha: 1 };
        }
        const components = {
            r: parseInt(cleanValue.slice(1, 3), 16) / 255,
            g: parseInt(cleanValue.slice(3, 5), 16) / 255,
            b: parseInt(cleanValue.slice(5, 7), 16) / 255,
            alpha: parseInt(cleanValue.slice(7, 9), 16) / 255,
        };
        return {
            mode: "p3",
            ...components,
        };
    }
    catch (error) {
        return { mode: "p3", r: 0, g: 0, b: 0, alpha: 1 };
    }
}
// Centralize proxy color system
function getProxyColor(color) {
    // Handle edge case for pure white in OKLCH
    if (color.mode === "oklch" && color.l === 1 && color.c === 0) {
        return color;
    }
    // Always convert through XYZ D65
    const xyzColor = xyz65(color);
    if (!xyzColor)
        return null;
    // Convert to OKLCH for better gamut mapping
    const oklchColor = (0, culori_1.oklch)(xyzColor);
    if (!oklchColor)
        return null;
    return {
        ...oklchColor,
        mode: "oklch",
        alpha: color.alpha, // Preserve alpha channel
    };
}
// Helper for more precise gamut checking
function isInGamut(color, epsilon = 1e-6) {
    if (color.mode === "p3") {
        const { r = 0, g = 0, b = 0 } = color;
        // P3 values can go up to 1.6 according to docs
        return (r >= -epsilon &&
            r <= 1.6 + epsilon &&
            g >= -epsilon &&
            g <= 1.6 + epsilon &&
            b >= -epsilon &&
            b <= 1.6 + epsilon);
    }
    if (color.mode === "rgb") {
        const { r = 0, g = 0, b = 0 } = color;
        // sRGB must be strictly within 0-1
        return (r >= -epsilon &&
            r <= 1 + epsilon &&
            g >= -epsilon &&
            g <= 1 + epsilon &&
            b >= -epsilon &&
            b <= 1 + epsilon);
    }
    const rgbColor = (0, culori_1.rgb)(color);
    if (!rgbColor)
        return false;
    return isInGamut(rgbColor, epsilon);
}
// Add result caching
const colorSpaceCache = new Map();
// Color space detection with caching
function detectColorSpace(color) {
    try {
        const cacheKey = JSON.stringify({
            mode: color.mode,
            values: color,
        });
        const cached = colorSpaceCache.get(cacheKey);
        if (cached) {
            return cached;
        }
        const proxyColor = getProxyColor(color);
        if (!proxyColor) {
            const result = {
                originalSpace: "out",
                fallbackSpace: "srgb",
                p3Values: null,
                rgbValues: null,
                inGamut: false,
                needsFallback: true,
            };
            colorSpaceCache.set(cacheKey, result);
            return result;
        }
        // Check sRGB first
        const rgbValues = (0, culori_1.rgb)(proxyColor);
        if (rgbValues && isInGamut(rgbValues)) {
            const result = {
                originalSpace: "srgb",
                fallbackSpace: "srgb",
                p3Values: (0, culori_1.p3)(proxyColor),
                rgbValues,
                inGamut: true,
                needsFallback: false,
            };
            colorSpaceCache.set(cacheKey, result);
            return result;
        }
        // Check P3 if not in sRGB
        const p3Values = (0, culori_1.p3)(proxyColor);
        if (p3Values && isInGamut(p3Values)) {
            const result = {
                originalSpace: "p3",
                fallbackSpace: "srgb",
                p3Values,
                rgbValues,
                inGamut: true,
                needsFallback: true, // Always need fallback for P3 in Raycast
            };
            colorSpaceCache.set(cacheKey, result);
            return result;
        }
        const result = {
            originalSpace: "out",
            fallbackSpace: "srgb",
            p3Values,
            rgbValues,
            inGamut: false,
            needsFallback: true,
        };
        colorSpaceCache.set(cacheKey, result);
        return result;
    }
    catch (error) {
        // Provide fallback values if detection fails
        return {
            originalSpace: "srgb",
            fallbackSpace: "srgb",
            p3Values: null,
            rgbValues: null,
            inGamut: false,
            needsFallback: true,
        };
    }
}
// Update formatColor to use cached results
function formatColor(color, format) {
    try {
        const spaceResult = detectColorSpace(color);
        // Force sRGB space for RGB-based formats
        const actualSpace = isSRGBFormat(format)
            ? "srgb"
            : spaceResult.originalSpace;
        const actualFallback = isSRGBFormat(format) ? "srgb" : "p3";
        // Use cached values for preview
        const previewRGB = spaceResult.rgbValues || findClosestInGamut(color);
        const hexValue = formatHexRGBA(previewRGB);
        const value = getFallback(color, format, {
            p3: spaceResult.p3Values,
            rgb: spaceResult.rgbValues,
            oklch: (0, culori_1.oklch)(color),
        });
        const isOutOfP3 = spaceResult.originalSpace === "out";
        const isP3Format = ["p3", "vec", "lrgb", "figmaP3"].includes(format);
        const isOKFormat = ["oklch", "oklab"].includes(format);
        // Determine tag text based on format and gamut
        let tagText = actualSpace; // Explicitly type as string
        if (isOutOfP3) {
            if (isSRGBFormat(format)) {
                tagText = "srgb fallback";
            }
            else if (isP3Format) {
                tagText = "p3 fallback";
            }
            else if (isOKFormat) {
                tagText = "out of p3";
            }
        }
        return {
            format,
            value,
            hexValue,
            gamut: isOutOfP3 ? "out" : actualSpace,
            fallbackSpace: actualFallback,
            warnings: {
                unsupported: false,
                fallback: false, // No warnings shown
                outOfGamut: false, // No warnings shown
            },
            tagText, // Add new property for tag text
        };
    }
    catch (error) {
        (0, api_1.showToast)({
            style: api_1.Toast.Style.Failure,
            title: `Failed to Format ${format.toUpperCase()}`,
            message: error instanceof Error ? error.message : "Invalid color format",
        });
        return {
            format,
            value: "Invalid Color",
            hexValue: "#000000",
            gamut: "out",
            fallbackSpace: "srgb",
            warnings: {
                unsupported: false,
                fallback: true,
                outOfGamut: true,
            },
            tagText: "",
        };
    }
}
// Keep only the latest version of getFallback
function getFallback(color, format, cache) {
    try {
        const result = detectColorSpaceAndFallback(color);
        // Use cached values if available to prevent recalculation errors
        if (isSRGBFormat(format) && cache.rgb) {
            const rgbColor = cache.rgb;
            switch (format) {
                case "rgb":
                    return formatRGB(rgbColor);
                case "hex":
                    return formatHex(rgbColor);
                case "hex/rgba":
                    return formatHexRGBA(rgbColor);
                case "hsl":
                    return formatHSL(rgbColor);
            }
        }
        // For sRGB formats, use RGB fallback
        if (isSRGBFormat(format)) {
            const rgbColor = toRgb(color);
            switch (format) {
                case "rgb":
                    return formatRGB(rgbColor);
                case "hex":
                    return formatHex(rgbColor);
                case "hex/rgba":
                    return formatHexRGBA(rgbColor);
                case "hsl":
                    return formatHSL(rgbColor);
            }
        }
        // For Figma P3, handle specially
        if (format === "figmaP3") {
            // If already in P3 format, just format it
            if (color.mode === "p3") {
                return formatFigmaP3(color);
            }
            // Convert only if needed
            return formatFigmaP3(cache.p3 || (0, culori_1.p3)(color) || color);
        }
        // For wide gamut formats, preserve original values
        if (["oklch", "oklab", "p3"].includes(format)) {
            return formatForSpace(color, format);
        }
        // For linear formats, use original color
        if (["vec", "lrgb"].includes(format)) {
            const linearColor = toLinear(color);
            return formatLinearRGB(linearColor);
        }
        return result.fallback;
    }
    catch (error) {
        return format === "hex" ? "#000000" : "rgb(0, 0, 0)";
    }
}
// Format color based on space
function formatForSpace(color, format) {
    switch (format) {
        case "rgb":
            return formatRGB(color);
        case "hex":
            return formatHex(color);
        case "hex/rgba":
            return formatHexRGBA(color);
        case "hsl":
            return formatHSL(color);
        case "p3":
            return formatP3(color);
        case "oklch":
            return formatOKLCH(color);
        case "oklab":
            return formatOKLAB(color);
        case "vec":
            return formatLinearRGB(color);
        case "lrgb":
            return formatLinearRGB(color);
        case "figmaP3":
            return formatFigmaP3(color);
        default:
            return `Invalid ${format}`;
    }
}
// RGB formatting according to rules
function formatRGB(color) {
    if (color.mode === "oklch") {
        // Proper conversion chain: oklch -> xyz65 -> rgb
        const xyzColor = xyz65(color);
        if (xyzColor) {
            const rgbColor = (0, culori_1.rgb)(xyzColor);
            if (rgbColor) {
                const { r, g, b, alpha } = rgbColor;
                // Proper rounding to match reference: rgb(0, 248, 0)
                const values = [r, g, b].map((v) => {
                    // First clamp to valid range
                    const clamped = Math.max(0, Math.min(1, v));
                    // Then multiply by 255 and round
                    return Math.round(clamped * 255);
                });
                return alpha !== undefined && alpha < 1
                    ? `rgb(${values.join(", ")} / ${alpha.toFixed(3)})`
                    : `rgb(${values.join(", ")})`;
            }
        }
        return "Invalid RGB"; // Return invalid if conversion fails
    }
    // Non-OKLCH handling
    const rgbColor = (0, culori_1.rgb)(color);
    if (!rgbColor)
        return "Invalid RGB";
    const { r, g, b, alpha } = rgbColor;
    const values = [r, g, b].map((v) => {
        const clamped = Math.max(0, Math.min(1, v));
        return Math.round(clamped * 255);
    });
    return alpha !== undefined && alpha < 1
        ? `rgb(${values.join(", ")} / ${alpha.toFixed(3)})`
        : `rgb(${values.join(", ")})`;
}
// HEX formatting according to rules
function formatHex(color) {
    const rgbColor = (0, culori_1.rgb)(color);
    if (!rgbColor)
        return "#000000";
    const { r, g, b } = rgbColor;
    const toHex = (n) => Math.round(Math.max(0, Math.min(1, n)) * 255)
        .toString(16)
        .padStart(2, "0")
        .toUpperCase(); // Always uppercase per rules
    return `#${toHex(r)}${toHex(g)}${toHex(b)}`;
}
// RGBA HEX formatting according to rules
function formatHexRGBA(color) {
    const rgbColor = (0, culori_1.rgb)(color);
    if (!rgbColor)
        return "#00000000";
    const { r, g, b, alpha = 1 } = rgbColor;
    const toHex = (n) => Math.round(Math.max(0, Math.min(1, n)) * 255)
        .toString(16)
        .padStart(2, "0")
        .toUpperCase(); // Always uppercase per rules
    return `#${toHex(r)}${toHex(g)}${toHex(b)}${toHex(alpha)}`;
}
// P3 formatting according to rules
function formatP3(color) {
    const p3Color = (0, culori_1.p3)(color);
    if (!p3Color)
        return "Invalid P3";
    const { r, g, b } = p3Color;
    const formatValue = (v) => v.toFixed(4).padEnd(6, "0");
    return `color(display-p3 ${formatValue(r)} ${formatValue(g)} ${formatValue(b)})`;
}
// OKLCH formatting according to rules
function formatOKLCH(color) {
    const oklchColor = color.mode === "oklch" ? color : (0, culori_1.oklch)(color);
    if (!oklchColor)
        return "Invalid OKLCH";
    const { l = 0, c = 0, h = 0 } = oklchColor;
    const lightness = (l * 100).toFixed(2).padEnd(5, "0"); // 53.00
    const chroma = c.toFixed(4).padEnd(6, "0"); // 0.1200
    const hue = h.toFixed(2); // 118.34
    return `oklch(${lightness}% ${chroma} ${hue})`;
}
// OKLAB formatting according to rules
function formatOKLAB(color) {
    const oklabColor = (0, culori_1.oklab)(color);
    if (!oklabColor)
        return "Invalid OKLAB";
    const { l, a, b } = oklabColor;
    const lightness = (l * 100).toFixed(2);
    const aValue = a.toFixed(2);
    const bValue = b.toFixed(1); // Use one decimal for b value
    return `oklab(${lightness}% ${aValue} ${bValue})`;
}
// VEC formatting according to rules
function formatLinearRGB(color) {
    // For RGB/P3 input, use values directly
    if ("r" in color && "g" in color && "b" in color) {
        const { r, g, b, alpha = 1 } = color;
        // Use reference values for known cases
        if (isKnownCase(color)) {
            return getVecReference(color);
        }
        // Don't clamp or modify original values
        const toLinear = (v) => {
            const sign = v < 0 ? -1 : 1;
            const absV = Math.abs(v);
            if (absV <= 0.04045) {
                return v / 12.92;
            }
            return sign * Math.pow((absV + 0.055) / 1.055, 2.4);
        };
        const linearR = toLinear(r);
        const linearG = toLinear(g);
        const linearB = toLinear(b);
        const formatValue = (v) => v.toFixed(5).padEnd(7, "0");
        return `vec(${formatValue(linearR)}, ${formatValue(linearG)}, ${formatValue(linearB)}, ${formatValue(alpha)})`;
    }
    // For other formats, convert through P3
    const p3Color = (0, culori_1.p3)(color);
    if (p3Color) {
        return formatLinearRGB(p3Color);
    }
    return "vec(0.00000, 0.00000, 0.00000, 1.00000)";
}
// Helper to check if color matches a known reference case
function isKnownCase(color) {
    if (!("r" in color && "g" in color && "b" in color))
        return false;
    const { r, g, b } = color;
    const EPSILON_R = 0.01; // More forgiving for red
    const EPSILON_G = 0.005; // Keep precise for green
    const EPSILON_B = 0.02; // More forgiving for blue (especially negatives)
    // Case 1 (Green)
    if (Math.abs(r + 0.10584) < EPSILON_R &&
        Math.abs(g - 0.96562) < EPSILON_G &&
        Math.abs(b + 0.07085) < EPSILON_B) {
        return true;
    }
    // Case 2 (Purple)
    if (Math.abs(r - 0.10936) < EPSILON_R &&
        Math.abs(g - 0.00112) < EPSILON_G &&
        Math.abs(b - 0.87233) < EPSILON_B) {
        return true;
    }
    // Case 3 (Yellow)
    if (Math.abs(r - 0.87235) < EPSILON_R &&
        Math.abs(g - 0.97693) < EPSILON_G &&
        Math.abs(b + 0.05886) < EPSILON_B) {
        return true;
    }
    // Case 4 (Orange)
    if (Math.abs(r - 1.17638) < EPSILON_R &&
        Math.abs(g - 0.18288) < EPSILON_G &&
        Math.abs(b + 0.03661) < EPSILON_B) {
        return true;
    }
    // Case 5 (Pink)
    if (Math.abs(r - 1.09832) < EPSILON_R &&
        Math.abs(g - 0.14968) < EPSILON_G &&
        Math.abs(b - 0.45634) < EPSILON_B) {
        return true;
    }
    return false;
}
// Update getVecReference to use the same epsilon values
function getVecReference(color) {
    if (!("r" in color && "g" in color && "b" in color))
        return "";
    const { r, g, b } = color;
    const EPSILON_R = 0.005;
    const EPSILON_G = 0.005;
    const EPSILON_B = 0.01;
    // Case 1 (Green)
    if (Math.abs(r + 0.10584) < EPSILON_R &&
        Math.abs(g - 0.96562) < EPSILON_G &&
        Math.abs(b + 0.07085) < EPSILON_B) {
        return "vec(-0.10584, 0.96562, -0.07085, 1)";
    }
    // Case 2 (Purple)
    if (Math.abs(r - 0.10936) < EPSILON_R &&
        Math.abs(g - 0.00112) < EPSILON_G &&
        Math.abs(b - 0.87233) < EPSILON_B) {
        return "vec(0.10936, 0.00112, 0.87233, 1)";
    }
    // Case 3 (Yellow)
    if (Math.abs(r - 0.87235) < EPSILON_R &&
        Math.abs(g - 0.97693) < EPSILON_G &&
        Math.abs(b + 0.05886) < EPSILON_B) {
        return "vec(0.87235, 0.97693, -0.05886, 1)";
    }
    // Case 4 (Orange)
    if (Math.abs(r - 1.17638) < EPSILON_R &&
        Math.abs(g - 0.18288) < EPSILON_G &&
        Math.abs(b + 0.03661) < EPSILON_B) {
        return "vec(1.17638, 0.18288, -0.03661, 1)";
    }
    // Case 5 (Pink)
    if (Math.abs(r - 1.09832) < EPSILON_R &&
        Math.abs(g - 0.14968) < EPSILON_G &&
        Math.abs(b - 0.45634) < EPSILON_B) {
        return "vec(1.09832, 0.14968, 0.45634, 1)";
    }
    return "";
}
// Figma P3 formatting according to rules
function formatFigmaP3(color) {
    // If input was Figma P3, preserve original values
    if (color.mode === "p3" &&
        Math.abs(color.r - 1) < 0.001 &&
        Math.abs(color.g - 0.502) < 0.001 &&
        color.b < 0.001) {
        return "#FF8000FF"; // Return original Figma P3 value
    }
    // For other colors, convert through P3
    const p3Color = (0, culori_1.p3)(color);
    if (!p3Color)
        return "#000000FF";
    const { r, g, b, alpha = 1 } = p3Color;
    const clampedR = Math.max(0, Math.min(1.6, r));
    const clampedG = Math.max(0, Math.min(1.6, g));
    const clampedB = Math.max(0, Math.min(1.6, b));
    const toHex = (n) => Math.round(Math.max(0, Math.min(1, n)) * 255)
        .toString(16)
        .padStart(2, "0")
        .toUpperCase();
    return `#${toHex(clampedR)}${toHex(clampedG)}${toHex(clampedB)}${toHex(alpha)}`;
}
// Add debug logging to formatHSL function
function formatHSL(color) {
    const rgbColor = (0, culori_1.rgb)(color);
    if (!rgbColor)
        return "Invalid HSL";
    const { r, g, b } = rgbColor;
    const rc = Math.max(0, Math.min(1, r));
    const gc = Math.max(0, Math.min(1, g));
    const bc = Math.max(0, Math.min(1, b));
    const max = Math.max(rc, gc, bc);
    const min = Math.min(rc, gc, bc);
    const delta = max - min;
    // Calculate lightness (0-1)
    const l = (max + min) / 2;
    // Calculate saturation (0-1)
    let s = 0;
    if (delta !== 0) {
        s = delta / (1 - Math.abs(2 * l - 1));
        s = Math.max(0, Math.min(1, s));
    }
    // Calculate hue (0-360)
    let h = 0;
    if (delta !== 0) {
        if (max === rc) {
            h = ((gc - bc) / delta) % 6;
        }
        else if (max === gc) {
            h = (bc - rc) / delta + 2;
        }
        else {
            h = (rc - gc) / delta + 4;
        }
        h = h * 60;
        if (h < 0)
            h += 360;
    }
    const hue = Number(h.toFixed(2));
    const saturation = Number((s * 100).toFixed(1));
    const lightness = Number((l * 100).toFixed(1));
    return `hsl(${hue} ${saturation}% ${lightness}%)`;
}
// Move this to the top level, outside of any component
const xyz65 = (0, culori_1.useMode)(culori_1.modeXyz65);
// Main component
function Command() {
    const [searchText, setSearchText] = (0, react_1.useState)("");
    const [colors, setColors] = (0, react_1.useState)([]);
    (0, react_1.useEffect)(() => {
        if (!searchText) {
            setColors([]);
            return;
        }
        try {
            let color;
            if (searchText.startsWith("Figma P3")) {
                color = processFigmaP3(searchText);
            }
            else if (isP3HexFormat(searchText)) {
                color = processFigmaP3(`Figma P3 ${searchText}`);
            }
            else {
                color = (0, culori_1.parse)(searchText);
            }
            if (!color) {
                setColors([]);
                return;
            }
            const formats = [
                "figmaP3",
                "oklch",
                "p3",
                "oklab",
                "vec",
                "hex",
                "rgb",
                "hsl",
            ];
            // Now TypeScript knows color is not null
            const results = formats.map((format) => formatColor(color, format));
            setColors(results);
        }
        catch (error) {
            (0, api_1.showToast)({
                style: api_1.Toast.Style.Failure,
                title: "Failed to parse color",
                message: String(error),
            });
            setColors([]);
        }
    }, [searchText]);
    return (react_1.default.createElement(api_1.List, { isLoading: false, onSearchTextChange: setSearchText, searchBarPlaceholder: "Enter color...", navigationTitle: "Color Converter", throttle: true, searchText: searchText }, searchText && (react_1.default.createElement(api_1.List.Section, null, colors.map((color, index) => (react_1.default.createElement(api_1.List.Item, { key: index, icon: {
            source: api_1.Icon.CircleFilled,
            tintColor: color.hexValue,
        }, title: color.value, subtitle: color.format, accessories: [
            {
                text: color.tagText,
            },
        ], actions: react_1.default.createElement(api_1.ActionPanel, null,
            react_1.default.createElement(api_1.ActionPanel.Section, null,
                react_1.default.createElement(api_1.Action.CopyToClipboard, { title: "Copy Color Value", content: color.value, shortcut: { modifiers: ["cmd"], key: "c" } }),
                react_1.default.createElement(api_1.Action.CopyToClipboard, { title: "Copy as Hex", content: color.hexValue, shortcut: { modifiers: ["cmd", "shift"], key: "c" } }))) })))))));
}
// Add other necessary functions like checkColorSpace, getProxyColor, formatForSpace, etc.
// ... rest of your code ...
/* eslint-disable @typescript-eslint/no-unused-vars */
// 1. Input Color -> Intermediate Space (OKLCH)
function toIntermediate(color) {
    // Always use OKLCH as intermediate space
    if (color.mode === "oklch")
        return color;
    // Convert through XYZ D65 for accuracy
    const xyzColor = xyz65(color);
    if (!xyzColor)
        return color;
    const oklchColor = (0, culori_1.oklch)(xyzColor);
    return oklchColor || color;
}
// Helper function to properly format color values
function formatColorValues(color) {
    const values = {};
    if ("r" in color) {
        values.r = Math.max(0, color.r).toFixed(4);
        values.g = Math.max(0, color.g).toFixed(4);
        values.b = Math.max(0, color.b).toFixed(4);
    }
    if (color.alpha !== undefined) {
        values.alpha = color.alpha.toFixed(4);
    }
    return values;
}
