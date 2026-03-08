use iced::{
    widget,
    Alignment, Element, Length, Theme,
};

use crate::docker::{ContainerInfo, ContainerState};
use crate::ui::theme::AppTheme;
use crate::ui::app::Message;

/// Stats row showing running/stopped/total counts
pub fn stats_row(running: usize, stopped: usize, total: usize, theme: &AppTheme) -> Element<'_, Message> {
    let running_badge = stat_badge("Running", running, theme.success, theme);
    let stopped_badge = stat_badge("Stopped", stopped, theme.warning, theme);
    let total_badge = stat_badge("Total", total, theme.primary, theme);
    
    widget::row![running_badge, stopped_badge, total_badge]
        .spacing(12)
        .into()
}

fn stat_badge<'a>(label: &'a str, value: usize, color: iced::Color, theme: &'a AppTheme) -> Element<'a, Message> {
    widget::container(
        widget::row![
            widget::text(format!("{}", value))
                .size(16)
                .color(color)
                .font(iced::Font::DEFAULT),
            widget::text(label)
                .size(11)
                .color(theme.text_muted),
        ]
        .spacing(6)
        .align_y(Alignment::Center)
    )
    .padding([6, 12])
    .style(move |_t: &Theme| {
        iced::widget::container::Style {
            background: Some(iced::Background::Color(theme.surface)),
            border: iced::Border {
                radius: 6.0.into(),
                width: 1.0,
                color: theme.border,
            },
            ..Default::default()
        }
    })
    .into()
}

/// Table header row
pub fn table_header(theme: &AppTheme) -> Element<'_, Message> {
    widget::container(
        widget::row![
            widget::text("Name").size(12).color(theme.text_muted).width(Length::FillPortion(3)),
            widget::text("Image").size(12).color(theme.text_muted).width(Length::FillPortion(3)),
            widget::text("Status").size(12).color(theme.text_muted).width(Length::FillPortion(2)),
            widget::text("Ports").size(12).color(theme.text_muted).width(Length::FillPortion(2)),
            widget::text("Actions").size(12).color(theme.text_muted).width(Length::Fixed(200.0)),
        ]
        .spacing(12)
        .padding([8, 12])
    )
    .style(move |_t: &Theme| {
        iced::widget::container::Style {
            background: Some(iced::Background::Color(theme.surface)),
            border: iced::Border {
                radius: iced::border::top(6.0),
                width: 0.0,
                color: theme.border,
            },
            ..Default::default()
        }
    })
    .into()
}

/// Container row with sleek design
pub fn container_row<'a>(
    container: &'a ContainerInfo,
    theme: &'a AppTheme,
    on_start: Message,
    on_stop: Message,
    on_restart: Message,
    on_remove: Message,
    on_logs: Message,
) -> Element<'a, Message> {
    let status_color = match container.state {
        ContainerState::Running => theme.success,
        ContainerState::Exited | ContainerState::Dead => theme.error,
        _ => theme.warning,
    };
    
    let status_dot = widget::text("●").color(status_color).size(10);
    
    let name = widget::column![
        widget::text(&container.name).size(13).color(theme.text),
        widget::text(&container.short_id).size(10).color(theme.text_muted),
    ]
    .spacing(2);
    
    let image = widget::column![
        widget::text(truncate(&container.image, 25)).size(12).color(theme.text),
        widget::text(format!("{}", container.created.format("%Y-%m-%d")))
            .size(10)
            .color(theme.text_muted),
    ]
    .spacing(2);
    
    let status_text = widget::column![
        widget::row![status_dot, widget::text(&container.status).size(12).color(theme.text)]
            .spacing(6)
            .align_y(Alignment::Center),
    ];
    
    let ports_text: Element<_> = if container.ports.is_empty() {
        widget::text("-").size(11).color(theme.text_muted).into()
    } else {
        let port_list: Vec<String> = container.ports.iter()
            .filter(|p| p.public_port.is_some())
            .take(2)
            .map(|p| format!("{}:{}", p.ip.as_deref().unwrap_or("0.0.0.0"), p.public_port.unwrap()))
            .collect();
        
        if port_list.is_empty() {
            widget::text("-").size(11).color(theme.text_muted).into()
        } else {
            let text = if container.ports.len() > 2 {
                format!("{}, +{}", port_list.join(", "), container.ports.len() - 2)
            } else {
                port_list.join(", ")
            };
            widget::text(text).size(11).color(theme.text).into()
        }
    };
    
    // Action buttons based on state
    let action_buttons: Element<_> = match container.state {
        ContainerState::Running => {
            widget::row![
                icon_button_small("📋", "Logs", theme).on_press(on_logs),
                icon_button_small("⏸", "Stop", theme).on_press(on_stop),
                icon_button_small("↻", "Restart", theme).on_press(on_restart),
            ]
            .spacing(4)
            .into()
        }
        _ => {
            widget::row![
                icon_button_small("📋", "Logs", theme).on_press(on_logs),
                icon_button_small("▶", "Start", theme).on_press(on_start),
                icon_button_small("🗑", "Remove", theme).on_press(on_remove),
            ]
            .spacing(4)
            .into()
        }
    };
    
    widget::container(
        widget::row![
            widget::container(name).width(Length::FillPortion(3)),
            widget::container(image).width(Length::FillPortion(3)),
            widget::container(status_text).width(Length::FillPortion(2)),
            widget::container(ports_text).width(Length::FillPortion(2)),
            widget::container(action_buttons).width(Length::Fixed(200.0)),
        ]
        .spacing(12)
        .align_y(Alignment::Center)
        .padding([10, 12])
    )
    .style(move |_t: &Theme| {
        iced::widget::container::Style {
            background: Some(iced::Background::Color(theme.background)),
            border: iced::Border {
                radius: 0.0.into(),
                width: 0.0,
                color: theme.border,
            },
            ..Default::default()
        }
    })
    .into()
}

