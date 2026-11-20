use crate::screen::{Color, Screen};
use resvg::{tiny_skia, usvg};
use std::error::Error;
use std::fmt::Write;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

const DEFAULT_FONTS: &str = "Menlo, 'DejaVu Sans Mono', 'Liberation Mono', Consolas, monospace";

#[derive(Clone, Copy, Default)]
pub struct Font<'a>(Option<&'a str>);

impl<'a> Font<'a> {
    pub fn new(family: Option<&'a str>) -> Result<Self, &'static str> {
        if let Some(name) = family.map(str::trim) {
            if name.is_empty()
                || !name
                    .chars()
                    .all(|c| c.is_alphanumeric() || " -_.".contains(c))
            {
                return Err("pass a single font family name (such as 'Agent Mono Nerd Font')");
            }
            return Ok(Self(Some(name)));
        }
        Ok(Self(None))
    }

    fn stack(self) -> String {
        match self.0 {
            Some(name) => format!("'{name}', {DEFAULT_FONTS}"),
            None => DEFAULT_FONTS.into(),
        }
    }
}
const DARK: [&str; 19] = [
    "#171923", "#252938", "#e5e9f0", "#34394b", "#f78c9b", "#a4d893", "#f4cd81", "#8eb6ff",
    "#d7a9ff", "#83dcca", "#d0d6e5", "#818aa4", "#ffadb8", "#bdeba8", "#ffe0a3", "#aecaff",
    "#e4c5ff", "#a8eadf", "#ffffff",
];
const LIGHT: [&str; 19] = [
    "#ffffff", "#edf0f7", "#24283b", "#24283b", "#a42643", "#286339", "#805500", "#264da0",
    "#7738a0", "#126c67", "#565d72", "#626b80", "#ae3751", "#337340", "#8a5800", "#345eb5",
    "#8545ad", "#14766f", "#555f7b",
];

#[derive(Clone, Copy, usage_rs::ValueEnum)]
pub enum Theme {
    Dark,
    Light,
}

impl Theme {
    fn name(self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
        }
    }

    fn palette(self) -> &'static [&'static str; 19] {
        match self {
            Self::Dark => &DARK,
            Self::Light => &LIGHT,
        }
    }

    fn color(self, color: Color) -> String {
        let palette = self.palette();
        match color {
            Color::DefaultBg => palette[0].into(),
            Color::DefaultFg => palette[2].into(),
            Color::Indexed(i) => palette[3 + usize::from(i)].into(),
            Color::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
        }
    }
}

fn css_color(color: Color) -> String {
    match color {
        Color::DefaultBg => "var(--tui-bg)".into(),
        Color::DefaultFg => "var(--tui-fg)".into(),
        Color::Indexed(i) => format!("var(--tui-c{i})"),
        Color::Rgb(r, g, b) => {
            format!("var(--tui-rgb-{r:02x}{g:02x}{b:02x},#{r:02x}{g:02x}{b:02x})")
        }
    }
}

fn escape(text: &str, out: &mut String) {
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
}

pub fn text(screen: &Screen) -> String {
    let mut out = String::new();
    for row in screen {
        for run in row {
            out.push_str(&run.text);
        }
        out.push('\n');
    }
    out
}

