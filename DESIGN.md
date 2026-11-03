---
name: Clean Engineering Slate
colors:
  surface: '#0b1326'
  surface-dim: '#0b1326'
  surface-bright: '#31394d'
  surface-container-lowest: '#060e20'
  surface-container-low: '#131b2e'
  surface-container: '#171f33'
  surface-container-high: '#222a3d'
  surface-container-highest: '#2d3449'
  on-surface: '#dae2fd'
  on-surface-variant: '#c2c6d6'
  inverse-surface: '#dae2fd'
  inverse-on-surface: '#283044'
  outline: '#8c909f'
  outline-variant: '#424754'
  surface-tint: '#adc6ff'
  primary: '#adc6ff'
  on-primary: '#002e6a'
  primary-container: '#4d8eff'
  on-primary-container: '#00285d'
  inverse-primary: '#005ac2'
  secondary: '#a4c9ff'
  on-secondary: '#00315d'
  secondary-container: '#0267b8'
  on-secondary-container: '#d6e5ff'
  tertiary: '#4edea3'
  on-tertiary: '#003824'
  tertiary-container: '#00a572'
  on-tertiary-container: '#00311f'
  error: '#ffb4ab'
  on-error: '#690005'
  error-container: '#93000a'
  on-error-container: '#ffdad6'
  primary-fixed: '#d8e2ff'
  primary-fixed-dim: '#adc6ff'
  on-primary-fixed: '#001a42'
  on-primary-fixed-variant: '#004395'
  secondary-fixed: '#d4e3ff'
  secondary-fixed-dim: '#a4c9ff'
  on-secondary-fixed: '#001c39'
  on-secondary-fixed-variant: '#004883'
  tertiary-fixed: '#6ffbbe'
  tertiary-fixed-dim: '#4edea3'
  on-tertiary-fixed: '#002113'
  on-tertiary-fixed-variant: '#005236'
  background: '#0b1326'
  on-background: '#dae2fd'
  surface-variant: '#2d3449'
typography:
  headline-lg:
    fontFamily: Inter
    fontSize: 30px
    fontWeight: '700'
    lineHeight: 38px
  headline-md:
    fontFamily: Inter
    fontSize: 22px
    fontWeight: '600'
    lineHeight: 28px
  headline-sm:
    fontFamily: Inter
    fontSize: 18px
    fontWeight: '600'
    lineHeight: 24px
  body-lg:
    fontFamily: Inter
    fontSize: 16px
    fontWeight: '400'
    lineHeight: 24px
  body-md:
    fontFamily: Inter
    fontSize: 14px
    fontWeight: '400'
    lineHeight: 20px
  body-sm:
    fontFamily: Inter
    fontSize: 12px
    fontWeight: '400'
    lineHeight: 16px
  code-lg:
    fontFamily: JetBrains Mono
    fontSize: 15px
    fontWeight: '500'
    lineHeight: 22px
  code-md:
    fontFamily: JetBrains Mono
    fontSize: 13px
    fontWeight: '400'
    lineHeight: 19px
  code-sm:
    fontFamily: JetBrains Mono
    fontSize: 11px
    fontWeight: '400'
    lineHeight: 15px
  label-md:
    fontFamily: JetBrains Mono
    fontSize: 12px
    fontWeight: '600'
    lineHeight: 16px
  label-sm:
    fontFamily: JetBrains Mono
    fontSize: 10px
    fontWeight: '500'
    lineHeight: 14px
rounded:
  sm: 0.25rem
  DEFAULT: 0.5rem
  md: 0.75rem
  lg: 1rem
  xl: 1.5rem
  full: 9999px
spacing:
  gutter: 0.75rem
  gutter-tablet: 1rem
  margin: 1rem
  margin-tablet: 1.5rem
  space-xs: 0.25rem
  space-sm: 0.5rem
  space-md: 0.75rem
  space-lg: 1rem
  space-xl: 1.5rem
---

## Brand & Style

