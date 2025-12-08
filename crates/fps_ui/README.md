## 💻 `fps_ui` Crate Overview: A Caller-Driven Toolkit

The **`fps_ui`** crate is designed as a **single-threaded, high-performance UI toolkit** built on top of `winit` and `pixels`. It focuses on providing a stable rendering canvas and high-level $\text{UI}$ components, allowing the application (the caller) to remain in full control of the game state and event execution.

### 1. Single-Threaded Architecture and Core Abstraction 🚀

The primary mission of this crate is to **extend the low-level functionalities** of three core libraries into a unified, high-level $\text{UI}$ system:

* **`winit`** is extended for **event handling**, translating raw $\text{OS}$ events (like mouse clicks and window resizing) into structured **`UIEvent`** messages.
* **`pixels`** is extended for **drawing**, providing abstract rendering functions (e.g., `draw_rect`, `draw_text`) atop the raw pixel buffer.
* **`glam`** is extended for **geometry**, providing the foundation for reliable positioning, scaling, and hit-testing across all components.

The $\text{UI}$ logic runs entirely on a single thread, driven by the $\text{winit}$ event loop (`run_event_loop`). This thread handles all window management, input polling, and rendering.

* **Caller-Driven Loop:** The `fps_ui` crate does **not** manage application logic. Instead, the application calls the `UIManager` methods within the $\text{winit}$ loop.
* **Event Consumption:** The `UIManager::process_input()` method translates raw $\text{winit}$ events into a list of high-level **`UIEvent`** messages (e.g., `ButtonClicked`, `TextSubmitted`). The application logic in `main.rs` must **consume and handle** these events accordingly (e.g., switching game modes, sending data over `fps_net`).
* **Efficiency:** By running the $\text{UI}$ on a dedicated thread and relying on the $\text{OS}$ to fire events, the crate minimizes $\text{CPU}$ consumption during idle periods.

---

### 2. Window and Initialization 🖼️

The crate provides the necessary functionality to establish the visual environment based on caller-supplied parameters.

* **Window Creation:** The $\text{UI}$ exposes methods (via `ui_window.rs`) to create the application window using `winit` and initialize the GPU-backed pixel buffer using `pixels`.
* **Parameter-Based Setup:** The application provides initial parameters (e.g., width, height, title) to the $\text{UI}$ initialization routines, guaranteeing a consistent starting canvas size.

---

### 3. Layered Component System (Layout and Grouping) 🧱

The layout system is built around layers and reusable components, enabling complex $\text{UI}$ structures.

* **Layers for Organization:** The application can define **multiple layers** (`UILayer`) within the `UIManager`. Layers serve two primary purposes:
    1.  **Z-Ordering:** Layers are drawn based on their **`z_index`**, guaranteeing the correct render order (e.g., drawing the main menu overlay on top of the $\text{HUD}$).
    2.  **Pooling and Visibility:** Layers are used for logical grouping, allowing the application to toggle the visibility of entire $\text{UI}$ sets (e.g., hiding the "Main Menu" layer and showing the "In Game $\text{HUD}$" layer).
* **Multiple Components:** Each defined layer can contain **multiple components** ($\text{Button}$, $\text{Label}$, $\text{TextField}$, etc.).

---

### 4. Component Drawing and Redraw Logic 🎨

The `fps_ui` crate handles the low-level rendering, but the redraw timing is managed intelligently.

* **Standard Elements:** The crate provides implementations for standard $\text{UI}$ elements, abstracting away the tedious pixel manipulation (using `ui_renderer.rs` primitives). The user defines the component's position, size, and data, and the component handles its own drawing logic (e.g., a $\text{Button}$ knows how to draw its background, border, and center its text).
* **Event-Driven Redrawing:** Redrawing is **not** constant. The system only requests a new frame when a visual change occurs, such as:
    * A **`WindowEvent::Resized`** event.
    * An **internal $\text{UI}$ state change** (e.g., the mouse moving over a $\text{Button}$ triggers the component to change its `is_hovered` state).
    * The application explicitly requests a redraw after updating dynamic data (e.g., a score change).

---