pub fn html(screen: &Screen, theme: Theme, title: &str, fragment: bool, font: Font<'_>) -> String {
    let font_stack = font.stack();
    let mut out = String::new();
    if !fragment {
        out.push_str(
            "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n\
            <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<title>",
        );
        escape(
            if title.is_empty() {
                "Terminal screenshot"
            } else {
                title
            },
            &mut out,
        );
        out.push_str("</title>\n");
    }
    out.push_str("<style>\n");
    for theme in [Theme::Dark, Theme::Light] {
        let p = theme.palette();
        let border = match theme {
            Theme::Dark => "#3c4356",
            Theme::Light => "#c5ccda",
        };
        write!(
            out,
            ":where(.tui-shot[data-theme=\"{}\"]) {{ --tui-bg: {}; --tui-bar: {}; --tui-fg: {}; --tui-border: {border}; ",
            theme.name(),
            p[0],
            p[1],
            p[2]
        )
        .unwrap();
        for (n, color) in p[3..].iter().enumerate() {
            write!(out, "--tui-c{n}: {color}; ").unwrap();
        }
        out.push_str("}\n");
    }
    write!(
        out,
        r#".tui-shot {{
  box-sizing: border-box; width: fit-content; max-width: 100%;
  margin: 0; border-radius: 12px; overflow: hidden;
  border: 1px solid var(--tui-border);
  color: var(--tui-fg); background: var(--tui-bg);
}}
.tui-shot figcaption {{
  padding: 10px 20px; background: var(--tui-bar);
  border-bottom: 1px solid var(--tui-border);
  overflow-wrap: anywhere;
  font: 600 13px/20px system-ui, sans-serif;
}}
.tui-shot pre {{
  margin: 0; padding: 20px; overflow: auto; white-space: pre;
  font: 14px/1.35 var(--tui-font, var(--tui-capture-font));
  font-variant-ligatures: none;
  color: inherit; background: transparent;
}}
.tui-shot code {{ font: inherit; color: inherit; background: transparent; }}
</style>
"#
    )
    .unwrap();
    if !fragment {
        out.push_str("</head>\n<body>\n");
    }
    writeln!(
        out,
        "<figure class=\"tui-shot\" data-theme=\"{}\" style=\"--tui-capture-font: {font_stack}\">",
        theme.name()
    )
    .unwrap();
    if !title.is_empty() {
        out.push_str("<figcaption>");
        escape(title, &mut out);
        out.push_str("</figcaption>\n");
    }
    out.push_str("<pre><code>");
    for (y, row) in screen.iter().enumerate() {
        if y != 0 {
            out.push('\n');
        }
        for run in row {
            let s = run.style;
            let (fg, bg) = s.colors();
            let mut css = String::new();
            if fg != Color::DefaultFg {
                write!(css, "color:{};", css_color(fg)).unwrap();
            }
            if s.faint {
                write!(
                    css,
                    "color:color-mix(in srgb,{} 60%,transparent);",
                    css_color(fg)
                )
                .unwrap();
            }
            if bg != Color::DefaultBg {
                write!(css, "background-color:{};", css_color(bg)).unwrap();
            }
            if s.bold {
                css.push_str("font-weight:bold;");
            }
            if s.italic {
                css.push_str("font-style:italic;");
            }
            if !s.decoration().is_empty() {
                write!(css, "text-decoration:{};", s.decoration()).unwrap();
            }
            if !css.is_empty() {
                write!(out, "<span style=\"{css}\">").unwrap();
            }
            escape(&run.text, &mut out);
            if !css.is_empty() {
                out.push_str("</span>");
            }
        }
    }
    out.push_str("</code></pre>\n</figure>\n");
    if !fragment {
        out.push_str("</body>\n</html>\n");
    }
    out
}

fn block_rect(glyph: &str) -> Option<(f64, f64, f64, f64)> {
    let mut chars = glyph.chars();
    let ch = chars.next()?;
    if chars.next().is_some() {
        return None;
    }
    Some(match ch {
        '▀' => (0., 0., 1., 0.5),
        '▁'..='█' => {
            let h = f64::from(ch as u32 - '▁' as u32 + 1) / 8.;
            (0., 1. - h, 1., h)
        }
        '▉'..='▏' => (0., 0., f64::from('▐' as u32 - ch as u32) / 8., 1.),
        '▐' => (0.5, 0., 0.5, 1.),
        _ => return None,
    })
}

