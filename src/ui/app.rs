use iced::{
    widget, 
    window, Task, 
    Alignment, Element, Length, Subscription, Theme, 
};


use std::path::PathBuf;
use std::time::Duration;

use crate::config::AppConfig;
use crate::docker::{ContainerInfo, ContainerState, DockerClient};
use crate::export;
use crate::ui::theme::AppTheme;
use crate::ui::widgets;

#[derive(Debug, Clone)]
pub enum Message {
    RefreshContainers,
    ContainersLoaded(Vec<ContainerInfo>),
    Error(String),
    
    StartContainer(String),
    StopContainer(String),
    RestartContainer(String),
    RemoveContainer(String),
    ActionCompleted(String),
    
    SearchChanged(String),
    StateFilterChanged(StateFilter),
    SortBy(SortColumn),
    
    ExportCsv,
    ExportJson,
    GenerateCompose,
    ExportCompleted(String),
    
    ChangeTheme(String),
    ToggleSettings,
    
    // Logs
    ViewLogs(String),
    LogsLoaded(String, String),
    CloseLogs,
    
    ShowWindow,
    HideWindow,
    CloseRequested(window::Id),
    Exit,
    
    Tick,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Name,
    Image,
    State,
    Created,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StateFilter {
    #[default]
    All,
    Running,
    Stopped,
}

impl std::fmt::Display for StateFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StateFilter::All => write!(f, "All States"),
            StateFilter::Running => write!(f, "Running"),
            StateFilter::Stopped => write!(f, "Stopped"),
        }
    }
}

pub struct ContainerManagerApp {
    containers: Vec<ContainerInfo>,
    filtered_containers: Vec<ContainerInfo>,
    docker: DockerClient,
    config: AppConfig,
    theme: AppTheme,
    
    search_query: String,
    state_filter: StateFilter,
    sort_column: SortColumn,
    sort_ascending: bool,
    
    error_message: Option<String>,
    success_message: Option<String>,
    show_settings: bool,
    last_refresh: std::time::Instant,
    
    // Logs modal state
    logs_container: Option<String>,
    logs_content: Option<String>,
    logs_loading: bool,
}

impl ContainerManagerApp {
    fn apply_filters(&mut self) {
        let mut filtered: Vec<ContainerInfo> = self.containers
            .clone()
            .into_iter()
            .filter(|c| {
                let matches_search = self.search_query.is_empty()
                    || c.name.to_lowercase().contains(&self.search_query.to_lowercase())
                    || c.image.to_lowercase().contains(&self.search_query.to_lowercase())
                    || c.short_id.to_lowercase().contains(&self.search_query.to_lowercase());
                
                let matches_state = match self.state_filter {
                    StateFilter::All => true,
                    StateFilter::Running => c.state == ContainerState::Running,
                    StateFilter::Stopped => c.state == ContainerState::Exited || c.state == ContainerState::Dead,
                };
                
                matches_search && matches_state
            })
            .collect();
        
        // Apply sorting
        filtered.sort_by(|a, b| {
            let ordering = match self.sort_column {
                SortColumn::Name => a.name.cmp(&b.name),
                SortColumn::Image => a.image.cmp(&b.image),
                SortColumn::State => a.state.to_string().cmp(&b.state.to_string()),
                SortColumn::Created => a.created.cmp(&b.created),
            };
            if self.sort_ascending { ordering } else { ordering.reverse() }
        });
        
        self.filtered_containers = filtered;
    }
    
    fn refresh_containers(&self) -> Task<Message> {
        let docker = self.docker.clone();
        Task::perform(
            async move {
                match docker.list_containers(true).await {
                    Ok(containers) => Message::ContainersLoaded(containers),
                    Err(e) => Message::Error(e.to_string()),
                }
            },
            |msg| msg,
        )
    }
    
    fn perform_action(&self, id: String, action: Action) -> Task<Message> {
        let docker = self.docker.clone();
        Task::perform(
            async move {
                let result = match action {
                    Action::Start => docker.start_container(&id).await,
                    Action::Stop => docker.stop_container(&id, Some(10)).await,
                    Action::Restart => docker.restart_container(&id, Some(10)).await,
                    Action::Remove => docker.remove_container(&id, true).await,
                };
                match result {
                    Ok(_) => Message::ActionCompleted(format!("{:?} completed", action)),
                    Err(e) => Message::Error(e.to_string()),
                }
            },
            |msg| msg,
        )
    }
    
