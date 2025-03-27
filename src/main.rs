use eframe::egui;
use chrono::{Local, NaiveDate, Duration};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::io::{self, Read, Write};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
struct DayEntry {
    note: String,
    actions: Vec<String>,
}

fn main() -> Result<(), eframe::Error> {
    // Create notes directory if it doesn't exist
    fs::create_dir_all("notes").expect("Failed to create notes directory");
    
    // Run migration for old .txt files
    migrate_txt_to_json();
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Reflect",
        options,
        Box::new(|cc| {
            // Configure the background color to white and text color to black
            let mut visuals = cc.egui_ctx.style().visuals.clone();
            visuals.window_fill = egui::Color32::WHITE;
            visuals.panel_fill = egui::Color32::WHITE;
            visuals.widgets.noninteractive.fg_stroke.color = egui::Color32::BLACK;
            visuals.widgets.inactive.fg_stroke.color = egui::Color32::BLACK;
            visuals.widgets.hovered.fg_stroke.color = egui::Color32::BLACK;
            visuals.widgets.active.fg_stroke.color = egui::Color32::BLACK;
            cc.egui_ctx.set_visuals(visuals);
            
            Box::new(ReflectApp::new())
        }),
    )
}

fn migrate_txt_to_json() {
    // Read the notes directory
    let notes_dir = Path::new("notes");
    if let Ok(entries) = fs::read_dir(notes_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(extension) = path.extension() {
                if extension == "txt" {
                    // Found a .txt file, let's migrate it
                    if let Some(stem) = path.file_stem() {
                        if let Some(date_str) = stem.to_str() {
                            // Try to parse the date from the filename
                            if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                                println!("Migrating {} to JSON format...", date_str);
                                
                                // Read the old txt content
                                if let Ok(note_content) = fs::read_to_string(&path) {
                                    // Create new DayEntry with the content
                                    let entry = DayEntry {
                                        note: note_content,
                                        actions: Vec::new(), // Start with empty actions for old entries
                                    };
                                    
                                    // Create the new JSON file
                                    let json_path = notes_dir.join(format!("{}.json", date_str));
                                    if let Ok(json_content) = serde_json::to_string_pretty(&entry) {
                                        if let Err(e) = fs::write(&json_path, json_content) {
                                            eprintln!("Failed to write JSON file for {}: {}", date_str, e);
                                            continue;
                                        }
                                    }
                                    
                                    // Delete the old .txt file
                                    if let Err(e) = fs::remove_file(&path) {
                                        eprintln!("Failed to delete old txt file for {}: {}", date_str, e);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Default)]
struct ReflectApp {
    entries: HashMap<NaiveDate, DayEntry>,
    current_date: NaiveDate,
    last_saved_entry: Option<DayEntry>,
    new_action: String,
}

impl ReflectApp {
    fn new() -> Self {
        let current_date = Local::now().date_naive();
        let mut app = Self {
            entries: HashMap::new(),
            current_date,
            last_saved_entry: None,
            new_action: String::new(),
        };
        app.load_entry(current_date);
        app
    }

    fn get_entry_path(date: NaiveDate) -> PathBuf {
        Path::new("notes").join(format!("{}.json", date.format("%Y-%m-%d")))
    }

    fn load_entry(&mut self, date: NaiveDate) {
        let path = Self::get_entry_path(date);
        let content = fs::read_to_string(&path).unwrap_or_default();
        let entry = if content.is_empty() {
            DayEntry::default()
        } else {
            serde_json::from_str(&content).unwrap_or_default()
        };
        self.entries.insert(date, entry.clone());
        self.last_saved_entry = Some(entry);
    }

    fn save_current_entry(&mut self) -> io::Result<()> {
        if let Some(entry) = self.entries.get(&self.current_date) {
            // Only save if the entry has changed
            if self.last_saved_entry.as_ref() != Some(entry) {
                let path = Self::get_entry_path(self.current_date);
                let content = serde_json::to_string_pretty(entry)?;
                fs::write(path, content)?;
                self.last_saved_entry = Some(entry.clone());
            }
        }
        Ok(())
    }

    fn render_app_header(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Add some padding from the left
            ui.add_space(10.0);
            
            // Create a circle frame for the icon
            let circle_size = 24.0;
            let circle_frame = egui::Frame::none()
                .fill(egui::Color32::from_gray(200))
                .rounding(circle_size / 2.0);
            
            circle_frame.show(ui, |ui| {
                ui.allocate_space(egui::vec2(circle_size, circle_size));
            });
            
            ui.add_space(8.0); // Space between circle and text
            
            // Add the "Reflect" text
            ui.heading("Reflect");
        });
        ui.add_space(10.0); // Space after the header
    }

    fn render_header(&self, ui: &mut egui::Ui) {
        let header_frame = egui::Frame::none()
            .fill(egui::Color32::from_gray(160))
            .inner_margin(egui::style::Margin::symmetric(20.0, 40.0))
            .outer_margin(egui::style::Margin::symmetric(0.0, 0.0));
            
        header_frame.show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading(
                    egui::RichText::new("Add Your Daily Note")
                        .color(egui::Color32::WHITE)
                        .size(24.0)
                );
                ui.add_space(5.0);
                ui.label(
                    egui::RichText::new("Reflect on your day and jot down your thoughts")
                        .color(egui::Color32::WHITE)
                );
            });
        });
        ui.add_space(20.0);
    }

    fn render_date_navigation(&mut self, ui: &mut egui::Ui) {
        // Save current entry before changing date
        if let Err(e) = self.save_current_entry() {
            eprintln!("Failed to save entry: {}", e);
        }

        ui.horizontal(|ui| {
            ui.add_space(ui.available_width() * 0.2);
            
            if ui.add(egui::Button::new(
                egui::RichText::new("⬅").size(24.0)
            ).frame(false)).clicked() {
                self.current_date = self.current_date - Duration::days(1);
                self.load_entry(self.current_date);
            }
            
            ui.add_space(20.0);
            
            ui.label(
                egui::RichText::new(self.current_date.format("%A, %B %d, %Y").to_string())
                    .size(18.0)
                    .strong()
            );
            
            ui.add_space(20.0);
            
            if ui.add(egui::Button::new(
                egui::RichText::new("➡").size(24.0)
            ).frame(false)).clicked() {
                self.current_date = self.current_date + Duration::days(1);
                self.load_entry(self.current_date);
            }
            
            ui.add_space(ui.available_width() * 0.2);
        });
    }

    fn render_text_box(&mut self, ui: &mut egui::Ui) {
        let text_frame = egui::Frame::none()
            .fill(egui::Color32::from_gray(245))
            .rounding(8.0)
            .inner_margin(10.0)
            .outer_margin(10.0);

        text_frame.show(ui, |ui| {
            ui.add_space(5.0);
            ui.heading("Write about your day");
            ui.add_space(10.0);
            
            let entry = self.entries.entry(self.current_date).or_default();
            
            let text_edit = egui::TextEdit::multiline(&mut entry.note)
                .desired_width(f32::INFINITY)
                .desired_rows(4)
                .frame(false);
            
            if ui.add(text_edit).changed() {
                // Entry has been modified, will be saved on date change or app close
            }
            
            ui.add_space(5.0);
        });
    }

    fn render_actions(&mut self, ui: &mut egui::Ui) {
        let actions_frame = egui::Frame::none()
            .fill(egui::Color32::from_gray(245))
            .rounding(8.0)
            .inner_margin(10.0)
            .outer_margin(10.0);

        actions_frame.show(ui, |ui| {
            ui.add_space(5.0);
            ui.heading("Actions");
            ui.add_space(10.0);

            let entry = self.entries.entry(self.current_date).or_default();
            
            // Add new action input
            ui.horizontal(|ui| {
                let text_edit = egui::TextEdit::singleline(&mut self.new_action)
                    .desired_width(ui.available_width() - 60.0)
                    .hint_text("Add a new action item...")
                    .frame(false);
                
                ui.add(text_edit);
                
                if ui.add(egui::Button::new(
                    egui::RichText::new("Add")
                        .color(egui::Color32::BLACK)
                ).fill(egui::Color32::from_gray(230)))
                .clicked() && !self.new_action.trim().is_empty() {
                    entry.actions.push(self.new_action.trim().to_string());
                    self.new_action.clear();
                }
            });
            
            ui.add_space(10.0);
            
            // Display existing actions
            let mut action_to_remove = None;
            for (idx, action) in entry.actions.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label("•");
                    ui.add_space(5.0);
                    ui.label(action);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(egui::Button::new(
                            egui::RichText::new("✖")
                                .color(egui::Color32::BLACK)
                        ).frame(false))
                        .clicked() {
                            action_to_remove = Some(idx);
                        }
                    });
                });
            }
            
            // Remove action if delete button was clicked
            if let Some(idx) = action_to_remove {
                entry.actions.remove(idx);
            }
            
            ui.add_space(5.0);
        });
    }
}

impl eframe::App for ReflectApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_app_header(ui);
            self.render_header(ui);
            
            let content_frame = egui::Frame::none()
                .inner_margin(egui::style::Margin::symmetric(0.0, 0.0));
            
            content_frame.show(ui, |ui| {
                self.render_date_navigation(ui);
                ui.add_space(16.0);
                self.render_text_box(ui);
                ui.add_space(16.0);
                self.render_actions(ui);
            });
        });

        // Save entry periodically
        if let Err(e) = self.save_current_entry() {
            eprintln!("Failed to save entry: {}", e);
        }
    }
} 