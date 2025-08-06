use egui;
use std::path::PathBuf;

pub enum MenuAction {
    None,
    NewFile,
    OpenFile,
    SaveFile,
    SaveFileAs,
    Exit,
    ExpandAll,
    CollapseAll,
}

pub struct MenuBar<'a> {
    current_file: &'a Option<PathBuf>,
}

impl<'a> MenuBar<'a> {
    pub fn new(current_file: &'a Option<PathBuf>) -> Self {
        Self { current_file }
    }

    pub fn show(&self, ctx: &egui::Context) -> MenuAction {
        let mut action = MenuAction::None;

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                // File menu
                ui.menu_button("File", |ui| {
                    if ui.button("New").clicked() {
                        action = MenuAction::NewFile;
                        ui.close();
                    }

                    ui.separator();

                    if ui.button("Open...").clicked() {
                        action = MenuAction::OpenFile;
                        ui.close();
                    }

                    if ui.button("Save").clicked() {
                        action = MenuAction::SaveFile;
                        ui.close();
                    }

                    if ui.button("Save As...").clicked() {
                        action = MenuAction::SaveFileAs;
                        ui.close();
                    }

                    ui.separator();

                    if ui.button("Exit").clicked() {
                        action = MenuAction::Exit;
                        ui.close();
                    }
                });

                // View menu
                ui.menu_button("View", |ui| {
                    if ui.button("Expand All").clicked() {
                        action = MenuAction::ExpandAll;
                        ui.close();
                    }

                    if ui.button("Collapse All").clicked() {
                        action = MenuAction::CollapseAll;
                        ui.close();
                    }
                });

                // Show current file in menu bar
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(path) = self.current_file {
                        if let Some(filename) = path.file_name() {
                            ui.label(format!("File: {}", filename.to_string_lossy()));
                        }
                    }
                });
            });
        });

        action
    }
}

// Helper function to show file dialogs
pub fn open_file_dialog() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("JSON files", &["json"])
        .add_filter("All files", &["*"])
        .pick_file()
}

pub fn save_file_dialog() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("JSON files", &["json"])
        .set_file_name("flamegraph.json")
        .save_file()
}
