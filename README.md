# OKLCH Color Converter for Tezbar

Convert colors between sRGB, wide-gamut, and linear formats with out-of-gamut handling in Tezbar on Windows or macOS.

## Command

- **Convert Color** — view with searchable input that parses any CSS color; shows a list of converted formats (`rgb`, `hex`, `hex/rgba`, `hsl`, `p3`, `oklch`, `oklab`, `vec`/`lrgb`, `figmaP3`) with detail preview, space badges (`sRGB`, `P3`, `Out`), and copy actions. Reference calculations are documented in `reference-calculations.md`.

Supported inputs include `rgb(...)`, `#RRGGBB`, `hsl(...)`, `color(display-p3 ...)`, `oklch(...)`, Figma P3 strings, and linear `vec(...)` values.

## Compatibility

- Windows: built and exercised in Tezbar on Windows 11.
- macOS: uses only cross-platform `@raycast/api` and `culori` (pure JS color math) and retains the upstream macOS-compatible design.
- Linux: not currently declared.

## Local verification

```powershell
pnpm install
pnpm typecheck
pnpm test
pnpm build
```

## Credits

Adapted for Tezbar from Raycast's MIT-licensed `oklch-color-converter` extension at upstream commit `6734d85992290066fd31a7f5bee3cca3d3545a1a`. Original author `strongeron` remains credited in `package.json`.
