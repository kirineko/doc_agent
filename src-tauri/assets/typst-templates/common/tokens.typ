// doc-agent 设计 token：字号/间距/行距/配色/线宽/页边距 + 可主题化强调色
// 锁定轴：字号阶、正文墨色、行距；自由轴：palette/accent/density/heading-style/cover

#let fs-footnote = 9pt
#let fs-small = 9.5pt
#let fs-body = 11pt
#let fs-lead = 12pt
#let fs-h3 = 12pt
#let fs-h2 = 14pt
#let fs-h1 = 16pt
#let fs-title = 20pt

#let sp-2xs = 0.25em
#let sp-xs = 0.4em
#let sp-sm = 0.6em
#let sp-md = 1em
#let sp-lg = 1.6em
#let sp-xl = 2.4em
#let sp-exam-calc-below = 3.5cm

#let leading-cjk = 0.9em
#let leading-latin = 0.65em
#let par-spacing = 1.1em
#let indent-cjk = 2em

#let color-ink = rgb("#1a1a1a")
#let color-muted = rgb("#5f6368")
#let color-rule = rgb("#d0d0d0")

#let stroke-hair = 0.5pt
#let stroke-rule = 0.75pt
#let stroke-heavy = 1pt

#let margin-report = (x: 2.5cm, y: 2.5cm)
#let margin-paper = (x: 2.4cm, y: 2.6cm)
#let margin-exam = 1.8cm
#let margin-lecture = (x: 2.2cm, y: 2cm)

#let density-scale-map = (
  "compact": 0.85,
  "normal": 1.0,
  "relaxed": 1.2,
)

#let palettes = (
  "academic-blue": (accent: rgb("#1f4e79"), fill: rgb("#f5f7fa")),
  "slate": (accent: rgb("#334155"), fill: rgb("#f1f5f9")),
  "burgundy": (accent: rgb("#7a1f3d"), fill: rgb("#faf5f6")),
  "forest": (accent: rgb("#1f5132"), fill: rgb("#f3f8f4")),
  "charcoal": (accent: rgb("#1a1a1a"), fill: rgb("#f5f5f5")),
)

#let make-theme(
  palette: "academic-blue",
  accent: none,
  fill: none,
  density: "normal",
  heading-style: "accent-rule",
  cover: "none",
) = {
  let base = palettes.at(palette)
  let accent-color = if accent != none { accent } else { base.accent }
  let fill-color = if fill != none { fill } else { base.fill }
  let scale = density-scale-map.at(density)
  (
    palette: palette,
    accent: accent-color,
    fill: fill-color,
    density-scale: scale,
    heading-style: heading-style,
    cover: cover,
  )
}

#let default-theme = make-theme()
#let exam-theme(accent: none, fill: none, density: "compact", heading-style: "plain", cover: "none") = {
  make-theme(
    palette: "charcoal",
    accent: none,
    fill: none,
    density: density,
    heading-style: heading-style,
    cover: cover,
  )
}

#let scaled-sp(size, theme) = size * theme.density-scale

#let theme-heading-size(level) = {
  if level == 1 { fs-h1 } else if level == 2 { fs-h2 } else { fs-h3 }
}