pub fn svg(screen: &Screen, theme: Theme, title: &str, font: Font<'_>) -> String {
    const CELL: f64 = 8.43;
    const ROW: f64 = 18.;
    const PAD: f64 = 20.;
    let columns = screen
        .iter()
        .filter_map(|row| row.last().map(|run| run.start + run.width))
        .max()
        .unwrap_or(1)
        .max(title.width());
    let bar = if title.is_empty() { 0. } else { 36. };
    let width = (columns as f64 * CELL + 2. * PAD + 1.).floor();
    let height = screen.len() as f64 * ROW + 2. * PAD + bar;
    let [bg, title_bg, fg, ..] = *theme.palette();
    let font_stack = font.stack();
    let mut out = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" \
        height=\"{height}\" viewBox=\"0 0 {width} {height}\" role=\"img\">\n<title>"
    );
    escape(
        if title.is_empty() {
            "Terminal screenshot"
        } else {
            title
        },
        &mut out,
    );
    write!(
        out,
        "</title>\n<clipPath id=\"card\"><rect width=\"{width}\" height=\"{height}\" \
        rx=\"12\"/></clipPath>\n<g clip-path=\"url(#card)\">\n<rect width=\"{width}\" \
        height=\"{height}\" fill=\"{bg}\"/>\n"
    )
    .unwrap();
    if !title.is_empty() {
        write!(
            out,
            "<rect width=\"{width}\" height=\"{bar}\" fill=\"{title_bg}\"/>\n\
            <text x=\"{PAD}\" y=\"23\" fill=\"{fg}\" font-family=\"{font_stack}\" \
            font-size=\"13\" font-weight=\"bold\" xml:space=\"preserve\">"
        )
        .unwrap();
        escape(title, &mut out);
        out.push_str("</text>\n");
    }
    writeln!(
        out,
        "<g font-family=\"{font_stack}\" font-size=\"14\" xml:space=\"preserve\">"
    )
    .unwrap();
    for (y, row) in screen.iter().enumerate() {
        let y = PAD + bar + y as f64 * ROW;
        // Paint backgrounds before any text, so adjacent cells don't clip font overhangs.
        for run in row {
            let (_, bg) = run.style.colors();
            if bg != Color::DefaultBg {
                writeln!(
                    out,
                    "<rect x=\"{:.2}\" y=\"{y}\" width=\"{:.2}\" height=\"{ROW}\" fill=\"{}\"/>",
                    PAD + run.start as f64 * CELL,
                    run.width as f64 * CELL,
                    theme.color(bg)
                )
                .unwrap();
            }
        }
        for run in row {
            let s = run.style;
            if s.hidden || (run.text.trim().is_empty() && s.decoration().is_empty()) {
                continue;
            }
            let fill = theme.color(s.colors().0);
            let opacity = if s.faint { " opacity=\"0.6\"" } else { "" };
            let mut attrs = format!("fill=\"{fill}\"{opacity}");
            if s.bold {
                attrs.push_str(" font-weight=\"bold\"");
            }
            if s.italic {
                attrs.push_str(" font-style=\"italic\"");
            }
            if !s.decoration().is_empty() {
                write!(attrs, " text-decoration=\"{}\"", s.decoration()).unwrap();
            }
            let mut x = PAD + run.start as f64 * CELL;
            let baseline = y + 13.;
            if run.text.is_ascii() {
                out.push_str("<text x=\"");
                for i in 0..run.text.len() {
                    if i != 0 {
                        out.push(' ');
                    }
                    write!(out, "{:.2}", x + i as f64 * CELL).unwrap();
                }
                write!(out, "\" y=\"{baseline}\" {attrs}>").unwrap();
                escape(&run.text, &mut out);
                out.push_str("</text>\n");
                continue;
            }
            let mut spans = String::new();
            for glyph in run.text.graphemes(true) {
                if let Some((dx, dy, w, h)) =
                    block_rect(glyph).filter(|_| s.decoration().is_empty())
                {
                    writeln!(out, "<rect x=\"{:.2}\" y=\"{}\" width=\"{:.2}\" height=\"{}\" fill=\"{fill}\"{opacity}/>",
                        x + dx * CELL, y + dy * ROW, w * CELL, h * ROW).unwrap();
                } else if !glyph.trim().is_empty() || !s.decoration().is_empty() {
                    write!(spans, "<tspan x=\"{x:.2}\">").unwrap();
                    escape(glyph, &mut spans);
                    spans.push_str("</tspan>");
                }
                x += glyph.width() as f64 * CELL;
            }
            if !spans.is_empty() {
                writeln!(out, "<text y=\"{baseline}\" {attrs}>{spans}</text>").unwrap();
            }
        }
    }
    out.push_str("</g></g></svg>\n");
    out
}

