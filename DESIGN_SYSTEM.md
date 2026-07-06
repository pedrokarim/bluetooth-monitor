# Bluetooth Monitor — Design System

> Inspired by the dashboard mockup: **deep-violet gradient**, **warm accent palette**, **airy typography**, **pill-shaped controls**, **circular data viz** (donut).

---

## 1. Design principles

1. **Data first, chrome second.** No borders where a color-shift or spacing already reads. Frames only when they group truly distinct blocks.
2. **Warm on cool.** Cool violet base + warm accent colors (teal, coral, yellow, orange). Warmth is reserved for values that carry meaning (battery, actions, live indicators).
3. **Airy.** Prefer generous vertical rhythm and large touch/click targets over dense listings.
4. **Numeric hierarchy.** Numbers are the heroes. Labels are typographic support.
5. **Roundness.** Pills (radius = height) for controls; large soft rounding (14–20 px) for cards. Only avatars and dots use full circles.
6. **One motion of accent per row.** A row/card should carry at most one bright accent so the eye can land.

---

## 2. Color palette

### 2.1 Background (gradient, top → bottom)

| Token | Hex | Role |
|---|---|---|
| `bg.top` | `#2A1C40` | Deep violet — top of window |
| `bg.mid` | `#44275A` | Mid stop |
| `bg.bot` | `#7A406F` | Warm mauve — bottom of window |

The window background is a **3-stop vertical linear gradient** with two decorative low-opacity white circles (bokeh) at ~85% bottom-left and ~15% top-right.

### 2.2 Surfaces

| Token | Hex | Role |
|---|---|---|
| `surface.card` | `#382655` | Standard card |
| `surface.card_strong` | `#48306A` | Card on hover / selected tab |
| `surface.outline` | `#5A3C7A` | 1 px hairline outline (subtle) |
| `surface.pill` | `#2D1C45` | Inactive pill / segmented track |

### 2.3 Text

| Token | Hex | Alpha | Role |
|---|---|---|---|
| `text.default` | `#F7EFF6` | 100% | Primary text, big numbers |
| `text.muted` | `#C8B8D8` | ~78% | Labels, sub-titles |
| `text.dim` | `#9A88AA` | ~55% | Captions, disabled, units |

### 2.4 Accent palette (warm)

Used for status, values, donut rings, device dots. Cycle in this order when assigning per-device colors.

| Token | Hex | Semantic pairing |
|---|---|---|
| `accent.teal` | `#4CDDCF` | Excellent / connected / battery ≥ 60% |
| `accent.coral` | `#F78473` | Destructive / low battery |
| `accent.yellow` | `#F5C74A` | Warning / battery 30–59% |
| `accent.orange` | `#F6935C` | Attention / battery 15–29% |
| `accent.purple` | `#C98CF1` | Trusted / hover states |
| `accent.pink` | `#EA76B1` | Extra category |
| `accent.red` | `#EF5A6F` | Blocked / critical |
| `accent.green` | `#8FE09A` | Positive confirmations |

**Rule.** No hard-coded color per device — always pick from the cycle by index of connected order. This keeps the donut and the list visually aligned.

### 2.5 Battery gradient (semantic)

| Range | Color |
|---|---|
| ≥ 60% | `accent.teal` |
| 30–59% | `accent.yellow` |
| 15–29% | `accent.orange` |
| < 15% | `accent.coral` |

---

## 3. Typography

Single family: **egui default proportional** (Ubuntu-derived). Sizes ladder:

| Token | Size | Weight | Tracking | Usage |
|---|---|---|---|---|
| `type.hero` | 40 px | Regular | — | Big stat numbers ("2", "70%") |
| `type.title` | 22 px | Bold | — | Screen titles ("BLUETOOTH") — uppercase |
| `type.subtitle` | 14 px | Regular | wide | Section headers ("Devices") |
| `type.body` | 13 px | Regular | — | Device names, values |
| `type.label` | 11 px | Regular | wide | Small labels above numbers |
| `type.caption` | 10.5 px | Regular | — | Address, UUIDs (monospace), units |

