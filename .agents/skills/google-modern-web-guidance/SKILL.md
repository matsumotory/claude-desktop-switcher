# Google Modern Web Guidance (2026 Edition)

## Abstract
This skill enforces the official Google I/O 2026 guidelines for AI agents generating web frontend code. It strictly prohibits legacy CSS and JavaScript hacks, forcing the use of modern, native Web Platform features.

## Core Directives

### 1. Modern CSS Layouts & Anchor Positioning
* **Prohibit Legacy Positioning:** Do not use heavy JavaScript calculations or third-party libraries (like Popper.js) for tooltips, popovers, or floating menus.
* **Enforce CSS Anchor Positioning:** Use the native `anchor()` API for attaching UI elements to anchors.

### 2. Native UI Components (Popover & Dialog)
* **Use the Popover API:** For non-modal overlays (menus, tooltips), ALWAYS use the HTML `popover` attribute. Do not build custom click-outside handlers in JS.
* **Use Native Dialogs:** For modal windows, always use `<dialog>` and `showModal()`. 

### 3. View Transitions API
* **Seamless Navigation:** Implement the View Transitions API (`::view-transition`) for all state changes and page navigations to provide smooth, app-like transitions without complex React/Vue animation libraries.

### 4. Progressive Enhancement & Baseline
* **Follow Baseline 2026:** Ensure all code adheres to the "Baseline 2026" standard, focusing on features widely supported across modern Chrome, Safari, and Firefox. 
* **Accessibility (a11y) First:** All native components used (Dialog, Popover) already manage focus and accessibility trees. Do not override this behavior with custom ARIA hacks unless strictly necessary.
