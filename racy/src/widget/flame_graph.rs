use egui::{self, Vec2};
use std::{collections::HashMap};

use crate::event::{Events, Thread};

pub const TIMELINE_MARK_INTERVAL: u64 = 1000000000;

pub struct FlameGraphWidget<'a> {
    events: &'a Events,
    folded_processes: &'a mut HashMap<usize, bool>,
}

impl<'a> FlameGraphWidget<'a> {
    pub fn new(events: &'a Events, folded_processes: &'a mut HashMap<usize, bool>) -> Self {
        Self {
            events,
            folded_processes,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) -> Option<egui::scroll_area::ScrollAreaOutput<()>> {
        if self.events.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("No events loaded. Use File → Open to load a profile.");
            });
            return None;
        }

        let (min_time, max_time) = self.get_global_time_range();

        // Create horizontal scroll area for main content
        let mut scroll_output = egui::ScrollArea::both()
            .id_salt("main_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // Set a wide virtual width for horizontal scrolling
                let virtual_width = ui.available_width() * 3.0;

                ui.set_min_width(virtual_width);

                // Draw vertical grid lines
                self.draw_grid_lines(ui, min_time, max_time, virtual_width);
                ui.add_space(5.0);

                let grouped_events = &self.events.threads;

                let mut process_ids: Vec<_> = grouped_events.keys().copied().collect();

                process_ids.sort();
                for process_id in process_ids {
                    
                    if let Some(events) = grouped_events.get(&process_id) {
                        let is_folded = self
                            .folded_processes
                            .get(&(process_id as usize))
                            .copied()
                            .unwrap_or(false);

                        // Use frame without border
                        egui::Frame::new()
                            .inner_margin(egui::Margin::same(8))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());

                                // Header with fold/unfold button
                                ui.horizontal(|ui| {
                                    let button_text = if is_folded { "▶" } else { "▼" };
                                    if ui.small_button(button_text).clicked() {
                                        self.folded_processes.insert(process_id as usize, !is_folded);
                                    }

                                    ui.label(
                                        egui::RichText::new(format!("Process {}", process_id))
                                            .size(14.0)
                                            .strong(),
                                    );

                                    let span_count = events.spans.len();
                                    ui.label(format!("({} spans)", span_count));
                                });

                                // Show flamegraph if not folded
                                if !is_folded {
                                    ui.add_space(5.0);
                                    self.draw_flamegraph(ui, events, min_time, max_time);
                                }
                            });

                        ui.add_space(5.0);
                    }
                }
                ui.add_space(60.0);
            });
        scroll_output.state.offset = Vec2::new(scroll_output.state.offset.x, 0.0);
        Some(scroll_output)
    }

    pub fn draw_timeline_axis(&self, ui: &mut egui::Ui) {
        let (min_time, max_time) = self.get_global_time_range();
        self.draw_time_axis(ui, min_time, max_time);
    }

    pub fn get_global_time_range(&self) -> (u64, u64) {
        let padding = (self.events.total_duration as f32 * 0.2) as u64;
        (0, self.events.total_duration + padding)
    }

    fn draw_grid_lines(&self, ui: &mut egui::Ui, min_time: u64, max_time: u64, virtual_width: f32) {
        let content_rect = ui.available_rect_before_wrap();
        let painter = ui.painter();
        let time_range = (max_time - min_time) as f32;

        // Draw major grid lines
        let first_mark = ((min_time / TIMELINE_MARK_INTERVAL) + 1) * TIMELINE_MARK_INTERVAL;
        let mut current_mark = first_mark;

        while current_mark <= max_time {
            let x_pos = ((current_mark - min_time) as f32 / time_range) * virtual_width;
            let x = content_rect.left() + x_pos;

            painter.line_segment(
                [
                    egui::pos2(x, content_rect.top()),
                    egui::pos2(x, content_rect.bottom()),
                ],
                egui::Stroke::new(0.5, egui::Color32::from_gray(50)),
            );

            // Draw minor grid lines
            if TIMELINE_MARK_INTERVAL > 10 {
                let minor_interval = 100000000;
                for i in 1..(TIMELINE_MARK_INTERVAL / minor_interval) {
                    let minor_time = current_mark - TIMELINE_MARK_INTERVAL + (i * minor_interval);
                    if minor_time >= min_time && minor_time <= max_time {
                        let minor_x_pos =
                            ((minor_time - min_time) as f32 / time_range) * virtual_width;
                        let minor_x = content_rect.left() + minor_x_pos;

                        painter.line_segment(
                            [
                                egui::pos2(minor_x, content_rect.top()),
                                egui::pos2(minor_x, content_rect.bottom()),
                            ],
                            egui::Stroke::new(0.3, egui::Color32::from_gray(40)),
                        );
                    }
                }
            }

            current_mark += TIMELINE_MARK_INTERVAL;
        }
    }

    fn draw_time_axis(&self, ui: &mut egui::Ui, min_time: u64, max_time: u64) {
        let available_width = ui.available_width();
        let axis_height = 40.0;

        let (response, painter) = ui.allocate_painter(
            egui::Vec2::new(available_width, axis_height),
            egui::Sense::hover(),
        );

        let rect = response.rect;
        let time_range = (max_time - min_time) as f32;

        // Draw axis line
        painter.line_segment(
            [
                egui::pos2(rect.left(), rect.top() + 10.0),
                egui::pos2(rect.right(), rect.top() + 10.0),
            ],
            egui::Stroke::new(1.0, egui::Color32::GRAY),
        );

        // Calculate the first mark position
        let first_mark = ((min_time / TIMELINE_MARK_INTERVAL) + 1) * TIMELINE_MARK_INTERVAL;

        // Draw marks at regular intervals
        let mut current_mark = first_mark;
        while current_mark <= max_time {
            let x_pos = ((current_mark - min_time) as f32 / time_range) * rect.width();
            let x = rect.left() + x_pos;

            // Draw major tick
            painter.line_segment(
                [
                    egui::pos2(x, rect.top() + 5.0),
                    egui::pos2(x, rect.top() + 15.0),
                ],
                egui::Stroke::new(2.0, egui::Color32::GRAY),
            );

            // Draw label
            painter.text(
                egui::pos2(x, rect.top() + 25.0),
                egui::Align2::CENTER_CENTER,
                format!("{}", current_mark),
                egui::FontId::proportional(12.0),
                egui::Color32::GRAY,
            );

            // Draw minor ticks
            if TIMELINE_MARK_INTERVAL > 10 {
                let minor_interval = 100000000;
                for i in 1..(TIMELINE_MARK_INTERVAL / minor_interval) {
                    let minor_time = current_mark - TIMELINE_MARK_INTERVAL + (i * minor_interval);
                    if minor_time >= min_time {
                        let minor_x_pos =
                            ((minor_time - min_time) as f32 / time_range) * rect.width();
                        let minor_x = rect.left() + minor_x_pos;

                        painter.line_segment(
                            [
                                egui::pos2(minor_x, rect.top() + 8.0),
                                egui::pos2(minor_x, rect.top() + 12.0),
                            ],
                            egui::Stroke::new(1.0, egui::Color32::from_gray(100)),
                        );
                    }
                }
            }

            current_mark += TIMELINE_MARK_INTERVAL;
        }

        // Draw start and end markers
        if min_time % TIMELINE_MARK_INTERVAL != 0 {
            painter.text(
                egui::pos2(rect.left(), rect.top() + 25.0),
                egui::Align2::LEFT_CENTER,
                format!("{}", min_time),
                egui::FontId::proportional(10.0),
                egui::Color32::from_gray(150),
            );
        }

        if max_time % TIMELINE_MARK_INTERVAL != 0 {
            painter.text(
                egui::pos2(rect.right(), rect.top() + 25.0),
                egui::Align2::RIGHT_CENTER,
                format!("{}", max_time),
                egui::FontId::proportional(10.0),
                egui::Color32::from_gray(150),
            );
        }
    }

    fn draw_flamegraph(&self, ui: &mut egui::Ui, spans: &Thread, min_time: u64, max_time: u64) {

        if spans.is_empty() {
            ui.label("No complete event spans to display");
            return;
        }

        let time_range = (max_time - min_time) as f32;
        let available_width = ui.available_width();

        // Calculate required height based on maximum depth
        let max_depth = spans.spans.iter().map(|s| s.depth).max().unwrap_or(0);
        let block_height = 25.0;
        let block_spacing = 2.0;
        let graph_height = ((max_depth + 1) as f32) * (block_height + block_spacing) + 20.0;

        // Draw the flamegraph
        let (response, painter) = ui.allocate_painter(
            egui::Vec2::new(available_width, graph_height),
            egui::Sense::hover(),
        );

        let rect = response.rect;

        // Draw spans
        for span in &spans.spans {
            let x_start =
                rect.left() + ((span.timestamp - min_time) as f32 / time_range) * rect.width();
            let x_end = rect.left()
                + ((span.timestamp + span.duration - min_time) as f32 / time_range) * rect.width();
            let width = x_end - x_start;

            let y = rect.top() + (span.depth as f32) * (block_height + block_spacing);

            let block_rect = egui::Rect::from_min_size(
                egui::pos2(x_start, y),
                egui::Vec2::new(width, block_height),
            );

            // Generate color based on depth and message hash
            let hash = span
                .name
                .bytes()
                .fold(0u32, |acc, b| acc.wrapping_add(b as u32));
            let color = match (span.depth + hash as u64) % 6 {
                0 => egui::Color32::from_rgb(66, 165, 245),  // Blue
                1 => egui::Color32::from_rgb(102, 187, 106), // Green
                2 => egui::Color32::from_rgb(255, 167, 38),  // Orange
                3 => egui::Color32::from_rgb(239, 83, 80),   // Red
                4 => egui::Color32::from_rgb(156, 39, 176),  // Purple
                _ => egui::Color32::from_rgb(0, 188, 212),   // Cyan
            };

            // Draw block
            painter.rect_filled(block_rect, egui::CornerRadius::same(2), color);

            // Draw border
            painter.rect_stroke(
                block_rect,
                egui::CornerRadius::same(2),
                egui::Stroke::new(1.0, egui::Color32::from_gray(60)),
                egui::StrokeKind::Inside,
            );

            // Draw text if there's enough space
            if width > 30.0 {
                let text = if span.name.len() > 20 && width < 150.0 {
                    format!("{}...", &span.name[..17])
                } else {
                    span.name.clone()
                };

                painter.text(
                    block_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    text,
                    egui::FontId::proportional(11.0),
                    egui::Color32::WHITE,
                );
            }
        }
    }
}