This design system embodies high-density technical precision calibrated specifically for mobile software engineering and compiler diagnostics. It prioritizes deliberate efficiency, operational clarity, and ergonomic touch interaction for complex development workflows.

The visual aesthetic balances Modern Technical Minimalist and Tactile Engineering conventions:
- **Surface Palette**: Tiered slate-graphite architectures establish visual calm, minimizing eye strain across extended debugging and authoring sessions.
- **Electric Precision**: Electric blue acts as the singular high-signal operational accelerator for builds, active terminals, and cursor states.
- **Semantic Code Dialects**: Controlled chromatic accents (emerald, amber, purple, rose) represent machine states, lint severities, and syntax structures.
- **Form Factor Ergonomics**: Replaces conventional mouse-dependent IDE density with generous 44pt+ touch targets, persistent accessory toolstrips, and tactile finger-calibrated controls.

## Colors

The system uses a dark-first color model calibrated for high-contrast mobile readability under variable lighting conditions.

### Surface Tiers
- **Canvas Base (`#0F172A`)**: Root application background, code gutter canvas, and bottom sheet scaffolds.
- **Surface Elevated (`#1E293B`)**: Editor panels, file explorer tree rows, floating toolbars, and modal sheets.
- **Surface Interactive (`#334155`)**: Pressed states, button backgrounds, segmented controls, and unfocused input chrome.
- **Surface Stroke (`#475569`)**: 1px structural hairpins defining layout splits and panel separations.

### Signal Accents
- **Primary Electric (`#3B82F6`)**: Primary CTAs, active Git branches, running compilation status, and focused cursor state.
- **Secondary Cyan Glow (`#60A5FA`)**: Inline symbol hovers, code selections, active tabs, and secondary links.
- **Success / Build Clean (`#10B981`)**: Passing tests, successful deployments, additions in diff view.
- **Warning / Lint Delta (`#F59E0B`)**: Diagnostic warnings, dirty working trees, deprecation markers.
- **Error / Breakpoint (`#F43F5E`)**: Compile failures, hit breakpoints, deleted diff hunks.
- **Keyword / Structural Accent (`#A855F7`)**: Language reserved words, decorator scopes, type interfaces.

## Typography

Typography balances rapid UI navigation with dense, legible code consumption:
- **Inter** handles all chrome, workspace management, navigation titles, settings, and modal controls.
- **JetBrains Mono** powers the code viewport, live terminal outputs, hex viewers, compiler error traces, and virtual accessory keyboard keycaps.
- Optical sizing enforces high x-height clarity on mobile screens. Tabular figures are turned on across both families (`tnum`) to maintain structural vertical alignment inside code gutters, line counters, and memory profiling monitors.

## Layout & Spacing

This layout system leverages a fluid vertical-column hierarchy with strict mobile ergonomics:
- **Viewport Layout**: Full-bleed fluid model using absolute edge-to-edge rendering. Top app bar and bottom accessory toolbars dock via safe-area boundary calculations.
- **The Touch Rhythm**: Component margins and gutters are pinned to an 8pt base grid with a 4pt subgrid (`0.25rem`). Tap zones maintain a mandatory 44px (2.75rem) minimum bounding box, even when visual tokens reflect tighter technical tags or miniature file tabs.
- **Panels & Drawers**: Mobile screens feature overlay sheets and bottom slide-overs for tree navigation, Git diff inspection, and build terminals. Tablets introduce an anchor-collapsible master-detail side dock (320px fixed master, fluid editor detail).

## Elevation & Depth

