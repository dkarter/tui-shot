# tui-shot

**Blazing-fast, agent-native terminal screenshots. Built in Rust.**

Capture any live [Herdr](https://herdr.dev/) pane without switching to it. Drop
a crisp 2× PNG into your README, publish an SVG, or share a terminal card with
text readers can select, copy, and recolor. One command gives agents and scripts
the same capture workflow you use by hand.

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

### Install the agent skill

The optional [GitHub README Images skill](skills/github-readme-images/SKILL.md)
teaches your agent to capture terminal screenshots, upload them to a GitHub
issue, and embed the images in a README without committing PNGs.

```sh
npx skills add dkarter/tui-shot --skill github-readme-images -g
```

Omit `-g` to install it for the current project only.

Starting with the next binary release, `tui-shot --skill` prints the skill.
Each release archive ships the skill, and the signed Packslip manifest points
to it.

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
with Tokyo Night syntax highlighting and Agent Mono Nerd Font icons.

![Neovim editing a Lua helper with Tokyo Night and Agent Mono Nerd Font icons](https://github.com/user-attachments/assets/79601ef9-c261-4611-ac71-6e764a0fdb2a)

Use `--font 'Agent Mono Nerd Font'` with the next release, or pass the name of
your own installed monospace font. PNG and SVG use a fixed terminal grid, so
glyph spacing can differ slightly from your terminal.

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
