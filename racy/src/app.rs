use std::collections::HashMap;
use std::path::PathBuf;

use crate::{
    data::load_from_file,
    event::Events,
    widget::{
        flame_graph::FlameGraphWidget,
        menu::{open_file_dialog, save_file_dialog, MenuAction, MenuBar},
    },
};

pub struct FlameGraphApp {
    events: Events,
    folded_processes: HashMap<usize, bool>,
    current_file: Option<PathBuf>,
    show_error_dialog: Option<String>,
}

impl Default for FlameGraphApp {
    fn default() -> Self {
        Self {
            events: load_from_file(),
            folded_processes: HashMap::new(),
            current_file: None,
            show_error_dialog: None,
        }
    }
}

impl FlameGraphApp {
    fn save_to_file(&mut self, path: PathBuf) -> Result<(), String> {
        // Serialize events to JSON
        let json = serde_json::to_string_pretty(&self.events)
            .map_err(|e| format!("Failed to serialize events: {}", e))?;

        // Write to file
        std::fs::write(&path, json).map_err(|e| format!("Failed to write file: {}", e))?;

        self.current_file = Some(path);
        Ok(())
    }

    fn load_from_file(&mut self, path: PathBuf) -> Result<(), String> {
        // Read file
        let contents =
            std::fs::read_to_string(&path).map_err(|e| format!("Failed to read file: {}", e))?;

        // Deserialize events
        let events: Events =
            serde_json::from_str(&contents).map_err(|e| format!("Failed to parse JSON: {}", e))?;

        self.events = events;
        self.folded_processes.clear(); // Reset fold states
        self.current_file = Some(path);
        Ok(())
    }

    fn clear_data(&mut self) {
        self.events.clear();
        self.folded_processes.clear();
        self.current_file = None;
    }

    fn handle_menu_action(&mut self, action: MenuAction, ctx: &egui::Context) {
        match action {
            MenuAction::NewFile => {
                self.clear_data();
            }
            MenuAction::OpenFile => {
                if let Some(path) = open_file_dialog() {
                    if let Err(e) = self.load_from_file(path) {
                        self.show_error_dialog = Some(e);
                    }
                }
            }
            MenuAction::SaveFile => {
                if let Some(path) = &self.current_file.clone() {
                    if let Err(e) = self.save_to_file(path.clone()) {
                        self.show_error_dialog = Some(e);
                    }
                } else {
                    // If no current file, trigger Save As
                    if let Some(path) = save_file_dialog() {
                        if let Err(e) = self.save_to_file(path) {
                            self.show_error_dialog = Some(e);
                        }
                    }
                }
            }
            MenuAction::SaveFileAs => {
                if let Some(path) = save_file_dialog() {
                    if let Err(e) = self.save_to_file(path) {
                        self.show_error_dialog = Some(e);
                    }
                }
            }
            MenuAction::Exit => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            MenuAction::ExpandAll => {
                self.folded_processes.clear();
            }
            MenuAction::CollapseAll => {
                let process_ids: Vec<_> = self
                    .events
                    .thread_ids()
                    .collect();

                for id in process_ids {
                    self.folded_processes.insert(*id as usize, true);
                }
            }
            MenuAction::None => {}
        }
    }
}

impl eframe::App for FlameGraphApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle error dialog
        if let Some(error_msg) = &self.show_error_dialog.clone() {
            egui::Window::new("Error")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(error_msg);
                    if ui.button("OK").clicked() {
                        self.show_error_dialog = None;
                    }
                });
        }

        // Show menu bar and handle actions
        let menu_bar = MenuBar::new(&self.current_file);
        let action = menu_bar.show(ctx);
        self.handle_menu_action(action, ctx);

        // Main content
        let mut scroll_output = None;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Process Event Flamegraphs");

            // Create and show flamegraph widget
            let flamegraph = FlameGraphWidget::new(&self.events, &mut self.folded_processes);
            scroll_output = flamegraph.show(ui);
        });

        // Timeline panel (only show if we have events and scroll output)
        if !self.events.is_empty() {
            if let Some(output) = scroll_output {
                egui::TopBottomPanel::bottom("timeline_panel").show(ctx, |ui| {
                    ui.add_space(5.0);

                    egui::ScrollArea::horizontal()
                        .id_salt("timeline_scroll")
                        .scroll_offset(output.state.offset)
                        .show(ui, |ui| {
                            let virtual_width = ui.available_width() * 3.0;
                            ui.set_min_width(virtual_width);

                            let flamegraph =
                                FlameGraphWidget::new(&self.events, &mut self.folded_processes);
                            flamegraph.draw_timeline_axis(ui);
                        });

                    ui.add_space(10.0);
                });
            }
        }
    }
}
