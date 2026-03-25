use std::sync::mpsc;
use std::time::Duration;

use crate::build::{self, BuildPhase};
use crate::compute::slot_advisor::{SlotAdvisor, SlotAdvisorInput};
use crate::compute::kv_cache::KvDtype;
use crate::config::backends::{auto_select, available_backends, make_backend};
use crate::config::{AppConfig, AppState, Backend, BackendName, ExpertiseLevel};
use crate::hardware::{self, HardwareInfo};
use crate::inference::session::{ChatMessage, ChatRole, ChatSession};
use crate::models::{self, ModelEntry};
use crate::models::fit::{self, FitLevel};
use crate::server::config::ServerConfig;
use crate::tui::{self, InputAction};

/// Messages sent from the build thread to the TUI.
pub enum BuildMessage {
    Log(String),
    Phase(BuildPhase),
    Progress(f64),
    Done { success: bool },
}

/// The screens the TUI can display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Welcome,
    Hardware,
    Backend,
    LlamaSwap,
    Models,
    Install,
    Summary,
    Chat,
    Settings,
    Logs,
}

/// Full application state, driving the TUI render loop.
pub struct App {
    pub screen: Screen,
    pub config: AppConfig,
    pub state: AppState,
    pub expertise_idx: usize,
    pub detected_platform: String,
    pub hw: Option<HardwareInfo>,
    // VRAM override
    pub vram_override: Option<u64>,
    pub editing_vram: bool,
    pub vram_input: String,
    // Backend selection
    pub backends: Vec<BackendName>,
    pub backend_idx: usize,
    pub selected_backend: Option<Backend>,
    pub recommended_backend: Option<BackendName>,
    // llama-swap
    pub llama_swap_idx: usize,
    pub llama_swap_installed: bool,
    // Models
    pub models: Vec<ModelEntry>,
    pub model_fits: Vec<FitLevel>,
    pub model_checked: Vec<bool>,
    pub model_idx: usize,
    pub selected_model: Option<ModelEntry>,
    pub search_query: String,
    pub is_searching: bool,
    // Build
    pub build_log: Vec<String>,
    pub build_phase: BuildPhase,
    pub build_progress: f64,
    pub build_done: bool,
    pub build_ok: bool,
    pub build_rx: Option<mpsc::Receiver<BuildMessage>>,
    // Chat state
    pub chat_session: ChatSession,
    pub chat_input: String,
    pub chat_scroll: usize,
    pub is_generating: bool,
    // Settings state
    pub server_config: ServerConfig,
    pub settings_row: usize,
    pub slot_recommendation: Option<crate::compute::slot_advisor::SlotRecommendation>,
    // Log viewer state
    pub log_lines: Vec<String>,
    pub log_scroll: usize,
    pub log_filter: String,
    pub should_quit: bool,
}

impl App {
    pub fn new(config: AppConfig) -> Self {
        // Load persisted state (expertise, setup progress, etc.)
        let state = AppState::load().unwrap_or_default();
        let expertise_idx = ExpertiseLevel::all()
            .iter()
            .position(|l| *l == state.expertise)
            .unwrap_or(1);

        // Quick platform detection for the welcome screen
        let detected_platform = detect_platform_summary();

        Self {
            screen: Screen::Welcome,
            config,
            state,
            expertise_idx,
            detected_platform,
            hw: None,
            vram_override: None,
            editing_vram: false,
            vram_input: String::new(),
            backends: Vec::new(),
            backend_idx: 0,
            selected_backend: None,
            recommended_backend: None,
            llama_swap_idx: 0,
            llama_swap_installed: false,
            models: Vec::new(),
            model_fits: Vec::new(),
            model_checked: Vec::new(),
            model_idx: 0,
            selected_model: None,
            search_query: String::new(),
            is_searching: false,
            build_log: Vec::new(),
            build_phase: BuildPhase::Clone,
            build_progress: 0.0,
            build_done: false,
            build_ok: false,
            build_rx: None,
            chat_session: ChatSession::new(4096),
            chat_input: String::new(),
            chat_scroll: 0,
            is_generating: false,
            server_config: ServerConfig::default(),
            settings_row: 0,
            slot_recommendation: None,
            log_lines: Vec::new(),
            log_scroll: 0,
            log_filter: String::new(),
            should_quit: false,
        }
    }

