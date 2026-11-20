# tui-shot

**Your TUI deserves better than a cropped desktop screenshot.**

Capture any live [Herdr](https://herdr.dev/) pane without switching to it. Drop
a crisp 2× PNG into your README, publish an SVG, or share a terminal card with
text readers can select, copy, and recolor. Agents and scripts can capture the
same pane with one command. No browser or separate renderer required.

![The tui-shot CLI showing capture flags, themes, and output formats](https://github.com/user-attachments/assets/abdd34e0-3ed1-4f1a-ab52-14759fa38812)

## Install

**With Mise** · macOS and Linux, ARM64 and x64:

```sh
mise use -g packslip:github.com/dkarter/tui-shot@0.1.0
```

**Without Mise**, use [Packslip](https://packslip.dev/docs/install/) directly:

```sh
packslip install github.com/dkarter/tui-shot
```

Prefer a manual download? The [GitHub releases](https://github.com/dkarter/tui-shot/releases)
include attested binaries and Bash, Fish, and Zsh completions.

Herdr is only needed for live capture. On a minimal Linux system, install a
monospace font such as DejaVu Sans Mono for PNG rendering.

## Take your first shot

From inside the pane you want to capture:

```sh
tui-shot --title "My TUI" -o screenshot.png
```

To capture a different pane, find its ID with `herdr pane list` and pass it
with `--pane`:

```sh
tui-shot --pane w1:p2 --theme light -o screenshot.html
```

Pick the format with the file extension: `.png`, `.svg`, `.html`, `.txt`, or
`.ansi`. Save `.ansi` if you want to render the exact same frame again:

```sh
tui-shot -o frame.ansi
tui-shot --input frame.ansi --theme light -o screenshot.svg
```

Review captures before sharing them. The raw `.ansi` file is not sanitized.

## See it in action

Every screenshot below came from `tui-shot` and a live Herdr pane.

### Vellum

[Vellum](https://github.com/dkarter/vellum) puts agents, tasks, and pull
requests in a searchable terminal picker.

![Vellum agent picker with two demo agents](https://github.com/user-attachments/assets/8b11ee8f-3092-415e-b202-e48243d7a9d7)

### Ink

[Ink](https://github.com/dkarter/ink) brings Vim editing to terminal prompts.
Here it’s editing a Markdown guide in Normal mode.

![Ink textarea showing a Markdown guide in Vim Normal mode](https://github.com/user-attachments/assets/0a13bb23-f42c-47c3-9170-0aa5bfe27303)

### Neovim

A Lua screenshot helper in
[Dorian’s Neovim setup](https://github.com/dkarter/dotfiles/tree/master/config/nvim),
with Tokyo Night syntax highlighting.

![Neovim editing a Lua helper with Tokyo Night syntax highlighting](https://github.com/user-attachments/assets/5a6630d3-95a6-4259-94ec-7bd21a8c9abe)

## Publish text, not pixels

HTML output preserves colors and terminal text, including Unicode box drawing.
Use `--fragment` to embed a card on a site that allows inline styles. Switch
`data-theme` between `dark` and `light`, or match your site with CSS variables:

```css
.tui-shot {
  --tui-bg: #10141c;
  --tui-fg: #eaf2ff;
  --tui-border: #3c4356;
  --tui-c6: #5eead4;       /* ANSI cyan; c0–c15 are palette entries */
  --tui-rgb-fa9646: #ffcc66; /* Override one exact RGB value */
}
```

## Script a tutorial

Run the app in a dedicated pane at a consistent size. After sending keys,
wait for text unique to the next screen before capturing it:

```sh
set -e
pane="${1:?pass the demo pane ID}"
herdr pane send-keys "$pane" s
herdr pane wait-output "$pane" --source visible --match 'Settings — General' \
  --timeout 5000 >/dev/null
tui-shot --pane "$pane" --title "02 · Settings" -o 02-settings.png
```

## Completions

With Mise activated, it can load the packaged completions for you. Otherwise
generate a script with `tui-shot completions bash`, `tui-shot completions fish`,
or `tui-shot completions zsh`.

[Development and releases](DEVELOPMENT.md) · [MIT license](LICENSE)
