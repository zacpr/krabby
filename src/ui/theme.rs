use iced::{
    border::Radius,
    theme::{Custom, Palette},
    Background, Border, Color, Theme,
};

#[derive(Debug, Clone)]
pub struct AppTheme {
    pub name: String,
    pub background: Color,
    pub surface: Color,
    pub surface_hover: Color,
    pub border: Color,
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub text: Color,
    pub text_muted: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub border_radius: f32,
    pub enable_glow: bool,
}

impl AppTheme {
    pub fn midnight() -> Self {
        Self {
            name: "Midnight".to_string(),
            background: Color::from_rgb8(15, 15, 35),      // #0f0f23
            surface: Color::from_rgb8(30, 30, 60),         // #1e1e3c
            surface_hover: Color::from_rgb8(45, 45, 80),   // #2d2d50
            border: Color::from_rgb8(50, 50, 90),          // #32325a
            primary: Color::from_rgb8(124, 58, 237),       // #7c3aed (violet)
            secondary: Color::from_rgb8(99, 102, 241),     // #6366f1 (indigo)
            accent: Color::from_rgb8(139, 92, 246),        // #8b5cf6
            text: Color::from_rgb8(226, 232, 240),         // #e2e8f0
            text_muted: Color::from_rgb8(148, 163, 184),   // #94a3b8
            success: Color::from_rgb8(34, 197, 94),        // #22c55e
            warning: Color::from_rgb8(251, 191, 36),       // #fbbf24
            error: Color::from_rgb8(239, 68, 68),          // #ef4444
            border_radius: 12.0,
            enable_glow: true,
        }
    }
    
    pub fn ocean() -> Self {
        Self {
            name: "Ocean".to_string(),
            background: Color::from_rgb8(12, 74, 110),     // #0c4a6e
            surface: Color::from_rgb8(8, 47, 73),          // #082f49
            surface_hover: Color::from_rgb8(12, 65, 95),   // #0c415f
            border: Color::from_rgb8(20, 80, 110),         // #14506e
            primary: Color::from_rgb8(14, 165, 233),       // #0ea5e9
            secondary: Color::from_rgb8(56, 189, 248),     // #38bdf8
            accent: Color::from_rgb8(125, 211, 252),       // #7dd3fc
            text: Color::from_rgb8(224, 242, 254),         // #e0f2fe
            text_muted: Color::from_rgb8(125, 211, 252),   // #7dd3fc
            success: Color::from_rgb8(52, 211, 153),       // #34d399
            warning: Color::from_rgb8(251, 191, 36),       // #fbbf24
            error: Color::from_rgb8(248, 113, 113),        // #f87171
            border_radius: 12.0,
            enable_glow: true,
        }
    }
    
    pub fn forest() -> Self {
        Self {
            name: "Forest".to_string(),
            background: Color::from_rgb8(6, 78, 59),       // #064e3b
            surface: Color::from_rgb8(5, 46, 22),          // #052e16
            surface_hover: Color::from_rgb8(8, 65, 35),    // #084123
            border: Color::from_rgb8(10, 70, 40),          // #0a4628
            primary: Color::from_rgb8(16, 185, 129),       // #10b981
            secondary: Color::from_rgb8(52, 211, 153),     // #34d399
            accent: Color::from_rgb8(110, 231, 183),       // #6ee7b7
            text: Color::from_rgb8(209, 250, 229),         // #d1fae5
            text_muted: Color::from_rgb8(167, 243, 208),   // #a7f3d0
            success: Color::from_rgb8(34, 197, 94),        // #22c55e
            warning: Color::from_rgb8(250, 204, 21),       // #facc15
            error: Color::from_rgb8(248, 113, 113),        // #f87171
            border_radius: 12.0,
            enable_glow: true,
        }
    }
    
    pub fn rose() -> Self {
        Self {
            name: "Rose".to_string(),
            background: Color::from_rgb8(136, 19, 55),     // #881337
            surface: Color::from_rgb8(74, 4, 26),          // #4a041a
            surface_hover: Color::from_rgb8(110, 15, 45),  // #6e0f2d
            border: Color::from_rgb8(120, 20, 50),         // #781432
            primary: Color::from_rgb8(244, 63, 94),        // #f43f5e
            secondary: Color::from_rgb8(251, 113, 133),    // #fb7185
            accent: Color::from_rgb8(253, 164, 175),       // #fda4af
            text: Color::from_rgb8(255, 228, 230),         // #ffe4e6
            text_muted: Color::from_rgb8(254, 205, 211),   // #fecdd3
            success: Color::from_rgb8(74, 222, 128),       // #4ade80
            warning: Color::from_rgb8(250, 204, 21),       // #facc15
            error: Color::from_rgb8(252, 165, 165),        // #fca5a5
            border_radius: 12.0,
            enable_glow: true,
        }
    }
    