Visual hierarchy uses physical layered slate planes combined with dark-mode rim illumination:
- **Level 0 (Canvas Base)**: Pure `#0F172A`. Deepest plane for background editor gutters and structural frame margins.
- **Level 1 (Docked Containers & Toolstrips)**: `#1E293B` backed by a 1px top or inner border stroke of `rgba(255, 255, 255, 0.08)`. No shadow.
- **Level 2 (Active Cards & Overlay Panels)**: `#1E293B` surfaced with a crisp 1px stroke of `#334155` and ambient shadow `0 8px 24px -4px rgba(0, 0, 0, 0.6)`.
- **Level 3 (Modal Sheets & Context Menus)**: Glassmorphic slate compound: background `rgba(30, 41, 59, 0.85)` with `backdrop-filter: blur(16px)`, bound by `rgba(96, 165, 250, 0.2)` highlight stroke and deep shadow `0 20px 32px -8px rgba(0, 0, 0, 0.75)`.

## Shapes

The interface balances sharp industrial utility with handheld comfort:
- **Level 2 Roundedness**: Standard interactive components (buttons, text fields, notification toasts, code action sheets) use `0.5rem` (8px) corner radii.
- **Panels & Large Cards**: Floating run consoles, diff inspect tiles, and modal bottom sheets utilize `rounded-lg` (`1rem` / 16px).
- **Keyboard Accessories & Micro-Chips**: Symbol buttons (`{}`, `()`, `=>`, `;`) on the virtual toolstrip use `rounded-sm` (4px) to retain maximum horizontal surface area while avoiding sharp tactile edges.

## Components

### Buttons
- **Primary (Build/Run)**: `#3B82F6` solid fill, `#FFFFFF` text, `0.5rem` radius, 44px min height. Active state scales to `98%` with an outer glow of `0 0 16px rgba(59, 130, 246, 0.4)`.
- **Secondary (Tool Chrome)**: `#1E293B` background, 1px `#334155` border, `#F8FAFC` text.
- **Ghost/Terminal Action**: Transparent background, hovering to `rgba(51, 65, 85, 0.5)`, `#94A3B8` icon/label tone.

### Input Fields & Editor Viewport
- **Code Field**: Multi-line canvas resting on `#0F172A`, line numbering in `#475569` tabular JetBrains Mono. Selected line gets background tint `rgba(59, 130, 246, 0.08)` and a 2px vertical electric blue indicator bar on the left margin.
- **Search & Filter Inputs**: Pill or 8px rounded `#1E293B` surface, embedded with a 1px border of `#334155`. Focused state swaps the border stroke to `#3B82F6` with zero outer ring glow.

### Chips & Diagnostics Pills
- **Diagnostics Badges**: JetBrains Mono 11px uppercase label with 4px border radius. 
  - Error: `rgba(244, 63, 94, 0.15)` fill with `#F43F5E` text and stroke.
  - Warning: `rgba(245, 158, 11, 0.15)` fill with `#F59E0B` text and stroke.
  - Info: `rgba(59, 130, 246, 0.15)` fill with `#60A5FA` text and stroke.

### Lists & File Explorer Trees
- **Tree Rows**: 40px height for reliable touch navigation. Active item displays a background of `rgba(59, 130, 246, 0.12)` with an active text weight of 600 in `#60A5FA`. Inactive items sit at `#94A3B8`. Nested levels indent via strict 16px multiples with subtle guide lines (`rgba(51, 65, 85, 0.4)`).

### Selection Controls
- **Checkboxes & Radios**: 20px squares/circles, `#1E293B` background, 1.5px `#475569` border. Active checked state shifts fill to `#3B82F6` with a pure white check icon or center dot.

### Specialized Developer Components
- **Accessory Keyboard Strip**: Sticky horizontal scroll row pinned above the mobile OS virtual keyboard. Heights at 42px, featuring `#334155` elevated keycaps housing programming tokens (`tab`, `{`, `}`, `[`, `]`, `(`, `)`, `;`, `_`, `=`, `>`, `$`).
- **Breakpoint Toggles**: Circular targets in line gutters. Empty hover/touch target of 24px; active breakpoint forms an illuminated `#F43F5E` dot with a subtle inner core.
- **Log Console Drawer**: Bottom-anchored glassmorphic sheet with integrated filter tabs (All, Out, Err, Network), utilizing 12px JetBrains Mono text and 1px horizontal split borders.
