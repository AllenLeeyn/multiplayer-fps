use crate::view::{View, ViewAction};
use std::collections::HashMap;

use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};

use fps_config::Config;
use fps_ui::{AppDriver, ComponentUpdate, WindowEvent, manager::UIManager};

use crate::{LOGICAL_HEIGHT, LOGICAL_WIDTH, PHYSICAL_HEIGHT, PHYSICAL_WIDTH};

pub struct App<'a> {
    pub window: Option<Window>,
    pub driver: Option<AppDriver<'a>>,
    pub manager: UIManager,

    pub views: HashMap<String, Box<dyn View>>,
    pub active_view: Option<String>,

    pub config: Config,
    pub config_path: String,
}

impl<'a> App<'a> {
    pub fn register_view(&mut self, view: Box<dyn View>) {
        let view_id = view.id().to_string();

        // Register layer immediately
        let layer = view.layer();
        self.manager
            .add_layer(layer)
            .expect("Failed to add view layer");

        self.views.insert(view_id, view);
    }

    pub fn activate_view(&mut self, view_id: &str) {
        // Hide current
        if let Some(current) = &self.active_view {
            self.manager.set_layer_visibility(current, false);
        }

        // Show new
        self.manager.set_layer_visibility(view_id, true);

        self.active_view = Some(view_id.to_string());
    }

    fn handle_view_action(&mut self, action: ViewAction, event_loop: &ActiveEventLoop) {
        match action {
            ViewAction::UpdateComponent(updates) => {
                self.manager.apply_updates(updates);
            }

            ViewAction::SwitchTo(view_id) => {
                self.activate_view(&view_id);
            }

            ViewAction::QuitApp => {
                event_loop.exit();
            }

            ViewAction::SaveUsername => {
                // Query the component for the username text
                let username = self
                    .manager
                    .find_component_by_id_mut("username_input")
                    .map(|c| c.get_text().to_string())
                    .unwrap_or_else(|| {
                        eprintln!("Warning: username_input component not found");
                        "ERROR".to_string()
                    });

                // Update config
                self.config.username = username.clone();

                // Update the label
                self.manager.apply_updates(vec![
                    ComponentUpdate::SetText(
                        "username_label".into(),
                        format!("Current user: {}", username),
                    ),
                    ComponentUpdate::SetText("connect_button".into(), "CONNECTED!".into()),
                    ComponentUpdate::SetText("username_input".into(), String::new()),
                ]);

                // Save to disk
                self.config.save(&self.config_path);
            }

            _ => {}
        }
    }
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        println!("--- Initializing Window and UI Renderer ---");

        let attrs = WindowAttributes::default()
            .with_title("fps_ui Crate Test Window")
            .with_inner_size(PhysicalSize::new(PHYSICAL_WIDTH, PHYSICAL_HEIGHT))
            .with_resizable(false);

        let window = event_loop.create_window(attrs).unwrap();

        // SAFETY: Window outlives the event loop
        let window_ref: &'a Window = unsafe { std::mem::transmute(&window) };

        match AppDriver::new(window_ref, LOGICAL_WIDTH, LOGICAL_HEIGHT) {
            Ok(driver) => {
                self.driver = Some(driver);
                self.window = Some(window);
            }
            Err(e) => {
                eprintln!("FATAL: Failed to initialize AppDriver: {}", e);
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let ui_events = {
            let driver = match self.driver.as_mut() {
                Some(d) => d,
                None => return,
            };

            self.manager.process_input(&event, driver.logical_cursor())
        };

        for ui_event in ui_events {
            let actions = match self.active_view.as_deref() {
                Some(id) => self
                    .views
                    .get_mut(id)
                    .map(|v| v.handle_ui_events(&ui_event))
                    .unwrap_or_default(),
                None => Vec::new(),
            };

            for action in actions {
                self.handle_view_action(action, event_loop);
            }
        }

        if let Some(driver) = self.driver.as_mut() {
            driver.handle_winit_event(&event);
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::RedrawRequested => {
                self.manager.update_components();
                if let Some(driver) = self.driver.as_mut() {
                    driver.render(&mut self.manager).unwrap();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}
