# TraceCraft — Branding Guide

## Brand Identity

**Tagline:** *Craft your circuits.*

**Brand Personality:**
- **Precise** — zero tolerance for slop, like a well-routed PCB
- **Craftsperson-like** — celebrates the artistry in board design
- **Approachable** — complex tools don't need to look like fighter jet cockpits
- **Indie/maker** — built by engineers, for engineers

**Tone of Voice:**
- Technical but not academic
- Direct, no fluff
- "We trust you know what you're doing"
- Error messages: helpful, not patronizing

---

## Color Palette

### Primary — Copper / Circuit

```
Copper (accent)    #D4783C  — warm, conductive energy
Dark Copper        #A55A2A  — hover states, depth
```

### Background — Deep Tech

```
BG Deep            #1A1B2E  — main background (dark mode primary)
BG Surface         #232540  — cards, sidebar, panels
BG Elevated        #2D2F52  — hovered items, dropdowns
BG Canvas          #0F1020  — schematic/PCB canvas area
```

### UI — Functional

```
Text Primary       #E8E9F3  — body text
Text Secondary     #9496B8  — labels, hints
Text Muted         #5C5E7E  — disabled, placeholders
Border             #343760  — dividers, card borders
Border Subtle      #282A4A  — subtle separators
```

### Semantic — Status

```
Error              #E6454A  — DRC errors, validation failures
Warning            #F0A030  — DRC warnings
Success            #42BE65  — DRC pass, netlist sync OK
Info               #5096E0  — informational, selection highlight
```

### Layer Colors (PCB Canvas)

```
Top Copper         #D47A2F  — warm copper tone
Bottom Copper      #A0793F  — slightly muted for depth
Inner Signal       #6F9E4A  — greenish signal layer
Inner Power        #B34343  — red/maroon power layer
Solder Mask Top    #30853A  — classic green soldermask
Solder Mask Bottom #25682D  — darker for bottom
Silkscreen         #F0F0F0  — white silkscreen
```

---

## Typography

### Primary Font: Inter

```
Weights: 400 (Regular), 500 (Medium), 600 (Semi-Bold), 700 (Bold)
Used for: UI text, labels, tool tips, property values
Fallback: system-ui, -apple-system, sans-serif
```

Why Inter: excellent legibility at small sizes, tight spacing, technical feel.

### Monospace Font: JetBrains Mono

```
Weights: 400 (Regular), 500 (Medium)
Used for: net names, pin numbers, coordinates, code-like data
Fallback: "Fira Code", "Cascadia Code", monospace
```

Why JetBrains Mono: ligatures for technical notation, clear distinction between
similar characters (1/l/I, 0/O).

---

## Logo Concept

### Symbol: "The Trace"

An abstract mark formed by a single continuous 45° PCB trace that bends three times,
creating a stylized "T" shape. The trace has:
- Rounded corners (like real PCB traces)
- A via circle at both endpoints (start and end)
- Subtle copper color gradient

```
    ┌─────┐
    │     │
    └──┐  │
       │  │
    ┌──┘  │
    │     │
    └─────┘
```

(In reality, rendered as smooth 45° angled trace segments with circular via pads.)

### Logo Lockups

1. **Icon only** (32×32, 64×64, 128×128) — app icon, favicon
2. **Icon + wordmark** (horizontal) — splash screen, about dialog
3. **Icon vertical** — (icon above wordmark) — installer, dock/taskbar

### Logo Variations

- **Full color** — copper trace on dark background (primary)
- **Monochrome** — white trace on transparent (for light backgrounds)
- **Small** — simplified 2-segment trace (for 16×16 favicon)

---

## Iconography

Use **outline-style icons** with 1.5px stroke weight. The icon set should feel
technical and precise — like tools in a lab.

- **Schematic mode:** logic gate / transistor outline
- **PCB mode:** board with traces icon
- **3D preview:** cube with PCB inside
- **Library:** bookshelf / component drawer
- **AI copilot:** sparkle / chip with brain
- **Tools:** cursor (select), pencil (wire), + (component), pen (trace), fill (zone)
- **DRC:** checkmark in circle (pass), x in circle (violation)

**Preferred icon library:** Lucide (already in dependencies) — matches the clean,
outline style. Supplement with custom SVG icons for PCB-specific tools.

---

## UI Design Principles

### Layout
- **Dark-first** — design for dark mode, light mode is secondary
- **Maximize canvas** — tools and panels are secondary to the work area
- **Keyboard-driven** — every tool has a single-key shortcut; mouse is for positioning

### Spacing
- Tight (4px/8px grid) — PCB tools are information-dense
- 4px base unit, 8px preferred step

### Motion
- Subtle transitions (150ms ease)
- No gratuitous animation
- Zoom/pan: instant, no animation (precision tool)

### Component Styles
- Buttons: no background by default, filled on hover, solid when active
- Panels: dark surface (BG Surface), subtle border (Border), rounded corners (8px)
- Inputs: flat, no border until focused (focus ring = Info color)
- Canvas: pure black (BG Canvas) with dim grid (0.05 opacity)

---

## Application Identity

- **Window title:** TraceCraft — `[project name]`
- **Splash screen:** Logo centered on BG Deep, "Loading..." in Text Secondary
- **About dialog:** Logo, version, "Craft your circuits.", link to docs
- **File icon (.tcproj):** Small logo mark with copper-colored overlay

---

## Dark Mode (Default)

This is the primary mode. All colors above are dark mode values.

### Light Mode (Future)

Not an MVP concern, but rough translations for primary colors:
- BG: white → light gray (F5F5F7)
- Text: near-black (1D1D1F)
- Copper accent unchanged (pops on light bg)
- Canvas: very light grid on white

---

## Examples

### Color application

```
┌──────────────────────────────────────┐
│ [Toolbar — BG Surface #232540]       │
├──────────────────────────────────────┤
│                                      │
│    [Canvas — BG Canvas #0F1020]      │
│                                      │
│    ┌──┐                              │
│    │  │  [Selection — #5096E0 30%]   │
│    └──┘                              │
│                                      │
│    [DRC Error — #E6454A dot]         │
│                                      │
├──────────────────────────────────────┤
│ StatusBar — BG Surface #232540       │
└──────────────────────────────────────┘
```

### Typography scale

```
Heading 1: Inter 24px / 32px Semi-Bold
Heading 2: Inter 18px / 24px Semi-Bold
Heading 3: Inter 14px / 20px Medium
Body:      Inter 13px / 18px Regular
Small:     Inter 11px / 16px Regular
Mono:      JetBrains Mono 13px / 18px Regular
Mono Small:JetBrains Mono 11px / 16px Regular
```