pub fn png(
    screen: &Screen,
    theme: Theme,
    title: &str,
    font: Font<'_>,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut options = usvg::Options::default();
    let fonts = options.fontdb_mut();
    fonts.load_system_fonts();
    fonts.set_monospace_family("DejaVu Sans Mono");
    if fonts.faces().next().is_none() {
        return Err("no fonts found; install a monospace font such as DejaVu Sans Mono".into());
    }
    let font = if let Some(requested) = font.0 {
        let name = fonts
            .faces()
            .flat_map(|face| face.families.iter())
            .map(|(name, _)| name.as_str())
            .find(|name| name.eq_ignore_ascii_case(requested))
            .ok_or_else(|| {
                format!("font family '{requested}' not found; install it or omit --font")
            })?;
        Font::new(Some(name))?
    } else {
        font
    };
    let svg = svg(screen, theme, title, font);
    let tree = usvg::Tree::from_str(&svg, &options)?;
    let size = tree.size().to_int_size();
    let (width, height) = (size.width() * 2, size.height() * 2);
    if u64::from(width) * u64::from(height) > 32_000_000 {
        return Err("image is too large to rasterize; export SVG instead".into());
    }
    let mut pixmap = tiny_skia::Pixmap::new(width, height).ok_or("invalid image size")?;
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(2., 2.),
        &mut pixmap.as_mut(),
    );
    Ok(pixmap.encode_png()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screen::parse;

    #[test]
    fn html_keeps_text_selectable_and_escaped() {
        let screen = parse("\x1b[36m<>&\x1b[0m\n\x1b[7m x \x1b[0m");
        let page = html(&screen, Theme::Light, "<demo>", true, Font::default());
        assert!(!page.contains("<!doctype"));
        assert!(page.contains("data-theme=\"light\""));
        assert!(page.contains("<figcaption>&lt;demo&gt;</figcaption>"));
        assert!(page.contains("var(--tui-c6);\">&lt;&gt;&amp;</span>\n"));
        assert!(page.contains("background-color:var(--tui-fg);\"> x </span>"));
        assert_eq!(text(&screen), "<>&\n x \n");
    }

    #[test]
    fn png_uses_two_pixels_per_svg_unit() {
        let bytes = png(&parse("\x1b[32m█\x1b[0m"), Theme::Dark, "", Font::default()).unwrap();
        assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
        assert_eq!(u32::from_be_bytes(bytes[16..20].try_into().unwrap()), 98);
        assert_eq!(u32::from_be_bytes(bytes[20..24].try_into().unwrap()), 116);
    }

    #[test]
    fn custom_font_is_first_in_the_stack() {
        let screen = parse("󰢱");
        let font = Font::new(Some("Agent Mono Nerd Font")).unwrap();
        assert!(
            svg(&screen, Theme::Dark, "", font)
                .contains("font-family=\"'Agent Mono Nerd Font', Menlo")
        );
        assert!(
            html(&screen, Theme::Dark, "", true, font)
                .contains("style=\"--tui-capture-font: 'Agent Mono Nerd Font', Menlo")
        );
        assert!(Font::new(Some("bad';font-size:999px")).is_err());
    }
}
