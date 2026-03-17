use std::time::Duration;

use crate::build::{self, BuildPhase};
use crate::config::backends::{auto_select, available_backends, make_backend};
use crate::config::{AppConfig, Backend, BackendName};
use crate::hardware::{self, HardwareInfo};
use crate::models::{self, ModelEntry};
use crate::tui::{self, InputAction};

/// The screens the TUI can display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Welcome,
    Hardware,
    Backend,
    Models,
    Install,
    Summary,
}

/// Full application state, driving the TUI render loop.
pub struct App {
    pub screen: Screen,
    pub config: AppConfig,
    pub hw: Option<HardwareInfo>,
    pub backends: Vec<BackendName>,
    pub backend_idx: usize,
    pub selected_backend: Option<Backend>,
    pub recommended_backend: Option<BackendName>,
    pub models: Vec<ModelEntry>,
    pub model_idx: usize,
    pub selected_model: Option<ModelEntry>,
    pub search_query: String,
    pub is_searching: bool,
    pub build_log: Vec<String>,
    pub build_phase: BuildPhase,
    pub build_progress: f64,
    pub build_done: bool,
    pub build_ok: bool,
    pub should_quit: bool,
}

impl App {
    pub fn new(config: AppConfig) -> Self {
        Self {
            screen: Screen::Welcome,
            config,
            hw: None,
            backends: Vec::new(),
            backend_idx: 0,
            selected_backend: None,
            recommended_backend: None,
            models: Vec::new(),
            model_idx: 0,
            selected_model: None,
            search_query: String::new(),
            is_searching: false,
            build_log: Vec::new(),
            build_phase: BuildPhase::Clone,
            build_progress: 0.0,
            build_done: false,
            build_ok: false,
            should_quit: false,
        }
    }

    /// Run the TUI main loop.
    pub fn run(&mut self) -> anyhow::Result<()> {
        let mut terminal = tui::init_terminal()?;

        while !self.should_quit {
            terminal.draw(|frame| self.render(frame))?;
            let action = tui::poll_input(Duration::from_millis(50), self.is_searching);
            self.handle_input(action);
        }

        tui::restore_terminal(&mut terminal)?;
        Ok(())
    }

    fn render(&self, frame: &mut ratatui::Frame) {
        let area = frame.area();
        match self.screen {
            Screen::Welcome => {
                tui::views::welcome::render(frame, area);
            }
            Screen::Hardware => {
                if let Some(hw) = &self.hw {
                    tui::views::hardware::render(frame, area, hw);
                }
            }
            Screen::Backend => {
                tui::views::backend::render(
                    frame,
                    area,
                    &self.backends,
                    self.backend_idx,
                    self.recommended_backend,
                );
            }
            Screen::Models => {
                tui::views::models::render(
                    frame,
                    area,
                    &self.models,
                    self.model_idx,
                    &self.search_query,
                    self.is_searching,
                );
            }
            Screen::Install => {
                tui::views::install::render(
                    frame,
                    area,
                    self.build_phase,
                    &self.build_log,
                    self.build_progress,
                    self.build_done,
                );
            }
            Screen::Summary => {
                if let (Some(hw), Some(backend)) = (&self.hw, &self.selected_backend) {
                    let prefix = self.config.resolved_prefix();
                    let model_name = self.selected_model.as_ref().map(|m| m.name.as_str());
                    tui::views::summary::render(
                        frame,
                        area,
                        hw,
                        backend,
                        &prefix,
                        model_name,
                        self.build_ok,
                    );
                }
            }
        }
    }

    fn handle_input(&mut self, action: InputAction) {
        if action == InputAction::Quit {
            self.should_quit = true;
            return;
        }

        match self.screen {
            Screen::Welcome => self.handle_welcome(action),
            Screen::Hardware => self.handle_hardware(action),
            Screen::Backend => self.handle_backend(action),
            Screen::Models => self.handle_models(action),
            Screen::Install => self.handle_install(action),
            Screen::Summary => self.handle_summary(action),
        }
    }

    fn handle_welcome(&mut self, action: InputAction) {
        if action == InputAction::Select {
            // Detect hardware and move forward
            match hardware::detect::detect_all() {
                Ok(hw) => {
                    let rec = auto_select(&hw).ok().map(|b| b.name);
                    self.backends = available_backends(&hw);
                    self.recommended_backend = rec;
                    // Pre-select recommended backend
                    if let Some(rec_name) = rec {
                        if let Some(idx) = self.backends.iter().position(|b| *b == rec_name) {
                            self.backend_idx = idx;
                        }
                    }
                    self.hw = Some(hw);
                    self.screen = Screen::Hardware;
                }
                Err(e) => {
                    self.build_log
                        .push(format!("Hardware detection failed: {}", e));
                }
            }
        }
    }

    fn handle_hardware(&mut self, action: InputAction) {
        match action {
            InputAction::Select => {
                self.screen = Screen::Backend;
            }
            InputAction::Back => {
                self.screen = Screen::Welcome;
            }
            _ => {}
        }
    }

