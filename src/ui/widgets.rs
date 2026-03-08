use iced::{
    widget::{button, container, row, text, Button, Container, Row},
    Alignment, Background, Border, Color, Element, Length, Theme,
};

use crate::docker::{ContainerInfo, ContainerState};
use crate::ui::theme::AppTheme;

pub fn styled_button<'a, Message>(
    label: &'a str,
    theme: &'a AppTheme,
) -> Button<'a, Message, Theme>
where
    Message: std::clone::Clone + 'static,
{
    button(text(label).size(14))
        .padding([8, 16])
        .style(move |_t, _s| theme.button_style())
}

pub fn styled_button_secondary<'a, Message>(
    label: &'a str,
    theme: &'a AppTheme,
) -> Button<'a, Message, Theme>
where
    Message: std::clone::Clone + 'static,
{
    button(text(label).size(14))
        .padding([8, 16])
        .style(move |_t, _s| theme.button_secondary_style())
}

pub fn card<'a, Message>(content: impl Into<Element<'a, Message, Theme>>) -> Container<'a, Message, Theme>
where
    Message: std::clone::Clone + 'a + 'static,
{
    container(content)
        .padding(16)
        .style(|_t| {
            iced::widget::container::Style {
                background: Some(Background::Color(Color::from_rgb8(30, 30, 60))),
                border: Border {
                    radius: 12.0.into(),
                    width: 1.0,
                    color: Color::from_rgb8(124, 58, 237).scale_alpha(0.2),
                },
                ..Default::default()
            }
        })
}

pub fn state_badge<Message>(state: ContainerState, theme: &AppTheme) -> Element<'static, Message, Theme>
where
    Message: std::clone::Clone + 'static,
{
    let (bg_color, text_color, label) = match state {
        ContainerState::Running => (theme.success.scale_alpha(0.2), theme.success, "Running"),
        ContainerState::Paused => (theme.warning.scale_alpha(0.2), theme.warning, "Paused"),
        ContainerState::Restarting => (theme.primary.scale_alpha(0.2), theme.primary, "Restarting"),
        ContainerState::Exited => (theme.error.scale_alpha(0.2), theme.text_muted, "Stopped"),
        ContainerState::Dead => (theme.error.scale_alpha(0.2), theme.error, "Dead"),
        ContainerState::Created => (theme.secondary.scale_alpha(0.2), theme.secondary, "Created"),
        ContainerState::Unknown => (Color::from_rgb8(100, 100, 100), theme.text_muted, "Unknown"),
    };
    
    container(text(label).size(11).color(text_color))
        .padding([4, 10])
        .style(move |_t| {
            iced::widget::container::Style {
                background: Some(Background::Color(bg_color)),
                border: Border {
                    radius: 20.0.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                ..Default::default()
            }
        })
        .into()
}

pub fn header_row<Message>(theme: &AppTheme) -> Row<'static, Message, Theme>
where
    Message: std::clone::Clone + 'static,
{
    row![
        text("Name").size(12).color(theme.text_muted).width(Length::FillPortion(2)),
        text("Image").size(12).color(theme.text_muted).width(Length::FillPortion(2)),
        text("State").size(12).color(theme.text_muted).width(Length::FillPortion(1)),
        text("Status").size(12).color(theme.text_muted).width(Length::FillPortion(2)),
        text("Ports").size(12).color(theme.text_muted).width(Length::FillPortion(2)),
        text("Actions").size(12).color(theme.text_muted).width(Length::Shrink),
    ]
    .spacing(12)
    .padding([8, 16])
    .align_y(Alignment::Center)
}

pub fn container_row<'a, Message>(
    container_info: &'a ContainerInfo,
    theme: &'a AppTheme,
    on_start: Message,
    on_stop: Message,
    on_restart: Message,
    on_remove: Message,
) -> Container<'a, Message, Theme>
where
    Message: std::clone::Clone + 'a + 'static,
{
    let is_running = container_info.state == ContainerState::Running;
    
    let content = row![
        text(&container_info.name)
            .size(13)
            .color(theme.text)
            .width(Length::FillPortion(2)),
        text(&container_info.image)
            .size(12)
            .color(theme.text_muted)
            .width(Length::FillPortion(2)),
        Element::from(container(state_badge::<Message>(container_info.state.clone(), theme))
            .width(Length::FillPortion(1))) ,
        text(&container_info.status)
            .size(12)
            .color(theme.text_muted)
            .width(Length::FillPortion(2)),
        text(format_ports(&container_info.ports))
            .size(11)
            .color(theme.text_muted)
            .width(Length::FillPortion(2)),
        row![
            if is_running {
                styled_button_secondary("Stop", theme).on_press(on_stop)
            } else {
                styled_button("Start", theme).on_press(on_start)
            },
            styled_button_secondary("Restart", theme).on_press(on_restart),
            styled_button_secondary("Remove", theme).on_press(on_remove),
        ]
        .spacing(8)
        .width(Length::Shrink),
    ]
    .spacing(12)
    .padding([12, 16])
    .align_y(Alignment::Center);
    
    container(content)
        .style(move |_t| {
            iced::widget::container::Style {
                background: Some(Background::Color(theme.surface)),
                border: Border {
                    radius: 8.0.into(),
                    width: 1.0,
                    color: Color::TRANSPARENT,
                },
                ..Default::default()
            }
        })
}

fn format_ports(ports: &[crate::docker::PortMapping]) -> String {
    if ports.is_empty() {
        return "-".to_string();
    }
    
    ports.iter()
        .map(|p| {
            if let Some(public) = p.public_port {
                format!("{}:{}", public, p.private_port)
            } else {
                format!("{}", p.private_port)
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn stats_badge<Message>(
    label: String, 
    value: String, 
    background: Color,
    primary: Color,
    text_color: Color,
    text_muted: Color,
) -> Element<'static, Message, Theme>
where
    Message: std::clone::Clone + 'static,
{
    container(
        iced::widget::column![
            text(label.clone()).size(10).color(text_muted),
            text(value).size(14).color(text_color).font(iced::Font::MONOSPACE),
        ]
        .spacing(4)
        .align_x(Alignment::Center)
    )
    .padding([12, 20])
    .style(move |_t| {
        iced::widget::container::Style {
            background: Some(Background::Color(background)),
            border: Border {
                radius: 8.0.into(),
                width: 1.0,
                color: primary.scale_alpha(0.2),
            },
            ..Default::default()
        }
    })
    .into()
}