/// Styled button
pub fn styled_button<'a>(label: &'a str, theme: &'a AppTheme) -> widget::Button<'a, Message> {
    let base = theme.primary;
    widget::button(widget::text(label).size(12))
        .padding([8, 16])
        .style(move |_t: &Theme, status| {
            let hover = iced::Color {
                r: (base.r * 1.2).min(1.0),
                g: (base.g * 1.2).min(1.0),
                b: (base.b * 1.2).min(1.0),
                a: base.a,
            };
            let color = match status {
                iced::widget::button::Status::Hovered => hover,
                _ => base,
            };
            iced::widget::button::Style {
                background: Some(iced::Background::Color(color)),
                text_color: iced::Color::WHITE,
                border: iced::Border {
                    radius: 6.0.into(),
                    width: 0.0,
                    color: iced::Color::TRANSPARENT,
                },
                ..Default::default()
            }
        })
}

/// Small icon button for table actions
pub fn icon_button_small<'a>(icon: &'a str, _tooltip: &'a str, theme: &'a AppTheme) -> widget::Button<'a, Message> {
    widget::button(
        widget::text(icon).size(12)
    )
    .width(Length::Fixed(32.0))
    .height(Length::Fixed(28.0))
    .style(move |_t: &Theme, status| {
        let base = theme.surface;
        let hover = theme.surface_hover;
        let color = match status {
            iced::widget::button::Status::Hovered => hover,
            _ => base,
        };
        iced::widget::button::Style {
            background: Some(iced::Background::Color(color)),
            text_color: theme.text,
            border: iced::Border {
                radius: 4.0.into(),
                width: 1.0,
                color: theme.border,
            },
            ..Default::default()
        }
    })
}

/// Icon button with label for toolbar
pub fn icon_button<'a>(icon: &'a str, label: &'a str, _theme: &'a AppTheme) -> widget::Button<'a, Message> {
    widget::button(
        widget::row![
            widget::text(icon).size(14),
            widget::text(label).size(12),
        ]
        .spacing(6)
        .align_y(Alignment::Center)
    )
    .padding([8, 14])
    .style(|_t: &Theme, status| {
        let base = iced::Color::from_rgb8(60, 60, 70);
        let color = match status {
            iced::widget::button::Status::Hovered => iced::Color::from_rgb8(80, 80, 90),
            _ => base,
        };
        iced::widget::button::Style {
            background: Some(iced::Background::Color(color)),
            text_color: iced::Color::WHITE,
            border: iced::Border {
                radius: 6.0.into(),
                width: 0.0,
                color: iced::Color::TRANSPARENT,
            },
            ..Default::default()
        }
    })
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}
