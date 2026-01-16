# `fps_ui` - Single-Threaded UI Toolkit

A high-performance, single-threaded UI toolkit built on top of `winit` and `pixels`. Designed for game UIs with a caller-driven architecture where the application maintains full control over game state and event execution.

## Features

- **Single-Threaded**: All UI logic runs on one thread for simplicity and performance
- **Caller-Driven**: Application maintains full control over game state
- **Layered Components**: Organize UI with Z-ordering and visibility control
- **Flexible Layout**: Pixel-based and percentage-based positioning with anchor points
- **Efficient Rendering**: Glyph caching, event-driven redraws, and optimized pixel operations
- **Type-Safe**: Strong typing for events, updates, and component interactions

## Quick Start

```rust
use fps_ui::{UIMainContext, UIManager, components::Button, Color, Rect, layout::LayoutMetrics};

// Create UI context
let mut context = UIMainContext::new("fonts/main.ttf", 800.0, 600.0)?;

// Load textures
context.load_texture("button_bg", "assets/button.png")?;

// Create UI manager
let mut manager = UIManager::new(context);

// Create a button component
let button = Button::new(
    "my_button".to_string(),
    "Click Me".to_string(),
    Rect::new(100.0, 100.0, 200.0, 50.0),
    LayoutMetrics::default(),
    Color::BLUE,
);

// Add to a layer
let layer = Layer {
    id: "main".to_string(),
    z_index: 0,
    is_visible: true,
    is_modal: false,
    components: vec![Box::new(button)],
};

manager.add_layer(layer)?;
```

## Architecture

### Design Philosophy

The crate extends low-level functionalities into a unified, high-level UI system:

- **`winit`** → Event handling: Translates raw OS events into structured `UIEvent` messages
- **`pixels`** → Drawing: Provides abstract rendering functions atop raw pixel buffers
- **`glam`** → Geometry: Foundation for positioning, scaling, and hit-testing

### Single-Threaded Execution

All UI logic runs on a single thread, driven by the winit event loop. The application calls `UIManager` methods within the winit loop, maintaining full control.

### Caller-Driven Loop

The crate does not manage application logic. Instead:

1. `UIManager::process_input()` translates raw winit events into `UIEvent` messages
2. The application consumes and handles these events (e.g., switching game modes)
3. The application updates components via `UIManager::apply_updates()`

### Coordinate Systems

The UI uses two coordinate systems:

- **Logical Coordinates**: UI components are positioned in logical space (e.g., 800x600)
- **Physical Coordinates**: The `AppDriver` scales logical to physical pixels for rendering

This allows the UI to scale appropriately when the window is resized.

## Core Concepts

### Layers

Layers organize components with Z-ordering and visibility control:

```rust
let layer = Layer {
    id: "menu".to_string(),
    z_index: 10,
    is_visible: true,
    is_modal: false,
    components: vec![/* ... */],
};
```

### Components

All UI elements implement the `Component` trait:

- **Identification**: Unique ID for updates and events
- **Layout**: Position, size, and layout metrics
- **Input Handling**: Process window events and emit UI events
- **Rendering**: Draw to the frame buffer
- **State Updates**: Apply `ComponentUpdate` messages

### Events

Events flow from components to the application:

```rust
pub enum UIEvent {
    ButtonClicked(String),
    TextSubmitted(String, String),
    // ...
}
```

### Updates

Updates flow from the application to components:

```rust
pub enum ComponentUpdate {
    SetText(String, String),
    SetVisibility(String, bool),
    // ...
}
```

## Layout System

### Anchor Points

Components can be anchored to different points:

- `TopLeft`, `TopRight`, `BottomLeft`, `BottomRight`, `Center`

### Length Modes

- `Px`: Values are in logical pixels
- `Percent`: Values are percentages (0.0 to 1.0)

### Example

```rust
let metrics = LayoutMetrics {
    anchor: AnchorPoint::Center,
    positioning: LengthMode::Percent,
    sizing: LengthMode::Px,
};

// Component will be centered horizontally, positioned 50% down,
// with fixed pixel width/height
```

## Text Rendering

The font system uses glyph caching for efficient text rendering:

```rust
// Text is automatically cached by character/size/color
font_manager.draw_text(
    frame,
    "Hello, World!",
    16.0,           // size
    Color::WHITE,
    x, y,
    width, height,
);
```

## Components

Available components:

- **Button**: Clickable buttons with text
- **Label**: Static text display
- **TextInput**: Editable text fields
- **TextBox**: Multi-line text display
- **Panel**: Solid color or texture backgrounds
- **FpsComponent**: FPS counter display
- **GameRender**: 3D game world rendering
- **MiniMap**: Top-down minimap view
- **MazeView**: Maze visualization
- **MazeEditor**: Interactive maze editor

## Usage Pattern

```rust
// In your event loop:
loop {
    event_loop.run_app(&mut app)?;
}

// In ApplicationHandler::window_event:
let ui_events = app.manager.process_input(&event, driver.logical_cursor());

for ui_event in ui_events {
    match ui_event {
        UIEvent::ButtonClicked(id) => {
            // Handle button click
        }
        // ...
    }
}

// Update components
app.manager.apply_updates(updates);

// Render
driver.render(&mut app.manager)?;
```

## Performance Considerations

- **Glyph Caching**: Characters are rasterized once and cached
- **Event-Driven Redraws**: Only redraw when visual state changes
- **Efficient Pixel Operations**: Direct frame buffer manipulation
- **Single-Threaded**: No synchronization overhead

## Modules

- **`components`**: UI component implementations
- **`context`**: Global UI context (fonts, textures, layout)
- **`driver`**: Window and rendering driver
- **`events`**: UI event types and component updates
- **`fonts`**: Font loading and text rendering
- **`geometry`**: Geometric primitives (Rect, IntRect)
- **`layout`**: Layout system with anchors and metrics
- **`manager`**: UI manager and layer orchestration

## License

Part of the multiplayer-fps project.