    /// Effective VRAM in MB (user override or detected).
    fn effective_vram(&self) -> u64 {
        self.vram_override
            .unwrap_or_else(|| self.hw.as_ref().map(|h| h.gpu.vram_mb).unwrap_or(0))
    }

    /// Effective RAM in MB.
    fn effective_ram(&self) -> u64 {
        self.hw.as_ref().map(|h| h.memory.total_mb).unwrap_or(0)
    }

    /// Re-score model fits with current VRAM/RAM.
    fn refresh_model_fits(&mut self) {
        let vram = self.effective_vram();
        let ram = self.effective_ram();
        self.model_fits = self
            .models
            .iter()
            .map(|m| fit::score_fit(m, vram, ram))
            .collect();
    }

    /// Populate recommended models and score their fits.
    fn populate_models(&mut self) {
        let vram = self.effective_vram();
        let ram = self.effective_ram();
        self.models = models::registry::recommend(vram, ram);
        self.refresh_model_fits();
        self.model_checked = vec![false; self.models.len()];
        self.model_idx = 0;
    }

    /// Compute SlotAdvisor recommendation for current hardware + model.
    fn compute_slot_recommendation(&mut self) {
        let vram = self.effective_vram() as usize;
        let model_size = self.selected_model.as_ref()
            .map(|m| m.min_vram_mb as usize)
            .unwrap_or(2048);

        let input = SlotAdvisorInput {
            model_size_mb: model_size,
            total_vram_mb: vram,
            context_length: self.chat_session.max_context,
            kv_dtype: KvDtype::F16,
            target_concurrent_users: 1,
        };

        let advisor = SlotAdvisor::new();
        self.slot_recommendation = Some(advisor.recommend(&input));
    }

    /// Run the TUI main loop.
    pub fn run(&mut self) -> anyhow::Result<()> {
        let mut terminal = tui::init_terminal()?;

        while !self.should_quit {
            // Drain build messages from background thread (non-blocking)
            self.drain_build_messages();

            terminal.draw(|frame| self.render(frame))?;
            let action = tui::poll_input(
                Duration::from_millis(50),
                self.is_searching || self.editing_vram,
            );
            self.handle_input(action);
        }

        tui::restore_terminal(&mut terminal)?;
        Ok(())
    }