    fn load_logs(&self, id: String) -> Task<Message> {
        let docker = self.docker.clone();
        Task::perform(
            async move {
                match docker.get_container_logs(&id, 500).await {
                    Ok(logs) => Message::LogsLoaded(id, logs),
                    Err(e) => Message::Error(format!("Failed to load logs: {}", e)),
                }
            },
            |msg| msg,
        )
    }
    
    fn export_csv(&self) -> Task<Message> {
        let containers = self.filtered_containers.clone();
        Task::perform(
            async move {
                let path = PathBuf::from("containers_export.csv");
                match export::export_to_csv(&containers, &path) {
                    Ok(_) => Message::ExportCompleted(format!("Exported to {}", path.display())),
                    Err(e) => Message::Error(e.to_string()),
                }
            },
            |msg| msg,
        )
    }
    
    fn export_json(&self) -> Task<Message> {
        let containers = self.filtered_containers.clone();
        Task::perform(
            async move {
                let path = PathBuf::from("containers_export.json");
                match export::export_to_json(&containers, &path) {
                    Ok(_) => Message::ExportCompleted(format!("Exported to {}", path.display())),
                    Err(e) => Message::Error(e.to_string()),
                }
            },
            |msg| msg,
        )
    }
    
    fn generate_compose(&self) -> Task<Message> {
        let containers = self.filtered_containers.clone();
        Task::perform(
            async move {
                match export::generate_compose(&containers) {
                    Ok(compose) => {
                        let path = PathBuf::from("docker-compose.yml");
                        match std::fs::write(&path, compose) {
                            Ok(_) => Message::ExportCompleted(format!("Compose file saved to {}", path.display())),
                            Err(e) => Message::Error(e.to_string()),
                        }
                    }
                    Err(e) => Message::Error(e.to_string()),
                }
            },
            |msg| msg,
        )
    }

    pub fn new() -> (Self, Task<Message>) {
        let config = AppConfig::load().unwrap_or_default();
        let theme = AppTheme::midnight();
        let docker = DockerClient::new().expect("Failed to connect to Docker");
        
        let app = Self {
            containers: Vec::new(),
            filtered_containers: Vec::new(),
            docker,
            config,
            theme,
            search_query: String::new(),
            state_filter: StateFilter::default(),
            sort_column: SortColumn::Name,
            sort_ascending: true,
            error_message: None,
            success_message: None,
            show_settings: false,
            last_refresh: std::time::Instant::now(),
            logs_container: None,
            logs_content: None,
            logs_loading: false,
        };
        
        (app, Task::perform(async {}, |_| Message::RefreshContainers))
    }