- Uppercase for **titles and micro-labels** only.
- Never use bold for body text — hierarchy through **size + color opacity**, not weight.
- Numbers: `type.hero` sits directly above `type.label`, no spacing except one line.

---

## 4. Spacing & rhythm

Base unit: **4 px**.

| Token | Value | Role |
|---|---|---|
| `space.1` | 4 px | Icon ↔ text |
| `space.2` | 8 px | Row internal gap |
| `space.3` | 12 px | Between related rows |
| `space.4` | 16 px | Between blocks in a card |
| `space.5` | 24 px | Between sections |
| `space.6` | 32 px | Around hero numbers |
| `space.7` | 48 px | Top of window / hero blocks |

Cards use **inner margin 20 px horizontal / 18 px vertical**.

Vertical rhythm on lists: one device row = **56 px minimum height**, breathable.

---

## 5. Shape

| Token | Radius | Role |
|---|---|---|
| `radius.card` | 18 px | Cards, panels |
| `radius.card_sm` | 12 px | Nested/compact cards |
| `radius.pill` | 999 px (fully rounded) | Buttons, tabs, badges |
| `radius.chip` | 10 px | Small chips (rare) |
| `radius.dot` | 999 px | Legend dots (always circle) |

**Hairlines** — 1 px stroke `surface.outline` at 60% alpha. Reserved for card edges when contrast with background is too low.

---

## 6. Components

### 6.1 Segmented tabs (pill)

- Full pill container (`radius.pill`) filled with `surface.pill`.
- Each option is text-only unless selected.
- Selected option: `surface.card_strong` bg pill inside the track, `text.default`, no border.
- Unselected: `text.muted` on transparent.
- Height: 40 px. Padding H: 20 px.

Options for BT Monitor: `ALL · CONNECTED · PAIRED`.

### 6.2 Primary action button

- Pill (`radius.pill`), fill `accent.teal` for constructive, `accent.coral` for destructive (Disconnect, Remove).
- Text: 12.5 px, all-caps optional, color `#1c1230` (dark) on light accent for contrast.
- Height: 32–36 px. Padding H: 18 px.

### 6.3 Ghost button (secondary)

- Pill, transparent bg, 1 px `surface.outline`, text `text.muted`.
- Hover: `surface.card` bg, `text.default`.

### 6.4 Icon-only chip (round)

- 32 × 32, `surface.card`, icon centered.
- Used in top-right (refresh) and left rail (menu).

### 6.5 Stat block

Layout: label above number, or number above label — number always dominant.

```
Connected                Battery avg
   2                        70%
of 4 devices             across 2 rings
```

- Number: `type.hero` in `text.default`.
- Label above: `type.label` uppercase in `text.dim`.
- Caption below number: `type.caption` in `text.dim`.

### 6.6 Device row (list item)

- Height: 56 px.
- Left: **colored dot** (8 px, `accent.*` cycled) + device name (`type.body`).
- Middle: signal bars (small) OR type icon.
- Right: **battery %** (`type.body` in battery color) + thin battery bar (60 × 4 px).
- Divider between rows: 1 px `surface.outline` at 30% alpha.
- Disconnected: dot becomes hollow ring, name at `text.muted`, no battery bar.

### 6.7 Donut chart

- Multi-ring: **one ring per connected device**.
- Ring width: 8 px, gap between rings: 4 px.
- Ring color: `accent.*` matching the same index used in the device list.
- Background track: same ring width, `surface.pill` at 60% alpha.
- Arc sweep: proportional to battery percentage (100% = full). If battery unknown, dashed full ring at `text.dim`.
- Center label: total connected count (`type.hero`) + "connected" label (`type.label`).
- Diameter: 200 px min, up to 260 px.

### 6.8 Badge (status pill)