    /// Non-blocking drain of build channel messages.
    fn drain_build_messages(&mut self) {
        if let Some(rx) = &self.build_rx {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    BuildMessage::Log(line) => {
                        self.build_log.push(line);
                    }
                    BuildMessage::Phase(phase) => {
                        self.build_phase = phase;
                    }
                    BuildMessage::Progress(pct) => {
                        self.build_progress = pct;
                    }
                    BuildMessage::Done { success } => {
                        self.build_done = true;
                        self.build_ok = success;
                        self.build_progress = 1.0;
                    }
                }
            }
        }
    }

    fn render(&self, frame: &mut ratatui::Frame) {
        let area = frame.area();
        match self.screen {
            Screen::Welcome => {
                tui::views::welcome::render(
                    frame,
                    area,
                    &self.detected_platform,
                    self.expertise_idx,
                );
            }
            Screen::Hardware => {
                if let Some(hw) = &self.hw {
                    tui::views::hardware::render(
                        frame,
                        area,
                        hw,
                        self.vram_override,
                        self.editing_vram,
                        &self.vram_input,
                    );
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
            Screen::LlamaSwap => {
                tui::views::llama_swap::render(
                    frame,
                    area,
                    self.llama_swap_idx,
                    self.llama_swap_installed,
                );
            }
            Screen::Models => {
                tui::views::models::render(
                    frame,
                    area,
                    &self.models,
                    &self.model_fits,
                    self.model_idx,
                    &self.model_checked,
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
            Screen::Chat => {
                let model_name = self
                    .selected_model
                    .as_ref()
                    .map(|m| m.name.as_str())
                    .unwrap_or("(none)");
                tui::views::chat::render(
                    frame,
                    area,
                    &self.chat_session.messages,
                    &self.chat_input,
                    model_name,
                    self.chat_scroll,
                    0.0,
                    self.is_generating,
                );
            }
            Screen::Settings => {
                let slots = self.slot_recommendation.as_ref()
                    .map(|r| r.parallel_slots)
                    .unwrap_or(1);
                tui::views::settings::render(
                    frame,
                    area,
                    &self.server_config.host,
                    self.server_config.port,
                    self.server_config.tls.is_some(),
                    slots,
                    self.chat_session.max_context,
                    self.settings_row,
                );
            }
            Screen::Logs => {
                tui::views::logs::render(
                    frame,
                    area,
                    &self.log_lines,
                    self.log_scroll,
                    &self.log_filter,
                );
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
            Screen::LlamaSwap => self.handle_llama_swap(action),
            Screen::Models => self.handle_models(action),
            Screen::Install => self.handle_install(action),
            Screen::Summary => self.handle_summary(action),
            Screen::Chat => self.handle_chat(action),
            Screen::Settings => self.handle_settings(action),
            Screen::Logs => self.handle_logs(action),
        }
    }

    fn handle_welcome(&mut self, action: InputAction) {
        let levels = ExpertiseLevel::all();
        match action {
            InputAction::Up => {
                if self.expertise_idx > 0 {
                    self.expertise_idx -= 1;
                }
            }
            InputAction::Down => {
                if self.expertise_idx + 1 < levels.len() {
                    self.expertise_idx += 1;
                }
            }
            InputAction::Select => {
                // Persist selected expertise level
                if self.expertise_idx < levels.len() {
                    self.state.expertise = levels[self.expertise_idx];
                    let _ = self.state.save();
                }

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
                        // Update platform string with full hardware info
                        self.detected_platform = format!(
                            "{} | {} | {} VRAM",
                            hw.os,
                            hw.gpu.name,
                            HardwareInfo::format_memory(hw.gpu.vram_mb),
                        );
                        self.hw = Some(hw);
                        self.screen = Screen::Hardware;
                    }
                    Err(e) => {
                        self.build_log
                            .push(format!("Hardware detection failed: {}", e));
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_hardware(&mut self, action: InputAction) {
        // If editing VRAM override, handle text input
        if self.editing_vram {
            match action {
                InputAction::Char(c) if c.is_ascii_digit() => {
                    self.vram_input.push(c);
                }
                InputAction::Back => {
                    if self.vram_input.is_empty() {
                        self.editing_vram = false;
                    } else {
                        self.vram_input.pop();
                    }
                }
                InputAction::Select => {
                    // Parse and apply VRAM override
                    if let Ok(val) = self.vram_input.parse::<u64>() {
                        if val > 0 {
                            self.vram_override = Some(val);
                        }
                    }
                    self.editing_vram = false;
                    self.vram_input.clear();
                }
                _ => {}
            }
            return;
        }

        match action {
            InputAction::Char('v') | InputAction::Char('V') => {
                self.editing_vram = true;
                self.vram_input.clear();
            }
            InputAction::Char('r') | InputAction::Char('R') => {
                self.vram_override = None;
            }
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

                    // Move to llama-swap screen
                    self.llama_swap_idx = 0;
                    self.screen = Screen::LlamaSwap;
                }
            }
            InputAction::Back => {
                self.screen = Screen::Hardware;
            }
            _ => {}
        }
    }

    fn handle_llama_swap(&mut self, action: InputAction) {
        match action {
            InputAction::Up => {
                if self.llama_swap_idx > 0 {
                    self.llama_swap_idx -= 1;
                }
            }
            InputAction::Down => {
                let max = if self.llama_swap_installed { 1 } else { 1 };
                if self.llama_swap_idx < max {
                    self.llama_swap_idx += 1;
                }
            }
            InputAction::Select => {
                if self.llama_swap_installed {
                    // Already installed — continue to models
                    self.populate_models();
                    self.screen = Screen::Models;
                } else if self.llama_swap_idx == 0 {
                    // Install selected — stub for now, just mark as installed
                    self.llama_swap_installed = true;
                    self.llama_swap_idx = 0;
                } else {
                    // Skip — continue to models
                    self.populate_models();
                    self.screen = Screen::Models;
                }
            }
            InputAction::Back => {
                self.screen = Screen::Backend;
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
                                        params: "?".into(),
                                        quantization: "Q4_K_M".into(),
                                        size_label: format!("{} downloads", r.downloads),
                                        context_length: "?".into(),
                                        min_vram_mb: 0,
                                        min_ram_mb: 0,
                                        description: "HuggingFace search result".into(),
                                        estimated_toks: "—".into(),
                                    })
                                    .collect();
                                if !search_models.is_empty() {
                                    self.models = search_models;
                                    self.refresh_model_fits();
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
            InputAction::Char(' ') => {
                // Toggle multi-select checkbox
                if self.model_idx < self.model_checked.len() {
                    self.model_checked[self.model_idx] = !self.model_checked[self.model_idx];
                }
            }
            InputAction::Select => {
                // Download all checked, or single selected
                let checked: Vec<ModelEntry> = self.model_checked.iter()
                    .enumerate()
                    .filter(|(_, c)| **c)
                    .filter_map(|(i, _)| self.models.get(i).cloned())
                    .collect();

                if !checked.is_empty() {
                    self.selected_model = Some(checked[0].clone());
                } else if self.model_idx < self.models.len() {
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
                    self.populate_models();
                } else {
                    self.screen = Screen::LlamaSwap;
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

        let prefix = self.config.resolved_prefix();
        if let (Some(hw), Some(backend)) = (&self.hw, &self.selected_backend) {
            let hw = hw.clone();
            let backend = backend.clone();
            let (tx, rx) = mpsc::channel();
            self.build_rx = Some(rx);

            // Spawn build in a background thread so the TUI stays responsive.
            std::thread::spawn(move || {
                let _ = tx.send(BuildMessage::Log("Starting build...".into()));
                let _ = tx.send(BuildMessage::Phase(BuildPhase::Preflight));
                let _ = tx.send(BuildMessage::Progress(0.05));

                match build::run_build(&prefix, &hw, &backend) {
                    Ok(blog) => {
                        for line in blog.lines {
                            let _ = tx.send(BuildMessage::Log(line));
                        }
                        let _ = tx.send(BuildMessage::Phase(blog.phase));
                        let _ = tx.send(BuildMessage::Done { success: blog.success });
                    }
                    Err(e) => {
                        let _ = tx.send(BuildMessage::Log(format!("Build error: {}", e)));
                        let _ = tx.send(BuildMessage::Done { success: false });
                    }
                }
            });
        } else {
            self.build_log
                .push("Missing hardware or backend info.".into());
            self.build_done = true;
        }
    }

    fn handle_install(&mut self, action: InputAction) {
        if self.build_done && action == InputAction::Select {
            self.screen = Screen::Summary;
        }
    }

    fn handle_summary(&mut self, action: InputAction) {
        match action {
            InputAction::Select => {
                // Transition to workspace (Chat)
                self.compute_slot_recommendation();
                self.screen = Screen::Chat;
            }
            InputAction::Back => {
                self.should_quit = true;
            }
            _ => {}
        }
    }

    fn handle_chat(&mut self, action: InputAction) {
        match action {
            InputAction::Char(c) => {
                if !self.is_generating {
                    self.chat_input.push(c);
                }
            }
            InputAction::Back => {
                if !self.chat_input.is_empty() {
                    self.chat_input.pop();
                } else {
                    // Esc with empty input goes to summary
                    self.screen = Screen::Summary;
                }
            }
            InputAction::Select => {
                if !self.is_generating && !self.chat_input.is_empty() {
                    // Send message
                    let user_msg = ChatMessage::new(
                        ChatRole::User,
                        self.chat_input.clone(),
                    );
                    self.chat_session.add_message(user_msg);
                    self.chat_input.clear();
                    // Reset scroll to bottom
                    self.chat_scroll = 0;
                    // TODO: trigger actual inference via engine
                    // For now, add a placeholder assistant response
                    let assistant_msg = ChatMessage::new(
                        ChatRole::Assistant,
                        "(Inference engine not connected yet — this is a placeholder response.)".into(),
                    );
                    self.chat_session.add_message(assistant_msg);
                }
            }
            InputAction::Up => {
                self.chat_scroll = self.chat_scroll.saturating_add(3);
            }
            InputAction::Down => {
                self.chat_scroll = self.chat_scroll.saturating_sub(3);
            }
            InputAction::Tab => {
                self.screen = Screen::Settings;
            }
            _ => {}
        }
    }

    fn handle_settings(&mut self, action: InputAction) {
        match action {
            InputAction::Up => {
                if self.settings_row > 0 {
                    self.settings_row -= 1;
                }
            }
            InputAction::Down => {
                if self.settings_row < 4 {
                    self.settings_row += 1;
                }
            }
            InputAction::Char('l') | InputAction::Char('L') => {
                self.log_scroll = 0;
                self.screen = Screen::Logs;
            }
            InputAction::Tab => {
                self.screen = Screen::Chat;
            }
            InputAction::Back => {
                self.screen = Screen::Chat;
            }
            _ => {}
        }
    }

    fn handle_logs(&mut self, action: InputAction) {
        match action {
            InputAction::Up => {
                self.log_scroll = self.log_scroll.saturating_add(3);
            }
            InputAction::Down => {
                self.log_scroll = self.log_scroll.saturating_sub(3);
            }
            InputAction::Char('e') | InputAction::Char('E') => {
                self.log_filter = "ERROR".into();
                self.log_scroll = 0;
            }
            InputAction::Char('w') | InputAction::Char('W') => {
                self.log_filter = "WARN".into();
                self.log_scroll = 0;
            }
            InputAction::Char('i') | InputAction::Char('I') => {
                self.log_filter = "INFO".into();
                self.log_scroll = 0;
            }
            InputAction::Char('d') | InputAction::Char('D') => {
                self.log_filter = "DEBUG".into();
                self.log_scroll = 0;
            }
            InputAction::Char('c') | InputAction::Char('C') => {
                self.log_filter.clear();
                self.log_scroll = 0;
            }
            InputAction::Back | InputAction::Tab => {
                self.screen = Screen::Settings;
            }
            _ => {}
        }
    }
}

/// Quick platform summary for the welcome screen (before full hardware detection).
fn detect_platform_summary() -> String {
    let os = if cfg!(target_os = "linux") {
        if std::path::Path::new("/proc/version").exists() {
            if let Ok(contents) = std::fs::read_to_string("/proc/version") {
                if contents.contains("Microsoft") || contents.contains("WSL") {
                    "Linux (WSL)"
                } else {
                    "Linux"
                }
            } else {
                "Linux"
            }
        } else {
            "Linux"
        }
    } else if cfg!(target_os = "macos") {
        "macOS"
    } else if cfg!(target_os = "windows") {
        "Windows"
    } else {
        std::env::consts::OS
    };

    let arch = std::env::consts::ARCH;
    format!("{} ({})", os, arch)
}
