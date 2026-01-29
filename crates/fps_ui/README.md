# `fps_ui` - Single-Threaded UI Toolkit

A high-performance, single-threaded UI toolkit built on top of `winit` and `pixels`. Designed for game UIs with a caller-driven architecture where the application maintains full control over game state and event execution.

## Features

- **Single-Threaded**: All UI logic runs on one thread for simplicity and performance
- **Caller-Driven**: Application maintains full control over game state
- **Layered Components**: Organize UI with Z-ordering and visibility control
- **Flexible Layout**: Pixel-based and percentage-based positioning with anchor points
- **Efficient Rendering**: Glyph caching and optimized pixel operations; supports app-driven redraw control
- **Type-Safe**: Strong typing for events, updates, and component interactions

## Quick Start

```rust
use fps_ui::{Color, LayoutMetrics, Rect, UIMainContext, UIManager};
use fps_ui::components::Button;
use fps_ui::manager::Layer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create UI context (font + logical canvas size)
    let mut context = UIMainContext::new("assets/fonts/Symtext.ttf", 800.0, 600.0)?;

    // Load textures (optional; components may use solid colors instead).
    // Note: this crate only ships fonts under `assets/fonts/`. In the main game, textures
    // are typically loaded from the project-level assets directory.
    // context.load_texture("some_texture_id", "path/to/texture.png")?;

    // Create UI manager
    let mut manager = UIManager::new(context);

    // Create a button component
    let button = Button::new(
        "my_button",
        "Click Me",
        Rect::new(100.0, 100.0, 200.0, 50.0),
        LayoutMetrics::default(),
        Color::WHITE, // text color
        Color::BLUE,  // background color
        Color::CYAN,  // hover color
    );

    // Add to a layer (Layer is defined in fps_ui::manager and is not re-exported)
    let layer = Layer {
        id: "main".to_string(),
        z_index: 0,
        is_visible: true,
        is_modal: false,
        components: vec![Box::new(button)],
    };

    manager.add_layer(layer).expect("layer id already exists");
    Ok(())
}
```

## Architecture

### Design Philosophy

The crate extends low-level functionalities into a unified, high-level UI system:

