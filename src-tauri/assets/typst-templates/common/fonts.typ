// doc-agent 内置字体栈：平台系统字体优先，回退捆绑 Noto Sans/Serif SC。
#import "/doc-agent/typst/common/fonts-stack.typ": *
#import "/doc-agent/typst/common/tokens.typ": *

#let font-body-zh = font-serif-zh
#let font-body-en = font-serif-en
#let font-heading-zh = font-sans-zh
#let font-heading-en = font-sans-en
#let font-emphasis-zh = font-sans-zh
#let font-emphasis-en = font-sans-en
#let font-math = "New Computer Modern Math"
#let font-mono = (
  "Consolas",
  "Menlo",
  "Courier New",
  "Libertinus Mono",
)

#let show-themed-heading(it, theme, lang: "zh") = {
  let accent = theme.accent
  let style = theme.heading-style
  let heading-font = if lang == "zh" { font-heading-zh } else { font-heading-en }
  set text(
    font: heading-font,
    size: theme-heading-size(it.level),
    fill: if style == "plain" { color-ink } else { accent },
    weight: "bold",
  )
  if style == "accent-rule" and it.level <= 2 {
    v(sp-xs * theme.density-scale)
    block(
      width: 100%,
      inset: (bottom: sp-2xs * theme.density-scale),
      stroke: (bottom: stroke-rule + accent),
    )[#it.body]
    v(sp-sm * theme.density-scale)
  } else if style == "accent-number" {
    v(sp-sm * theme.density-scale)
    grid(
      columns: (1.6em, 1fr),
      gutter: sp-xs,
      align: (center, left),
      box(
        fill: accent,
        inset: (x: sp-2xs, y: 0.08em),
        radius: 2pt,
      )[
        #text(size: fs-footnote, fill: white)[#it.level]
      ],
      it.body,
    )
    v(sp-xs * theme.density-scale)
  } else {
    v(sp-sm * theme.density-scale)
    it.body
    v(sp-xs * theme.density-scale)
  }
}

#let apply-cover-title(title, heading-font, title-size, subtitle: none, theme: default-theme) = {
  if theme.cover == "banner" {
    block(width: 100%, fill: theme.fill, inset: sp-md, stroke: (left: stroke-heavy + theme.accent))[
      #align(center)[
        #text(size: title-size, font: heading-font, weight: "bold", fill: theme.accent)[#title]
        #if subtitle != none [
          #v(sp-xs * theme.density-scale)
          #text(size: fs-lead, fill: color-muted)[#subtitle]
        ]
      ]
    ]
  } else {
    align(center)[
      #text(size: title-size, font: heading-font, weight: "bold", fill: theme.accent)[#title]
      #if subtitle != none [
        #v(sp-xs * theme.density-scale)
        #text(size: fs-lead, fill: color-muted)[#subtitle]
      ]
    ]
  }
}

#let apply-zh-body(body, theme: default-theme) = {
  set text(font: font-body-zh, lang: "zh", region: "cn", size: fs-body, fill: color-ink)
  set par(
    justify: true,
    leading: leading-cjk,
    spacing: par-spacing * theme.density-scale,
    first-line-indent: indent-cjk,
  )
  show heading: it => show-themed-heading(it, theme, lang: "zh")
  show link: set text(fill: theme.accent)
  show table: set table(
    stroke: stroke-hair + color-rule,
    inset: scaled-sp(sp-sm, theme),
  )
  show table.cell.where(y: 0): set table.cell(fill: theme.fill)
  show table.cell.where(y: 0): set text(weight: "bold", fill: theme.accent)
  show math.equation: set text(font: font-math)
  body
}

#let apply-en-body(body, theme: default-theme) = {
  set text(font: font-body-en, lang: "en", size: fs-body, fill: color-ink)
  set par(
    justify: true,
    leading: leading-latin,
    spacing: par-spacing * theme.density-scale,
  )
  show heading: it => show-themed-heading(it, theme, lang: "en")
  show link: set text(fill: theme.accent)
  show table: set table(
    stroke: stroke-hair + color-rule,
    inset: scaled-sp(sp-sm, theme),
  )
  show table.cell.where(y: 0): set table.cell(fill: theme.fill)
  show table.cell.where(y: 0): set text(weight: "bold", fill: theme.accent)
  show math.equation: set text(font: font-math)
  body
}

#let apply-zh-title(title, subtitle: none, theme: default-theme) = {
  apply-cover-title(title, font-heading-zh, fs-title, subtitle: subtitle, theme: theme)
  v(sp-lg * theme.density-scale)
}

#let apply-en-title(title, subtitle: none, theme: default-theme) = {
  apply-cover-title(title, font-heading-en, fs-title, subtitle: subtitle, theme: theme)
  v(sp-lg * theme.density-scale)
}