    pub fn amber() -> Self {
        Self {
            name: "Amber".to_string(),
            background: Color::from_rgb8(69, 26, 3),       // #451a03
            surface: Color::from_rgb8(66, 20, 0),          // #421400
            surface_hover: Color::from_rgb8(85, 35, 5),    // #552305
            border: Color::from_rgb8(90, 40, 8),           // #5a2808
            primary: Color::from_rgb8(245, 158, 11),       // #f59e0b
            secondary: Color::from_rgb8(251, 191, 36),     // #fbbf24
            accent: Color::from_rgb8(253, 224, 71),        // #fde047
            text: Color::from_rgb8(254, 243, 199),         // #fef3c7
            text_muted: Color::from_rgb8(253, 230, 138),   // #fde68a
            success: Color::from_rgb8(134, 239, 172),       // #86efac
            warning: Color::from_rgb8(251, 191, 36),       // #fbbf24
            error: Color::from_rgb8(252, 165, 165),        // #fca5a5
            border_radius: 12.0,
            enable_glow: true,
        }
    }
    
    pub fn all_themes() -> Vec<Self> {
        vec![
            Self::midnight(),
            Self::ocean(),
            Self::forest(),
            Self::rose(),
            Self::amber(),
        ]
    }
    
    pub fn container_style(&self) -> iced::widget::container::Style {
        iced::widget::container::Style {
            background: Some(Background::Color(self.surface)),
            border: Border {
                radius: Radius::new(self.border_radius),
                width: 1.0,
                color: self.primary.scale_alpha(0.2),
            },
            ..Default::default()
        }
    }
    
    pub fn button_style(&self) -> iced::widget::button::Style {
        iced::widget::button::Style {
            background: Some(Background::Color(self.primary)),
            border: Border {
                radius: Radius::new(self.border_radius),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            text_color: self.text,
            ..Default::default()
        }
    }
    
    pub fn button_secondary_style(&self) -> iced::widget::button::Style {
        iced::widget::button::Style {
            background: Some(Background::Color(self.surface)),
            border: Border {
                radius: Radius::new(self.border_radius),
                width: 1.0,
                color: self.primary.scale_alpha(0.5),
            },
            text_color: self.text,
            ..Default::default()
        }
    }
    
    pub fn text_input_style(&self) -> iced::widget::text_input::Style {
        iced::widget::text_input::Style {
            background: Background::Color(self.background),
            border: Border {
                radius: Radius::new(self.border_radius),
                width: 1.0,
                color: self.primary.scale_alpha(0.3),
            },
            placeholder: self.text_muted,
            value: self.text,
            selection: self.primary.scale_alpha(0.4),
            icon: self.text,

        }
    }
    
    pub fn scrollable_style(&self) -> iced::widget::scrollable::Style {
        iced::widget::scrollable::Style {
            container: iced::widget::container::Style {
                background: Some(Background::Color(self.background)),
                ..Default::default()
            },
            vertical_rail: iced::widget::scrollable::Rail {
                background: Some(Background::Color(self.surface)),
                border: Border {
                    radius: Radius::new(4.0),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                scroller: iced::widget::scrollable::Scroller {
                    color: self.primary.scale_alpha(0.4),
                    border: Border {
                        radius: Radius::new(4.0),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                },
            },
            horizontal_rail: iced::widget::scrollable::Rail {
                background: Some(Background::Color(self.surface)),
                border: Border {
                    radius: Radius::new(4.0),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                scroller: iced::widget::scrollable::Scroller {
                    color: self.primary.scale_alpha(0.4),
                    border: Border {
                        radius: Radius::new(4.0),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                },
            },
            gap: None,
        }
    }
    
    pub fn main_container_style(&self) -> iced::widget::container::Style {
        iced::widget::container::Style {
            background: Some(Background::Color(self.background)),
            ..Default::default()
        }
    }
    
    pub fn logs_container_style(&self) -> iced::widget::container::Style {
        iced::widget::container::Style {
            background: Some(Background::Color(Color::from_rgb8(20, 20, 25))),
            border: Border {
                radius: Radius::new(8.0),
                width: 1.0,
                color: self.border,
            },
            ..Default::default()
        }
    }
    
    pub fn status_bar_style(&self) -> iced::widget::container::Style {
        iced::widget::container::Style {
            background: Some(Background::Color(self.surface)),
            border: Border {
                radius: Radius::new(6.0),
                width: 1.0,
                color: self.border,
            },
            ..Default::default()
        }
    }
    
    pub fn to_iced_theme(&self) -> Theme {
        let palette = Palette {
            background: self.background,
            text: self.text,
            primary: self.primary,
            success: self.success,
            danger: self.error,
        };
        
        Theme::Custom(std::sync::Arc::new(Custom::new(
            format!("rusty-{}", self.name.to_lowercase()),
            palette,
        )))
    }
}

impl Default for AppTheme {
    fn default() -> Self {
        Self::midnight()
    }
}