### 5. Positioning and Geometry 📐

The $\text{UI}$ supports flexible positioning that adapts to different screen sizes.

* **Absolute Position:** Components are primarily positioned using **absolute pixel coordinates** $(x, y)$ from a defined anchor point (e.g., top-left).
* **Relative Positioning:** Positioning logic uses the **whole window size** (read from `WindowContext`) to calculate final coordinates. This allows components to be positioned relative to the screen size (e.g., "always center," or "place $\text{10}$ pixels from the bottom-right corner"), enabling the $\text{UI}$ to scale correctly when the window is resized.

-----

## 📁 `fps_ui` Crate File Structure

```
│   ├── fps_ui/
│   │   └── src/
│   │       ├── lib.rs                  # Crate entry point and public API exports.
│   │       ├── fonts.rs                # Font loading and management (ab-glyph).
│   │       ├── context.rs              # UIMainContext: Global, application-wide resources.
│   │       ├── window.rs               # WindowContext, winit/pixels initialization, and the main event loop driver.
│   │       ├── input.rs                # Winit event translation and component internal state updates.
│   │       ├── layers.rs               # UIManager and UILayer: State management, ordering, and orchestration.
│   │       ├── events.rs               # UIEvent: Definition of the actionable messages for the caller.
│   │       ├── renderer.rs             # Low-level drawing primitives (rects, text) using the pixels buffer.
│   │       ├── components/
│   │       │   ├── mod.rs              # Component trait and common visual state definitions.
│   │       │   ├── component.rs        # (Typically merged into mod.rs or traits defined here)
│   │       │   ├── button.rs           # Button component implementation.
│   │       │   ├── label.rs            # Label component implementation.
│   │       │   ├── text_field.rs       # Text input component implementation.
│   │       │   └── dynamic_pixel.rs    # Custom component for dynamic, caller-supplied pixel data (e.g., mini-map).
│   │       └── geometry.rs             # 2D geometry, hit-testing, and positioning utilities (using glam).
```

-----

## 🔍 Module Responsibilities Overview

### A. Context and Initialization (The Foundation)

  * **`lib.rs`**: **Public API.** Declares all modules and re-exports the main structs (`UIManager`, `WindowContext`, `UIEvent`) for easy consumption by the application.
  * **`window.rs`**: **Loop Driver.** Contains **`WindowContext`** (holds `winit` and `pixels`) and the **`run_event_loop()`** function, which drives the single-threaded execution and handles $\text{OS}$-level window events.
  * **`context.rs`**: **Global State.** Defines **`UIMainContext`**, a container for resources like the `FontManager` and global settings, passed through the render cycle.
  * **`fonts.rs`**: **Asset Manager.** Handles the loading and management of font assets, providing rasterized glyph data to the renderer.

### B. Logic and State Management (The Engine)

  * **`layers.rs`**: **Orchestrator.** Defines **`UIManager`** and **`UILayer`**. Manages component collection, $\text{Z-ordering}$, visibility pooling, and exposes the primary methods: **`process_input()`** and **`render()`**.
  * **`input.rs`**: **Input Translator.** Consumes raw `winit` input events. It performs hit-testing on components and updates their **internal visual state** (`is_hovered`, `is_focused`).
  * **`events.rs`**: **Message Definition.** Defines the **`UIEvent`** enum, representing the actionable messages (`ButtonClicked`, `TextSubmitted`) returned to the application caller.

### C. Rendering and Components (The Visuals)

  * **`renderer.rs`**: **Drawing Interface.** Provides the low-level, high-performance functions (`draw_rect`, `draw_text`) that abstract the raw pixel manipulation of the `pixels` frame buffer.
  * **`components/mod.rs` & `component.rs`**: **Contract.** Define the core **`Component`** trait and required methods (`draw`, `bounds`, `handle_input`), which all specific components must implement.
  * **`components/...`**: **Implementations.** Contains the specific data and drawing logic for each standard $\text{UI}$ element.
  * **`utils/geometry.rs`**: **Math.** Provides necessary geometry structs (`Rect`) and functions (`contains_point`) for accurate layout and input detection.