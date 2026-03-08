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
    
    ShowWindow,
    HideWindow,
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
        let theme = AppTheme::midnight(); // TODO: load from config
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
                Task::none()
            }
            
            Message::ActionCompleted(msg) => {
                self.success_message = Some(msg);
                // Refresh after action
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
            
            Message::ShowWindow => {
                // Show and focus window - actual window control happens in main.rs wrapper
                Task::none()
            }
            Message::HideWindow => Task::none(), // Handled by tray
            Message::Exit => {
                // Save config before exit
                let _ = self.config.save();
                window::close(window::Id::unique())
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
        
        // Header with title and stats
        let stats = widget::row![
            widgets::stats_badge("Containers".to_string(), self.containers.len().to_string(), theme.background, theme.primary, theme.text, theme.text_muted),
            widgets::stats_badge("Running".to_string(), self.containers.iter().filter(|c| c.state == ContainerState::Running).count().to_string(), theme.background, theme.primary, theme.text, theme.text_muted),
            widgets::stats_badge("Stopped".to_string(), self.containers.iter().filter(|c| c.state == ContainerState::Exited).count().to_string(), theme.background, theme.primary, theme.text, theme.text_muted),
        ]
        .spacing(12);
        
        let header = widget::row![
            widget::text("Krabby Container").size(28).color(theme.primary).font(iced::Font::DEFAULT),
            widget::Space::with_width(Length::Fill),
            stats,
        ]
        .spacing(24)
        .align_y(Alignment::Center);
        
        // Toolbar
        let search_input = widget::text_input("Search containers...", &self.search_query)
            .on_input(Message::SearchChanged)
            .padding(12)
            .width(Length::Fixed(280.0));
        
        let filter_dropdown = widget::pick_list(
            vec![StateFilter::All, StateFilter::Running, StateFilter::Stopped],
            Some(self.state_filter),
            Message::StateFilterChanged,
        )
        .placeholder("All States")
        .width(Length::Fixed(140.0));
        
        let refresh_btn = widgets::styled_button("Refresh", theme).on_press(Message::RefreshContainers);
        let export_csv_btn = widgets::styled_button_secondary("CSV", theme).on_press(Message::ExportCsv);
        let export_json_btn = widgets::styled_button_secondary("JSON", theme).on_press(Message::ExportJson);
        let compose_btn = widgets::styled_button_secondary("Compose", theme).on_press(Message::GenerateCompose);
        let settings_btn = widgets::styled_button_secondary("Settings", theme).on_press(Message::ToggleSettings);
        
        let toolbar = widget::row![
            search_input,
            filter_dropdown,
            widget::Space::with_width(Length::Fill),
            refresh_btn,
            export_csv_btn,
            export_json_btn,
            compose_btn,
            settings_btn,
        ]
        .spacing(12)
        .align_y(Alignment::Center);
        
        // Container list
        let mut container_list = widget::Column::with_capacity(self.filtered_containers.len() + 1);
        container_list = container_list.push(widgets::header_row(theme));
        
        for container in &self.filtered_containers {
            let id = container.id.clone();
            container_list = container_list.push(
                widgets::container_row(
                    container,
                    theme,
                    Message::StartContainer(id.clone()),
                    Message::StopContainer(id.clone()),
                    Message::RestartContainer(id.clone()),
                    Message::RemoveContainer(id.clone()),
                )
            );
        }
        
        let scrollable = widget::scrollable(container_list.spacing(8))
            .height(Length::Fill)
            .style(|_t, _s| theme.scrollable_style());
        
        // Status messages
        let status_bar = if let Some(error) = &self.error_message {
            widget::container(widget::text(error).size(12).color(theme.error))
                .padding([8, 16])
                .style(|_t| {
                    iced::widget::container::Style {
                        background: Some(iced::Background::Color(theme.error.scale_alpha(0.1))),
                        border: iced::Border {
                            radius: 8.0.into(),
                            width: 1.0,
                            color: theme.error.scale_alpha(0.3),
                        },
                        ..Default::default()
                    }
                })
                .into()
        } else if let Some(success) = &self.success_message {
            widget::container(widget::text(success).size(12).color(theme.success))
                .padding([8, 16])
                .style(|_t| {
                    iced::widget::container::Style {
                        background: Some(iced::Background::Color(theme.success.scale_alpha(0.1))),
                        border: iced::Border {
                            radius: 8.0.into(),
                            width: 1.0,
                            color: theme.success.scale_alpha(0.3),
                        },
                        ..Default::default()
                    }
                })
                .into()
        } else {
            Element::from(widget::Space::with_height(Length::Fixed(0.0)))
        };
        
        // Settings panel (if shown)
        let main_content: Element<_> = if self.show_settings {
            widget::column![
                header,
                toolbar,
                self.settings_view(),
                status_bar,
            ]
            .spacing(16)
            .padding(24)
            .into()
        } else {
            widget::column![
                header,
                toolbar,
                scrollable,
                status_bar,
            ]
            .spacing(16)
            .padding(24)
            .into()
        };
        
        widget::container(main_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_t| {
                iced::widget::container::Style {
                    background: Some(iced::Background::Color(theme.background)),
                    ..Default::default()
                }
            })
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        // Auto-refresh subscription
        iced::time::every(Duration::from_secs(self.config.auto_refresh_interval)).map(|_| Message::Tick)
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

impl ContainerManagerApp {
    fn settings_view(&self) -> Element<'_, Message> {
        let theme = &self.theme;
        
        let theme_section = widget::column![
            widget::text("Appearance").size(18).color(theme.text),
            widget::row![
                widgets::styled_button("Midnight", theme).on_press(Message::ChangeTheme("midnight".to_string())),
                widgets::styled_button("Ocean", theme).on_press(Message::ChangeTheme("ocean".to_string())),
                widgets::styled_button("Forest", theme).on_press(Message::ChangeTheme("forest".to_string())),
                widgets::styled_button("Rose", theme).on_press(Message::ChangeTheme("rose".to_string())),
                widgets::styled_button("Amber", theme).on_press(Message::ChangeTheme("amber".to_string())),
            ]
            .spacing(8),
        ]
        .spacing(12);
        
        widget::container(
            widget::column![
                widget::text("Settings").size(24).color(theme.primary),
                theme_section,
            ]
            .spacing(24)
        )
        .padding(24)
        .style(|_t| theme.container_style())
        .into()
    }
}
