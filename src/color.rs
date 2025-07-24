use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use eyre::{bail, OptionExt, Report, Result};

pub fn smoothstep(a: &[u8; 3], b: &[u8; 3], t: f64) -> [u8; 3] {
    let st = t * t * (3.0 - 2.0 * t);
    (0..3)
        .map(|i| ((1.0 - st) * a[i] as f64 + st * b[i] as f64).round() as u8)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

pub fn hex_to_rgb(hex: &str) -> Option<[u8; 3]> {
    let hex = hex.strip_prefix('#').unwrap_or(hex);

    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

    Some([r, g, b])
}

#[derive(Clone, PartialEq)]
pub struct Wave {
    bias: f32,
    amplitude: f32,
    frequency: f32,
    phase: f32,
}

impl Wave {
    pub fn new(bias: f32, amplitude: f32, frequency: f32, phase: f32) -> Self {
        Self {
            bias,
            amplitude,
            frequency,
            phase,
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct WaveGradient {
    pub red: Wave,
    pub green: Wave,
    pub blue: Wave,
}

impl Display for WaveGradient {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl FromStr for WaveGradient {
    type Err = Report;

    fn from_str(_s: &str) -> Result<Self> {
        todo!()
    }
}

impl From<WaveGradient> for mandelbrot_explorer::WaveGradient {
    fn from(_value: WaveGradient) -> Self {
        todo!()
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Checkpoint {
    pub position: f64,
    pub color: [u8; 3],
}

impl Checkpoint {
    fn _color_rgb(&self) -> String {
        let [r, g, b] = self.color;
        format!("rgb({},{},{})", r, g, b)
    }

    pub fn color_hex(&self) -> String {
        let [r, g, b] = self.color;
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    }
}

impl Display for Checkpoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}{}", self.position, self.color_hex(),)
    }
}

impl FromStr for Checkpoint {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self> {
        let (position, color) = s
            .split_once('#')
            .ok_or_eyre("Failed to parse step gradient checkpoint")?;
        Ok(Self {
            position: position.parse()?,
            color: hex_to_rgb(color).ok_or_eyre("Fialed to parse hex color")?,
        })
    }
}

impl From<Checkpoint> for mandelbrot_explorer::Checkpoint {
    fn from(value: Checkpoint) -> Self {
        Self {
            position: value.position as f32,
            color: [
                value.color[0] as f32 / 255.0,
                value.color[1] as f32 / 255.0,
                value.color[2] as f32 / 255.0,
            ],
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct StepGradient {
    pub checkpoints: Vec<Checkpoint>,
}

impl Default for StepGradient {
    fn default() -> Self {
        Self {
            checkpoints: vec![
                Checkpoint {
                    position: 0.0,
                    color: hex_to_rgb("#ff0000").unwrap(),
                },
                Checkpoint {
                    position: 0.333,
                    color: hex_to_rgb("#00ff00").unwrap(),
                },
                Checkpoint {
                    position: 0.666,
                    color: hex_to_rgb("#0000ff").unwrap(),
                },
            ],
        }
    }
}

impl Display for StepGradient {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.checkpoints
                .iter()
                .map(|checkpoint| checkpoint.to_string())
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

impl FromStr for StepGradient {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self> {
        let checkpoints = s.split(',').map(|s| s.parse()).collect::<Result<Vec<_>>>()?;

        Ok(Self { checkpoints })
    }
}

impl From<StepGradient> for mandelbrot_explorer::StepGradient {
    fn from(value: StepGradient) -> Self {
        Self {
            checkpoints: value
                .checkpoints
                .iter()
                .map(|&Checkpoint { position, color }| mandelbrot_explorer::Checkpoint {
                    position: position as f32,
                    color: [
                        color[0] as f32 / 255.0,
                        color[1] as f32 / 255.0,
                        color[2] as f32 / 255.0,
                    ],
                })
                .collect(),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum Gradient {
    Wave(WaveGradient),
    Step(StepGradient),
}

impl Display for Gradient {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Wave(gradient) => write!(f, "W{gradient}"),
            Self::Step(gradient) => write!(f, "S{gradient}"),
        }
    }
}

impl FromStr for Gradient {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self> {
        let (prefix, gradient) = s.split_at(1);
        Ok(match prefix {
            "W" => Self::Wave(gradient.parse()?),
            "S" => Self::Step(gradient.parse()?),
            _ => bail!("Invalid gradient format"),
        })
    }
}

impl From<WaveGradient> for Gradient {
    fn from(value: WaveGradient) -> Self {
        Self::Wave(value)
    }
}

impl From<StepGradient> for Gradient {
    fn from(value: StepGradient) -> Self {
        Self::Step(value)
    }
}

impl From<Gradient> for mandelbrot_explorer::Gradient {
    fn from(value: Gradient) -> Self {
        match value {
            Gradient::Wave(gradient) => Self::Wave(gradient.into()),
            Gradient::Step(gradient) => Self::Step(gradient.into()),
        }
    }
}