- Small pill, 10.5 px text, uppercase.
- Fill: accent × 18% alpha, stroke: accent × 70% alpha.
- Reserved for: `TRUSTED`, `BLOCKED`, `PAIRED`, `CONNECTED`.

### 6.9 Empty state

- Centered.
- Large emoji (48 px) + `type.subtitle` in `text.muted` + `type.caption` hint in `text.dim`.

### 6.10 Toast / error strip

- Bottom of window, full width, 32 px height.
- Fill `accent.coral` at 15% alpha + 1 px stroke.
- Text: `type.body` in `text.default`.

---

## 7. Motion (future-facing)

Not implemented yet, planned:

- Tab switch: 200 ms ease-out translate of selected pill background.
- Donut sweep: 400 ms ease-out on refresh.
- Card enter: 250 ms fade + 8 px slide.
- Refresh spinner: 900 ms rotate on the top-right chip during a refresh.

---

## 8. Iconography

Emoji as first-tier icons because they're cross-platform and colorful — matches the vibe. Reserved icons:

| Icon | Use |
|---|---|
| 🎧 | Audio (headset, buds, speaker) |
| 🖱 | Mouse |
| ⌨ | Keyboard |
| 📱 | Phone |
| 💻 | Computer |
| 🎮 | Gamepad |
| ⌚ | Watch |
| 🔋 | Battery available |
| 🪫 | Battery low or unknown |
| 📶 | Signal |
| ⚡ | Tx power |
| ↻ | Refresh |
| ≡ | Menu |
| ⚙ | Settings |

---

## 9. Layout — main window

Target: 800 × 600 (min 540 × 400).

```
┌─ 24 px pad ───────────────────────────────────────┐
│  ≡    BLUETOOTH                          ↻       │  ← 48 px header
│                                                  │
│         ╭─ ALL ╌ CONNECTED ╌ PAIRED ─╮           │  ← 40 px pill
│                                                  │
│  Connected     Signal avg    Battery avg         │  ← stat strip
│    2            −45 dBm         70%              │
│                                                  │
│  ┌─────────────────┐  ┌─────────────────────┐    │
│  │                 │  │  DEVICES  (4)        │    │
│  │   [ DONUT ]     │  │  ● LOGI M240   80% ▓ │    │
│  │                 │  │  ● Redmi Buds  60% ▓ │    │
│  │                 │  │  ○ TM-m30III   —     │    │
│  │  2  connected   │  │  ○ LIFT        —     │    │
│  └─────────────────┘  └─────────────────────┘    │
│                                                  │
└─ status strip ───────────────────────────────────┘
```

- Left column (donut + stats): 45% width, min 320 px.
- Right column (device list): 55% width, scrollable.
- Stat strip is above both columns when window is wide; collapses under the donut when narrow.

---

## 10. Accessibility

- Contrast: text on gradient background must pass ≥ 4.5:1 — the palette is chosen so `text.default` on `bg.mid` = 8.6:1, on `bg.bot` = 4.9:1.
- No color-only semantics: battery uses **color + percentage number + bar length**; connection uses **filled vs hollow dot + label**.
- Focus ring: 2 px `accent.teal` at 80%, offset 2 px. `radius.pill` reused so it hugs any control.
- Minimum interactive size: 32 × 32.

---

## 11. Do / Don't quick reference

| Do | Don't |
|---|---|
| Use color to signal *value*, not decoration | Rainbow-fill icons that carry no meaning |
| Rely on size + opacity for hierarchy | Bold entire words for emphasis |
| Reuse the accent cycle across donut & list | Assign colors ad-hoc per device |
| Round pills fully (height = radius) | Use inconsistent radii (e.g. 6 px then 12 px) |
| Keep 4 px rhythm | Freehand `add_space(7.0)` values |
| Show units next to numbers in `text.dim` | Baseline-shift units by weight/size mismatch |

---

## 12. Open questions

- Should the donut animate on refresh?
- Do we want a compact table view toggle for power users?
- Should the tray icon reflect a critical battery (turn coral)?
