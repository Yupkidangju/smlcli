---
name: Terminal Precision
colors:
  surface: '#051424'
  surface-dim: '#051424'
  surface-bright: '#2c3a4c'
  surface-container-lowest: '#010f1f'
  surface-container-low: '#0d1c2d'
  surface-container: '#122131'
  surface-container-high: '#1c2b3c'
  surface-container-highest: '#273647'
  on-surface: '#d4e4fa'
  on-surface-variant: '#cbc3d7'
  inverse-surface: '#d4e4fa'
  inverse-on-surface: '#233143'
  outline: '#958ea0'
  outline-variant: '#494454'
  surface-tint: '#d0bcff'
  primary: '#d0bcff'
  on-primary: '#3c0091'
  primary-container: '#a078ff'
  on-primary-container: '#340080'
  inverse-primary: '#6d3bd7'
  secondary: '#bec6e0'
  on-secondary: '#283044'
  secondary-container: '#3f465c'
  on-secondary-container: '#adb4ce'
  tertiary: '#ffb869'
  on-tertiary: '#482900'
  tertiary-container: '#ca801e'
  on-tertiary-container: '#3f2300'
  error: '#ffb4ab'
  on-error: '#690005'
  error-container: '#93000a'
  on-error-container: '#ffdad6'
  primary-fixed: '#e9ddff'
  primary-fixed-dim: '#d0bcff'
  on-primary-fixed: '#23005c'
  on-primary-fixed-variant: '#5516be'
  secondary-fixed: '#dae2fd'
  secondary-fixed-dim: '#bec6e0'
  on-secondary-fixed: '#131b2e'
  on-secondary-fixed-variant: '#3f465c'
  tertiary-fixed: '#ffdcbb'
  tertiary-fixed-dim: '#ffb869'
  on-tertiary-fixed: '#2c1700'
  on-tertiary-fixed-variant: '#673d00'
  background: '#051424'
  on-background: '#d4e4fa'
  surface-variant: '#273647'
typography:
  code-block:
    fontFamily: jetbrainsMono
    fontSize: 14px
    fontWeight: '400'
    lineHeight: 20px
  ui-bold:
    fontFamily: jetbrainsMono
    fontSize: 14px
    fontWeight: '700'
    lineHeight: 20px
  ui-italic:
    fontFamily: jetbrainsMono
    fontSize: 14px
    fontWeight: '400'
    lineHeight: 20px
  label-sm:
    fontFamily: jetbrainsMono
    fontSize: 12px
    fontWeight: '600'
    lineHeight: 16px
  header-title:
    fontFamily: jetbrainsMono
    fontSize: 16px
    fontWeight: '800'
    lineHeight: 24px
rounded:
  sm: 0.25rem
  DEFAULT: 0.5rem
  md: 0.75rem
  lg: 1rem
  xl: 1.5rem
  full: 9999px
spacing:
  block-gap: 1.5rem
  container-padding-x: 2rem
  container-padding-y: 1rem
  chip-gap: 0.75rem
  composer-height: 4rem
---

## Brand & Style

The design system is a specialized framework for modern Terminal User Interfaces (TUI), prioritizing engineering efficiency and developer focus. It bridges the gap between the raw utility of a CLI and the polished ergonomics of a modern IDE. 

The visual style is **Minimalist / Modern**, utilizing a dark-mode-only foundation to reduce eye strain during deep-work sessions. It emphasizes logical grouping over decorative elements, using structural borders and semantic color coding to guide the user's attention through complex command outputs and multi-step interactions. The aesthetic is intentionally "low-noise," allowing code and technical data to remain the primary focus.

## Colors

The color strategy for the design system is built on a deep navy hierarchy (`bg_base` to `bg_layer`) to provide depth without breaking the terminal's monochromatic soul. 

- **Primary/Accent:** Purple is used sparingly for focus states, primary prompts, and the active cursor in the Composer.
- **Semantic Logic:** Standardized colors (Blue, Green, Yellow, Red) are reserved for status chips and tool outputs to provide immediate cognitive recognition of system states.
- **Contrast:** High-contrast whites (#F8FAFC) and light grays are used for primary text and code literals, while mid-tones are used for metadata and comments to establish a clear information hierarchy.

## Typography

This design system exclusively uses monospaced typography to ensure perfect alignment in terminal grids. Rather than varying font sizes—which is often unsupported or jarring in a TUI—hierarchy is established through weight and style modifiers.

- **Bold (Weight 700):** Used for command triggers, key labels in the header, and primary user inputs.
- **Italic:** Reserved for secondary metadata, such as file paths, timestamps, or "dimmed" suggestions in the command palette.
- **Code:** Code blocks use standard weight but rely on the semantic color palette for syntax highlighting.
- **Alignment:** All text should adhere to a strict character-cell grid.

## Layout & Spacing

The layout follows a **Block-based Logic**, where every interaction is a distinct horizontal card spanning the full width of the terminal.

- **Vertical Rhythm:** Generous padding (`block-gap`) is applied between interaction blocks to prevent the "wall of text" effect common in legacy CLIs.
- **Adaptive Header:** The header uses a flexible flexbox-style layout. When terminal width drops below 80 characters, it drops secondary metadata (CWD, Session ID) to preserve space for the primary status and mode.
- **The Composer:** Always pinned to the bottom of the viewport, the Composer acts as the anchor of the UI, utilizing a fixed height with internal status chips for Mode and Policy.

## Elevation & Depth

In a TUI environment, traditional shadows are unavailable. The design system uses **Tonal Layers** and **Unicode Borders** to create depth.

- **Base Layer:** The terminal background (`bg_base`).
- **Surface Layer:** Interaction blocks and cards use a slightly lighter background (`bg_surface`) or are defined by a single-pixel unicode border.
- **Overlay:** The Command Palette (Ctrl+K) uses the highest contrast background (`bg_layer`) and is centered on the screen, effectively "floating" over the interaction history.
- **Borders:** Use rounded unicode box-drawing characters (╭ ╮ ╯ ╰) for a modern feel. If the environment does not support UTF-8, the system must fallback to standard ASCII (+ - |).

## Shapes

The shape language is primarily rectangular but softened with **Rounded Unicode Corners**. This distinguishes the UI from standard blocky terminal outputs. 

- **Blocks:** Large containers use a 0.5rem visual equivalent (rounded unicode corners).
- **Chips:** Status indicators (Mode, Policy) use pill-shaped logic where possible, or brackets `[ ]` as a fallback.
- **Inputs:** The Composer field should feel "inset," achieved by using a solid color block for the active input area versus the surrounding frame.

## Components

### Interaction Blocks (Turns)
The primary unit of the design system. Each block represents one turn of conversation or tool execution. It features a left-side vertical accent line (Purple for user, Blue for system) to group input and output visually.

### Command Palette
A modal component triggered by `Ctrl+K`. It features a fuzzy-search input at the top and a scrollable list of actions below. Highlighted items use the `accent` background with high-contrast text.

### Status Chips
Compact elements used in the Toolbar/Composer. They use background colors to indicate state:
- **Mode:** `accent` (Purple) background.
- **CWD:** `neutral` (Gray) border.
- **Policy:** `warning` (Yellow) or `success` (Green) text indicators.

### Composer
The persistent input area. It includes a multi-line text field with syntax highlighting for commands and a toolbar row immediately above or below it for quick-toggle status settings.

### Input Fields & Checkboxes
- **Inputs:** Shown with a block cursor (█) and a distinct background color.
- **Checkboxes:** Represented by `[x]` and `[ ]` using `success` and `neutral` colors respectively.