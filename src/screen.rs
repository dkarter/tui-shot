use regex::Regex;
use std::sync::LazyLock;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

// The input is a serialized viewport, not a terminal instruction stream.
// Read SGR styling; discard other controls, including OSC hyperlinks.
static CONTROL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"(?:\x1b\]|\x9d)[^\x07\x1b\x9c]*(?:\x07|\x1b\\|\x9c|$)",
        r"|(?:\x1b[PX^_]|[\x90\x98\x9e\x9f])[^\x1b\x9c]*(?:\x1b\\|\x9c|$)",
        r"|(?:\x1b\[|\x9b)([0-?]*)([ -/]*)([@-~])",
        r"|\x1b[ -/]*[0-~]|[\x00-\x08\x0b-\x1f\x7f-\x9f\ufffe\uffff]"
    ))
    .expect("valid control-sequence regex")
});

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    DefaultFg,
    DefaultBg,
    Indexed(u8),
    Rgb(u8, u8, u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Style {
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub faint: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
    pub inverse: bool,
    pub hidden: bool,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            fg: Color::DefaultFg,
            bg: Color::DefaultBg,
            bold: false,
            faint: false,
            italic: false,
            underline: false,
            strike: false,
            inverse: false,
            hidden: false,
        }
    }
}

impl Style {
    pub fn colors(self) -> (Color, Color) {
        if self.inverse {
            (self.bg, self.fg)
        } else {
            (self.fg, self.bg)
        }
    }

    pub fn decoration(self) -> &'static str {
        match (self.underline, self.strike) {
            (true, true) => "underline line-through",
            (true, false) => "underline",
            (false, true) => "line-through",
            (false, false) => "",
        }
    }

    fn sgr(&mut self, params: &str) {
        let mut fields = params.split(';');
        while let Some(field) = fields.next() {
            let mut parts = field.split(':');
            let code = match parts.next().unwrap_or_default() {
                "" => 0,
                n => match n.parse::<u8>() {
                    Ok(n) => n,
                    Err(_) => continue,
                },
            };
            match code {
                0 => *self = Self::default(),
                1 => self.bold = true,
                2 => self.faint = true,
                3 => self.italic = true,
                4 => self.underline = parts.next() != Some("0"),
                7 => self.inverse = true,
                8 => self.hidden = true,
                9 => self.strike = true,
                22 => {
                    self.bold = false;
                    self.faint = false;
                }
                23 => self.italic = false,
                24 => self.underline = false,
                27 => self.inverse = false,
                28 => self.hidden = false,
                29 => self.strike = false,
                30..=37 => self.fg = Color::Indexed(code - 30),
                40..=47 => self.bg = Color::Indexed(code - 40),
                90..=97 => self.fg = Color::Indexed(code - 90 + 8),
                100..=107 => self.bg = Color::Indexed(code - 100 + 8),
                38 | 48 | 58 => {
                    let color = if field.contains(':') {
                        extended(&mut parts, true)
                    } else {
                        extended(&mut fields, false)
                    };
                    if let Some(color) = color {
                        match code {
                            38 => self.fg = color,
                            48 => self.bg = color,
                            _ => {}
                        }
                    }
                }
                39 => self.fg = Color::DefaultFg,
                49 => self.bg = Color::DefaultBg,
                _ => {}
            }
        }
    }
}

fn extended<'a>(parts: &mut impl Iterator<Item = &'a str>, colon: bool) -> Option<Color> {
    match parts.next()? {
        "5" => {
            let n: u8 = parts.next()?.parse().ok()?;
            let levels = [0, 95, 135, 175, 215, 255];
            Some(match n {
                0..=15 => Color::Indexed(n),
                232..=255 => {
                    let v = 8 + 10 * (n - 232);
                    Color::Rgb(v, v, v)
                }
                _ => {
                    let n = usize::from(n - 16);
                    Color::Rgb(levels[n / 36], levels[n / 6 % 6], levels[n % 6])
                }
            })
        }
        "2" => {
            // Colon form permits a reserved color-space field before RGB.
            let mut rgb = ["", "", ""];
            for c in &mut rgb {
                *c = parts.next()?;
            }
            if colon && let Some(blue) = parts.next() {
                rgb = [rgb[1], rgb[2], blue];
            }
            let [r, g, b] = rgb.map(str::parse::<u8>);
            Some(Color::Rgb(r.ok()?, g.ok()?, b.ok()?))
        }
        _ => None,
    }
}

pub type Screen = Vec<Vec<Run>>;

pub struct Run {
    pub style: Style,
    pub text: String,
    pub start: usize,
    pub width: usize,
}

pub fn parse(input: &str) -> Screen {
    let input = input.replace("\r\n", "\n");
    let mut rows = vec![Vec::<Run>::new()];
    let mut style = Style::default();
    let mut end = 0;
    let mut push = |text: &str, style: Style| {
        for (i, line) in text.split('\n').enumerate() {
            if i != 0 {
                rows.push(Vec::new());
            }
            if line.is_empty() {
                continue;
            }
            let row = rows.last_mut().expect("at least one row");
            if let Some(last) = row.last_mut().filter(|r| r.style == style) {
                last.text.push_str(line);
            } else {
                row.push(Run {
                    style,
                    text: line.to_owned(),
                    start: 0,
                    width: 0,
                });
            }
        }
    };
    for capture in CONTROL.captures_iter(&input) {
        let all = capture.get(0).expect("whole match");
        push(&input[end..all.start()], style);
        if let (Some(params), Some(space), Some(command)) =
            (capture.get(1), capture.get(2), capture.get(3))
            && space.is_empty()
            && command.as_str() == "m"
            && params
                .as_str()
                .bytes()
                .all(|c| c.is_ascii_digit() || matches!(c, b':' | b';'))
        {
            style.sgr(params.as_str());
        }
        end = all.end();
    }
    push(&input[end..], style);
    if rows.len() > 1 && rows.last().is_some_and(Vec::is_empty) {
        rows.pop();
    }

    for row in &mut rows {
        let mut col = 0;
        for run in row {
            run.start = col;
            let mut text = String::new();
            for glyph in run.text.graphemes(true) {
                if glyph == "\t" {
                    let width = 8 - col % 8;
                    text.push_str(&" ".repeat(width));
                    col += width;
                } else {
                    text.push_str(glyph);
                    col += glyph.width();
                }
            }
            run.width = col - run.start;
            run.text = if run.style.hidden {
                " ".repeat(run.width)
            } else {
                text
            };
        }
    }
    rows
}

pub fn clean_title(title: &str) -> String {
    CONTROL
        .replace_all(title, "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_viewport_and_discards_controls() {
        let rows = parse(
            "\x1b[1;36m┌界e\u{301}🇺🇸┐\x1b[0m\r\n\
             a\x1b[8m\tprivate\x1b[0m\x1b]8;;https://example.invalid\x1b\\<&>\x1b]8;;\x07\n\
             \x1b[48;5;232m  \x1b[38:2::250:150:70mX\x1b[0m\n",
        );
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0][0].width, 7);
        assert_eq!(rows[0][0].style.fg, Color::Indexed(6));
        assert!(rows[0][0].style.bold);
        assert_eq!(rows[1][1].text, " ".repeat(14));
        assert_eq!(rows[1][2].start, 15);
        assert_eq!(rows[1][2].text, "<&>");
        assert_eq!(rows[2][0].style.bg, Color::Rgb(8, 8, 8));
        assert_eq!(rows[2][1].style.fg, Color::Rgb(250, 150, 70));
    }
}