- **`winit`** → Event handling: Translates raw OS events into structured `UIEvent` messages
- **`pixels`** → Drawing: Provides abstract rendering functions atop raw pixel buffers
- **`glam`** → Utility math types: Used for small vector/color representations (e.g. `IVec2`, `U8Vec4`)

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
- **GameRender**: 3D raycasted game world (see [GameRender](#gamerender) below)
- **MiniMap**: Top-down minimap view
- **MazeView**: Maze visualization
- **MazeEditor**: Interactive maze editor

### GameRender

`GameRender` draws the first-person 3D view: raycasted walls, floor, ceiling, other players (as circle sprites), and bullets. Full details are in the [module docs](src/components/game_render.rs).

**Setup:** Create with `GameRender::new(id, bounds)`. The app feeds state via `ComponentUpdate`:

- `SetMaze(id, maze)` — maze grid to raycast against
- `SetGameRender(id, players_map, bullets_list)` — other players and bullets (e.g. from network snapshot)
- `SetGameRenderCamera(id, (player_id, x, y, angle, is_invincible))` — local camera / view player

**Input:** This component **only handles the Escape key**. It emits `UIEvent::ButtonClicked(component_id)` so the app can show pause/exit. All other input (WASD, mouse, shoot) is handled in the **app loop**; the app updates game state and pushes it back with the updates above.

**Draw world (raycasting math + logic):** `draw_world()` implements a classic “one ray per screen column” raycaster.

- **Projection plane distance**: With horizontal FOV \(FOV\) and logical screen width \(W\), we place the projection plane at:

  \[
  dist\_to\_plane = (W/2) / \tan(FOV/2)
  \]

  This is what turns a world-space distance into an on-screen height.

- **Ray direction per column**: For each column \(x \in [0, W)\), we map it to camera-space \(camera\_x \in [-1, 1]\):

  \[
  camera\_x = 2x/W - 1
  \]

  With camera yaw `angle`, we compute a world-space ray direction \((dir_x, dir_y)\). In code this is the forward vector \((\cos a, \sin a)\) plus a sideways component scaled by \(\tan(FOV/2)\).

- **Grid stepping (DDA)**: `cast_ray_dda(cam_x, cam_y, dir_x, dir_y, maze)` walks the maze grid cell-by-cell:
  - Convert world position into cell coordinates: `map_x = floor(px / BOX_SIZE)`, `map_y = floor(py / BOX_SIZE)`.
  - Precompute how far we must travel along the ray to cross one grid line in X/Y:
    - `delta_dist_x = abs(1/dir_x)`, `delta_dist_y = abs(1/dir_y)`
  - Maintain `side_dist_x` / `side_dist_y` (distance to the *next* vertical/horizontal grid boundary) and step toward whichever is smaller each iteration.
  - Stop when `maze.get_cell_safe(map_x, map_y)` reports a wall.
  - Return: hit distance, which side was hit (X-side vs Y-side), and hit position \((hit_x, hit_y)\).

- **Fish-eye correction and z-buffer**: Raw DDA distance is corrected so wall height uses distance along the camera forward direction (reduces fish-eye):
  - `corrected_dist = dist * dot(forward, dir)` where `forward = (cos a, sin a)`.
  - Store `corrected_dist` into the per-column `z_buffer[x]` for sprite/bullet occlusion later.

- **Wall projection**: Wall height in pixels is:

  \[
  wall\_height = (BOX\_SIZE \cdot dist\_to\_plane) / corrected\_dist
  \]

  We draw a vertical strip from `draw_start = H/2 - wall_height/2` to `draw_end = H/2 + wall_height/2`.

- **Wall texture mapping**:
  - Horizontal coordinate \(u\) comes from where the ray hit the wall within the cell: `wall_u = (hit_coord % BOX_SIZE) / BOX_SIZE` (choose X or Y hit coordinate depending on which side was hit).
  - Vertical coordinate \(v\) is the normalized position within the strip: `wall_v = (y - draw_start) / wall_height`.
  - The color is shaded by hit side (Y-side darker) and distance fade.

- **Ceiling + floor**: For pixels above the wall strip (ceiling) and below it (floor), we compute a distance to the plane for that row:
  - Let \(p\) be the vertical offset from screen center (in pixels). The code uses:
    - `p = H/2 - y` (ceiling) or `p = y - H/2` (floor)
    - `row_dist = (BOX_SIZE*0.5*dist_to_plane) / p`
  - Sample the ceiling/floor texture at the world position `cam + row_dist * dir`, then tile by `BOX_SIZE` using fractional parts (`rem_euclid(1.0)`).

**Players:** Drawn as circle sprites with **cylinder mapping**: the texture wraps horizontally by angle; a disc mask and simple shading give a rounded look. Size scales with depth; drawing is depth-tested against the raycast z-buffer. A soft shadow is drawn under each player.

**Bullets:** Drawn as small yellow perspective-scaled quads (no texture). Projected and depth-tested like players; size scales with distance.

## Usage Pattern

```rust
// In ApplicationHandler::window_event:
// 1) Let AppDriver update its scale/cursor state first
driver.handle_winit_event(&event);

// 2) Route the event into the UI using the *logical* cursor position
let ui_events = app.manager.process_input(&event, driver.logical_cursor());

for ui_event in ui_events {
    match ui_event {
        UIEvent::ButtonClicked(id) => {
            // Handle button click
        }
        // ...
    }
}

// Update components (returns whether any update requested a redraw)
let needs_redraw = app.manager.apply_updates(updates);

// Render (app decides when to request redraw)
if needs_redraw {
    driver.window().request_redraw();
}
```

## Performance Considerations

- **Glyph Caching**: Characters are rasterized once and cached
- **App-Driven Redraws**: You can request redraw only when visual state changes (e.g. when `apply_updates()` returns `true`)
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

## Dependencies

- **winit**: Windowing + input events (the app owns the event loop)
- **pixels**: Presents a logical RGBA framebuffer to a winit window
- **ab_glyph**: Font parsing/rasterization building block for `FontManager`
- **glam**: Small math/geometry utilities used throughout
- **image**: Texture loading in `UIMainContext::load_texture()`

## License

Part of the multiplayer-fps project.
