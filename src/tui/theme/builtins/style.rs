use anyhow::{Context, Result, bail};
use ratatui::style::{Color, Modifier, Style};
use serde::Deserialize;
use std::borrow::Cow;

#[derive(Debug, Deserialize)]
pub(super) struct StyleConfig {
    #[serde(default)]
    fg: Option<String>,
    #[serde(default)]
    bg: Option<String>,
    #[serde(default)]
    modifiers: Vec<String>,
}

impl StyleConfig {
    pub(super) fn to_style(&self, context: &str) -> Result<Style> {
        let mut style = Style::new();

        if let Some(fg) = &self.fg {
            let color = parse_color(fg)
                .with_context(|| format!("{context}: invalid foreground colour `{fg}`"))?;
            style = style.fg(color);
        }

        if let Some(bg) = &self.bg {
            let color = parse_color(bg)
                .with_context(|| format!("{context}: invalid background colour `{bg}`"))?;
            style = style.bg(color);
        }

        for modifier in &self.modifiers {
            let modifier_value = parse_modifier(modifier)
                .with_context(|| format!("{context}: invalid modifier `{modifier}`"))?;
            style = style.add_modifier(modifier_value);
        }

        Ok(style)
    }
}

fn parse_color(input: &str) -> Result<Color> {
    let value = input.trim();

    if let Some(hex) = value.strip_prefix('#') {
        return parse_hex_colour(hex);
    }

    if let Some(body) = value.strip_prefix("rgb(").and_then(|s| s.strip_suffix(')')) {
        return parse_rgb_triplet(body);
    }

    if let Some(body) = value
        .strip_prefix("ansi(")
        .and_then(|s| s.strip_suffix(')'))
    {
        let index: u8 = body
            .trim()
            .parse()
            .with_context(|| format!("invalid ANSI colour index `{body}`"))?;
        return Ok(Color::Indexed(index));
    }

    if let Ok(index) = value.parse::<u8>() {
        return Ok(Color::Indexed(index));
    }

    match normalise_key(value).as_ref() {
        "reset" | "none" | "default" => Ok(Color::Reset),
        "black" => Ok(Color::Black),
        "red" => Ok(Color::Red),
        "green" => Ok(Color::Green),
        "yellow" => Ok(Color::Yellow),
        "blue" => Ok(Color::Blue),
        "magenta" => Ok(Color::Magenta),
        "cyan" => Ok(Color::Cyan),
        "gray" | "grey" => Ok(Color::Gray),
        "dark_gray" | "dark_grey" => Ok(Color::DarkGray),
        "light_red" => Ok(Color::LightRed),
        "light_green" => Ok(Color::LightGreen),
        "light_yellow" => Ok(Color::LightYellow),
        "light_blue" => Ok(Color::LightBlue),
        "light_magenta" => Ok(Color::LightMagenta),
        "light_cyan" => Ok(Color::LightCyan),
        "white" => Ok(Color::White),
        other => bail!("unknown colour `{other}`"),
    }
}

fn parse_hex_colour(hex: &str) -> Result<Color> {
    let expanded = match hex.len() {
        3 => {
            let mut expanded = String::with_capacity(6);
            for ch in hex.chars() {
                expanded.push(ch);
                expanded.push(ch);
            }
            Cow::Owned(expanded)
        }
        6 => Cow::Borrowed(hex),
        _ => bail!("hex colours must be 3 or 6 characters long"),
    };

    let r = u8::from_str_radix(&expanded[0..2], 16)
        .with_context(|| format!("invalid red component `{hex}`"))?;
    let g = u8::from_str_radix(&expanded[2..4], 16)
        .with_context(|| format!("invalid green component `{hex}`"))?;
    let b = u8::from_str_radix(&expanded[4..6], 16)
        .with_context(|| format!("invalid blue component `{hex}`"))?;

    Ok(Color::Rgb(r, g, b))
}

fn parse_rgb_triplet(body: &str) -> Result<Color> {
    let components = body.split(',').map(|part| part.trim()).collect::<Vec<_>>();

    if components.len() != 3 {
        bail!(
            "expected three components for rgb() colour, found {}",
            components.len()
        );
    }

    let r = parse_rgb_component(components[0], 'r')?;
    let g = parse_rgb_component(components[1], 'g')?;
    let b = parse_rgb_component(components[2], 'b')?;

    Ok(Color::Rgb(r, g, b))
}

fn parse_rgb_component(value: &str, component: char) -> Result<u8> {
    value.parse::<u8>().with_context(|| {
        format!("invalid {component}-component `{value}` in rgb() colour specification")
    })
}

fn parse_modifier(input: &str) -> Result<Modifier> {
    match normalise_key(input).as_ref() {
        "bold" => Ok(Modifier::BOLD),
        "dim" => Ok(Modifier::DIM),
        "italic" => Ok(Modifier::ITALIC),
        "underline" | "underlined" => Ok(Modifier::UNDERLINED),
        "slow_blink" | "slowblink" => Ok(Modifier::SLOW_BLINK),
        "rapid_blink" | "rapidblink" | "fast_blink" => Ok(Modifier::RAPID_BLINK),
        "reversed" | "reverse" | "invert" | "inverted" => Ok(Modifier::REVERSED),
        "hidden" => Ok(Modifier::HIDDEN),
        "crossed_out" | "crossedout" | "strikethrough" => Ok(Modifier::CROSSED_OUT),
        other => bail!("unknown modifier `{other}`"),
    }
}

fn normalise_key(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .chars()
        .map(|ch| match ch {
            '-' | ' ' => '_',
            other => other,
        })
        .collect()
}