    pub fn title(&self) -> String {
        String::from("Krabby Container")
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::RefreshContainers => {
                self.last_refresh = std::time::Instant::now();
                self.error_message = None;
                self.refresh_containers()
            }
            
            Message::ContainersLoaded(containers) => {
                self.containers = containers;
                self.apply_filters();
                Task::none()
            }
            
            Message::Error(msg) => {
                self.error_message = Some(msg);
                self.logs_loading = false;
                Task::none()
            }
            
            Message::ActionCompleted(msg) => {
                self.success_message = Some(msg);
                self.refresh_containers()
            }
            
            Message::StartContainer(id) => self.perform_action(id, Action::Start),
            Message::StopContainer(id) => self.perform_action(id, Action::Stop),
            Message::RestartContainer(id) => self.perform_action(id, Action::Restart),
            Message::RemoveContainer(id) => self.perform_action(id, Action::Remove),
            
            Message::SearchChanged(query) => {
                self.search_query = query;
                self.apply_filters();
                Task::none()
            }
            
            Message::StateFilterChanged(state) => {
                self.state_filter = state;
                self.apply_filters();
                Task::none()
            }
            
            Message::SortBy(column) => {
                if self.sort_column == column {
                    self.sort_ascending = !self.sort_ascending;
                } else {
                    self.sort_column = column;
                    self.sort_ascending = true;
                }
                self.apply_filters();
                Task::none()
            }
            
            Message::ExportCsv => self.export_csv(),
            Message::ExportJson => self.export_json(),
            Message::GenerateCompose => self.generate_compose(),
            
            Message::ExportCompleted(msg) => {
                self.success_message = Some(msg);
                Task::none()
            }
            
            Message::ChangeTheme(theme_name) => {
                self.theme = match theme_name.as_str() {
                    "ocean" => AppTheme::ocean(),
                    "forest" => AppTheme::forest(),
                    "rose" => AppTheme::rose(),
                    "amber" => AppTheme::amber(),
                    _ => AppTheme::midnight(),
                };
                Task::none()
            }
            
            Message::ToggleSettings => {
                self.show_settings = !self.show_settings;
                Task::none()
            }
            
            // Logs handling
            Message::ViewLogs(id) => {
                self.logs_container = Some(id.clone());
                self.logs_content = None;
                self.logs_loading = true;
                self.load_logs(id)
            }
            
            Message::LogsLoaded(id, logs) => {
                if self.logs_container.as_ref() == Some(&id) {
                    self.logs_content = Some(logs);
                    self.logs_loading = false;
                }
                Task::none()
            }
            
            Message::CloseLogs => {
                self.logs_container = None;
                self.logs_content = None;
                self.logs_loading = false;
                Task::none()
            }
            
            // These are handled by the wrapper update in main.rs
            // They should never reach here, but just in case:
            Message::ShowWindow => Task::none(),
            Message::HideWindow => Task::none(),
            Message::CloseRequested(_) => Task::none(),
            Message::Exit => {
                let _ = self.config.save();
                Task::none()
            }
            
            Message::Tick => {
                if self.last_refresh.elapsed().as_secs() >= self.config.auto_refresh_interval {
                    return self.refresh_containers();
                }
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let theme = &self.theme;
        
        // Main content based on mode
        if let Some(container_id) = &self.logs_container {
            self.logs_view(container_id, theme)
        } else if self.show_settings {
            self.main_view(theme)
        } else {
            self.main_view(theme)
        }
    }
    
    fn main_view<'a>(&'a self, theme: &'a AppTheme) -> Element<'a, Message> {
        // Header with stats
        let running_count = self.containers.iter().filter(|c| c.state == ContainerState::Running).count();
        let stopped_count = self.containers.iter().filter(|c| c.state == ContainerState::Exited).count();
        
        let header = widget::row![
            widget::column![
                widget::text("Krabby Container").size(32).color(theme.primary).font(iced::Font::DEFAULT),
                widget::text("Container Management").size(12).color(theme.text_muted),
            ],
            widget::Space::with_width(Length::Fill),
            widgets::stats_row(running_count, stopped_count, self.containers.len(), theme),
        ]
        .spacing(24)
        .align_y(Alignment::Center);
        
        // Toolbar
        let search_input = widget::text_input("Search containers...", &self.search_query)
            .on_input(Message::SearchChanged)
            .padding(10)
            .width(Length::Fixed(280.0));
        
        let filter_dropdown = widget::pick_list(
            vec![StateFilter::All, StateFilter::Running, StateFilter::Stopped],
            Some(self.state_filter),
            Message::StateFilterChanged,
        )
        .placeholder("All States")
        .width(Length::Fixed(130.0));
        
        let refresh_btn = widgets::icon_button("↻", "Refresh", theme)
            .on_press(Message::RefreshContainers);
        let export_btn = widgets::icon_button("⬇", "Export", theme)
            .on_press(Message::ExportCsv);
        let settings_btn = widgets::icon_button("⚙", "Settings", theme)
            .on_press(Message::ToggleSettings);
        
        let toolbar = widget::row![
            search_input,
            filter_dropdown,
            widget::Space::with_width(Length::Fill),
            refresh_btn,
            export_btn,
            settings_btn,
        ]
        .spacing(10)
        .align_y(Alignment::Center);
        
        // Container table
        let container_table: Element<_> = if self.filtered_containers.is_empty() {
            widget::container(
                widget::column![
                    widget::Space::with_height(Length::Fixed(100.0)),
                    widget::text("No containers found").size(18).color(theme.text_muted),
                    widget::text("Try adjusting your search or filters").size(12).color(theme.text_muted),
                ]
                .align_x(Alignment::Center)
            )
            .center(Length::Fill)
            .into()
        } else {
            let mut table = widget::Column::new();
            
            // Header row
            table = table.push(widgets::table_header(theme));
            
            // Container rows
            for container in &self.filtered_containers {
                table = table.push(widgets::container_row(
                    container,
                    theme,
                    Message::StartContainer(container.id.clone()),
                    Message::StopContainer(container.id.clone()),
                    Message::RestartContainer(container.id.clone()),
                    Message::RemoveContainer(container.id.clone()),
                    Message::ViewLogs(container.id.clone()),
                ));
            }
            
            widget::scrollable(table.spacing(2))
                .height(Length::Fill)
                .style(|_t, _s| theme.scrollable_style())
                .into()
        };
        
        // Status bar
        let status_bar = self.status_bar(theme);
        
        // Main content
        let content = widget::column![
            header,
            toolbar,
            container_table,
            status_bar,
        ]
        .spacing(16)
        .padding(20);
        
        widget::container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_t| theme.main_container_style())
            .into()
    }
    
