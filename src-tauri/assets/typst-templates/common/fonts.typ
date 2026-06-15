// doc-agent 内置字体栈：优先 Windows/macOS 系统字体，回退 Typst 内嵌与 Noto CJK。
// 不捆绑微软字体；在已安装环境下自动命中。

#let font-serif-zh = (
  "Times New Roman",
  "SimSun",
  "Songti SC",
  "STSong",
  "Noto Serif CJK SC",
  "Libertinus Serif",
)

#let font-sans-zh = (
  "Microsoft YaHei",
  "PingFang SC",
  "Heiti SC",
  "SimHei",
  "Noto Sans CJK SC",
  "Arial",
)

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
