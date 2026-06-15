// doc-agent 内置字体栈：平台系统字体优先，回退捆绑 Noto Sans/Serif SC。
#import "/doc-agent/typst/common/fonts-stack.typ": *

#let font-serif-en = (
  "Times New Roman",
  "Libertinus Serif",
  "Noto Serif",
)

#let font-sans-en = (
  "Arial",
  "Helvetica Neue",
  "Noto Sans",
)

#let font-mono = (
  "Consolas",
  "Menlo",
  "Courier New",
  "Libertinus Mono",
)

#let font-math = "New Computer Modern Math"

#let apply-zh-body(body) = {
  set text(font: font-serif-zh, lang: "zh", region: "cn", size: 11pt)
  set par(justify: true, leading: 0.65em, spacing: 1.2em)
  show heading: set text(font: font-sans-zh)
  show math.equation: set text(font: font-math)
  body
}

#let apply-en-body(body) = {
  set text(font: font-serif-en, lang: "en", size: 11pt)
  set par(justify: true, leading: 0.65em, spacing: 1.2em)
  show heading: set text(font: font-sans-en)
  show math.equation: set text(font: font-math)
  body
}

#let apply-zh-title(title, subtitle: none) = {
  align(center)[
    #text(size: 18pt, font: font-sans-zh, weight: "bold")[#title]
    #if subtitle != none [
      #v(0.4em)
      #text(size: 12pt)[#subtitle]
    ]
  ]
  v(1.2em)
}

#let apply-en-title(title, subtitle: none) = {
  align(center)[
    #text(size: 18pt, font: font-sans-en, weight: "bold")[#title]
    #if subtitle != none [
      #v(0.4em)
      #text(size: 12pt)[#subtitle]
    ]
  ]
  v(1.2em)
}