    fn logs_view<'a>(&'a self, container_id: &'a str, theme: &'a AppTheme) -> Element<'a, Message> {
        let container = self.containers.iter().find(|c| c.id == container_id);
        let container_name = container.map(|c| c.name.clone()).unwrap_or_else(|| container_id[..12].to_string());
        
        // Header
        let header = widget::row![
            widget::text(format!("Logs: {}", container_name))
                .size(20)
                .color(theme.text),
            widget::Space::with_width(Length::Fill),
            widgets::styled_button("Close", theme).on_press(Message::CloseLogs),
        ]
        .align_y(Alignment::Center);
        
        // Logs content
        let logs_content: Element<_> = if self.logs_loading {
            widget::container(
                widget::text("Loading logs...").color(theme.text_muted)
            )
            .center(Length::Fill)
            .into()
        } else if let Some(logs) = &self.logs_content {
            widget::container(
                widget::scrollable(
                    widget::text(logs)
                        .size(11)
                        .font(iced::Font::MONOSPACE)
                        .color(theme.text)
                )
                .height(Length::Fill)
            )
            .padding(10)
            .style(|_t| theme.logs_container_style())
            .into()
        } else {
            widget::container(
                widget::text("No logs available").color(theme.text_muted)
            )
            .center(Length::Fill)
            .into()
        };
        
        let content = widget::column![
            header,
            logs_content,
        ]
        .spacing(12)
        .padding(20);
        
        widget::container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_t| theme.main_container_style())
            .into()
    }
    
    fn status_bar<'a>(&'a self, theme: &'a AppTheme) -> Element<'a, Message> {
        let status = if let Some(error) = &self.error_message {
            widget::row![
                widget::text("●").color(theme.error),
                widget::text(error).size(12).color(theme.error),
            ]
            .spacing(8)
        } else if let Some(success) = &self.success_message {
            widget::row![
                widget::text("●").color(theme.success),
                widget::text(success).size(12).color(theme.success),
            ]
            .spacing(8)
        } else {
            let refresh_ago = self.last_refresh.elapsed().as_secs();
            let time_text = if refresh_ago < 60 {
                format!("Updated {}s ago", refresh_ago)
            } else {
                format!("Updated {}m ago", refresh_ago / 60)
            };
            widget::row![
                widget::text("●").color(theme.success),
                widget::text(time_text).size(12).color(theme.text_muted),
            ]
            .spacing(8)
        };
        
        widget::container(status)
            .padding([8, 12])
            .style(|_t| theme.status_bar_style())
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        iced::time::every(Duration::from_secs(self.config.auto_refresh_interval))
            .map(|_| Message::Tick)
    }

    pub fn theme(&self) -> Theme {
        self.theme.to_iced_theme()
    }
}

#[derive(Debug, Clone, Copy)]
enum Action {
    Start,
    Stop,
    Restart,
    Remove,
}
