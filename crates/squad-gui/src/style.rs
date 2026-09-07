use iced::widget::{button, container};
use iced::{Background, Border, Color, Shadow, Theme};

pub mod color {
    use iced::Color;

    pub const WINDOW: Color = Color::from_rgb(0.039, 0.043, 0.051);
    pub const PANEL: Color = Color::from_rgb(0.075, 0.079, 0.086);
    pub const CARD: Color = Color::from_rgb(0.110, 0.114, 0.122);
    pub const CARD_HI: Color = Color::from_rgb(0.149, 0.157, 0.169);
    pub const BORDER: Color = Color::from_rgb(0.192, 0.200, 0.212);

    pub const TEXT_HI: Color = Color::from_rgb(0.906, 0.910, 0.918);
    pub const TEXT_MID: Color = Color::from_rgb(0.675, 0.682, 0.690);
    pub const TEXT_MUTED: Color = Color::from_rgb(0.471, 0.478, 0.494);

    pub const ACCENT: Color = Color::from_rgb(0.949, 0.557, 0.259);
    pub const ACCENT_HI: Color = Color::from_rgb(1.000, 0.624, 0.298);
    pub const ACCENT_DIM: Color = Color::from_rgb(0.267, 0.145, 0.047);
    pub const ACCENT_TEXT: Color = Color::from_rgb(0.098, 0.059, 0.035);

    pub const DANGER: Color = Color::from_rgb(0.875, 0.373, 0.435);

    pub const SUCCESS: Color = Color::from_rgb(0.427, 0.729, 0.439);
}

pub fn theme() -> Theme {
    Theme::custom(
        "Squad Editor".to_string(),
        iced::theme::Palette {
            background: color::WINDOW,
            text: color::TEXT_HI,
            primary: color::ACCENT,
            success: color::SUCCESS,
            warning: color::ACCENT_HI,
            danger: color::DANGER,
        },
    )
}

fn radius(r: f32) -> Border {
    Border {
        color: Color::TRANSPARENT,
        width: 0.0,
        radius: r.into(),
    }
}

pub fn button_primary(_theme: &Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => color::ACCENT_HI,
        button::Status::Pressed => color::ACCENT,
        button::Status::Disabled => color::CARD_HI,
        button::Status::Active => color::ACCENT,
    };
    let text_color = if status == button::Status::Disabled {
        color::TEXT_MUTED
    } else {
        color::ACCENT_TEXT
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color,
        border: radius(8.0),
        shadow: Shadow::default(),
        snap: false,
    }
}

pub fn button_secondary(_theme: &Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => color::CARD_HI,
        button::Status::Disabled => color::PANEL,
        _ => color::CARD,
    };
    let text_color = if status == button::Status::Disabled {
        color::TEXT_MUTED
    } else {
        color::TEXT_HI
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color,
        border: Border {
            color: color::BORDER,
            width: 1.0,
            radius: 8.0.into(),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

pub fn button_ghost(_theme: &Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Some(Background::Color(color::CARD)),
        _ => None,
    };
    let text_color = if status == button::Status::Disabled {
        color::TEXT_MUTED
    } else {
        color::TEXT_MID
    };
    button::Style {
        background: bg,
        text_color,
        border: radius(8.0),
        shadow: Shadow::default(),
        snap: false,
    }
}

pub fn nav_pill(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        if active {
            button::Style {
                background: Some(Background::Color(color::ACCENT)),
                text_color: color::ACCENT_TEXT,
                border: radius(8.0),
                shadow: Shadow::default(),
                snap: false,
            }
        } else {
            let bg = if status == button::Status::Hovered {
                color::CARD_HI
            } else {
                color::CARD
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: color::TEXT_MID,
                border: radius(8.0),
                shadow: Shadow::default(),
                snap: false,
            }
        }
    }
}

pub fn list_row(selected: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let bg = if selected {
            color::ACCENT_DIM
        } else if status == button::Status::Hovered {
            color::CARD_HI
        } else {
            Color::TRANSPARENT
        };
        let text_color = if selected {
            color::ACCENT_HI
        } else {
            color::TEXT_HI
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color,
            border: radius(8.0),
            shadow: Shadow::default(),
            snap: false,
        }
    }
}

pub fn chip(_theme: &Theme, status: button::Status) -> button::Style {
    let bg = if status == button::Status::Hovered {
        color::CARD_HI
    } else {
        color::CARD
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: color::TEXT_HI,
        border: Border {
            color: color::BORDER,
            width: 1.0,
            radius: 999.0.into(),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

pub fn window(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color::WINDOW)),
        text_color: Some(color::TEXT_HI),
        ..Default::default()
    }
}

pub fn panel(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color::PANEL)),
        text_color: Some(color::TEXT_HI),
        border: Border {
            color: color::BORDER,
            width: 1.0,
            radius: 12.0.into(),
        },
        ..Default::default()
    }
}

pub fn card(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color::CARD)),
        text_color: Some(color::TEXT_HI),
        border: Border {
            color: color::BORDER,
            width: 1.0,
            radius: 10.0.into(),
        },
        ..Default::default()
    }
}

pub fn header_bar(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color::PANEL)),
        text_color: Some(color::TEXT_HI),
        border: Border {
            color: color::BORDER,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

pub fn status_bar(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color::PANEL)),
        text_color: Some(color::TEXT_MID),
        border: Border {
            color: color::BORDER,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

pub fn ovr_badge(ovr: i64) -> impl Fn(&Theme) -> container::Style {
    move |_theme| {
        let (bg, fg) = if ovr >= 85 {
            (color::SUCCESS, color::WINDOW)
        } else if ovr >= 75 {
            (color::ACCENT, color::ACCENT_TEXT)
        } else {
            (color::CARD_HI, color::TEXT_MID)
        };
        container::Style {
            background: Some(Background::Color(bg)),
            text_color: Some(fg),
            border: radius(6.0),
            ..Default::default()
        }
    }
}