    fn handle_backend(&mut self, action: InputAction) {
        match action {
            InputAction::Up => {
                if self.backend_idx > 0 {
                    self.backend_idx -= 1;
                }
            }
            InputAction::Down => {
                if self.backend_idx + 1 < self.backends.len() {
                    self.backend_idx += 1;
                }
            }
            InputAction::Select => {
                if self.backend_idx < self.backends.len() {
                    let name = self.backends[self.backend_idx];
                    let cuda_arch = self.hw.as_ref().and_then(|hw| {
                        crate::config::backends::map_compute_capability(&hw.gpu.compute_capability)
                    });
                    let backend = make_backend(name, cuda_arch);
                    self.selected_backend = Some(backend);

                    // Populate model recommendations
                    if let Some(hw) = &self.hw {
                        self.models =
                            models::registry::recommend(hw.total_vram_mb(), hw.total_memory_mb());
                    }
                    self.model_idx = 0;
                    self.screen = Screen::Models;
                }
            }
            InputAction::Back => {
                self.screen = Screen::Hardware;
            }
            _ => {}
        }
    }

    fn handle_models(&mut self, action: InputAction) {
        if self.is_searching {
            match action {
                InputAction::Char(c) => {
                    self.search_query.push(c);
                }
                InputAction::Back => {
                    if self.search_query.is_empty() {
                        self.is_searching = false;
                    } else {
                        self.search_query.pop();
                    }
                }
                InputAction::Select => {
                    // Execute search
                    self.is_searching = false;
                    if !self.search_query.is_empty() {
                        // Create a tokio runtime to run the async search
                        let rt = tokio::runtime::Runtime::new().unwrap();
                        match rt.block_on(models::huggingface::search_models(&self.search_query)) {
                            Ok(results) => {
                                let search_models: Vec<ModelEntry> = results
                                    .into_iter()
                                    .map(|r| ModelEntry {
                                        name: r.id.clone(),
                                        repo: r.id,
                                        quantization: "Q4_K_M".into(),
                                        size_label: format!("{} downloads", r.downloads),
                                        min_vram_mb: 0,
                                        min_ram_mb: 0,
                                        description: "HuggingFace search result".into(),
                                    })
                                    .collect();
                                if !search_models.is_empty() {
                                    self.models = search_models;
                                    self.model_idx = 0;
                                } else {
                                    self.search_query.clear();
                                }
                            }
                            Err(_) => {
                                self.search_query.clear();
                            }
                        }
                    }
                }
                _ => {}
            }
            return;
        }

        match action {
            InputAction::Up => {
                if self.model_idx > 0 {
                    self.model_idx -= 1;
                }
            }
            InputAction::Down => {
                if self.model_idx + 1 < self.models.len() {
                    self.model_idx += 1;
                }
            }
            InputAction::Select => {
                if self.model_idx < self.models.len() {
                    self.selected_model = Some(self.models[self.model_idx].clone());
                }
                self.start_build();
            }
            InputAction::Char('/') => {
                self.is_searching = true;
                self.search_query.clear();
            }
            InputAction::Char('s') | InputAction::Char('S') => {
                // Skip model selection
                self.selected_model = None;
                self.start_build();
            }
            InputAction::Back => {
                if !self.search_query.is_empty() {
                    self.search_query.clear();
                    if let Some(hw) = &self.hw {
                        self.models =
                            models::registry::recommend(hw.total_vram_mb(), hw.total_memory_mb());
                    }
                    self.model_idx = 0;
                } else {
                    self.screen = Screen::Backend;
                }
            }
            _ => {}
        }
    }

    fn start_build(&mut self) {
        self.screen = Screen::Install;
        self.build_log.clear();
        self.build_phase = BuildPhase::Clone;
        self.build_progress = 0.0;
        self.build_done = false;
        self.build_ok = false;

        // Run build synchronously (blocking).
        let prefix = self.config.resolved_prefix();
        if let (Some(hw), Some(backend)) = (&self.hw, &self.selected_backend) {
            match build::run_build(&prefix, hw, backend) {
                Ok(blog) => {
                    self.build_log = blog.lines;
                    self.build_phase = blog.phase;
                    self.build_ok = blog.success;
                }
                Err(e) => {
                    self.build_log.push(format!("Build error: {}", e));
                    self.build_ok = false;
                }
            }
        } else {
            self.build_log
                .push("Missing hardware or backend info.".into());
        }
        self.build_progress = 1.0;
        self.build_done = true;
    }

    fn handle_install(&mut self, action: InputAction) {
        if self.build_done && action == InputAction::Select {
            self.screen = Screen::Summary;
        }
    }

    fn handle_summary(&mut self, action: InputAction) {
        match action {
            InputAction::Select | InputAction::Back => {
                self.should_quit = true;
            }
            _ => {}
        }
    }
}
