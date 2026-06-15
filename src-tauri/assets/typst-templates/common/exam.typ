#import "/doc-agent/typst/common/fonts.typ": *
#import "/doc-agent/typst/common/tokens.typ": *

// 试卷辅助模块。列表题用 `+` 连续写；不要在两个 `+` 项之间单独插入 `#v(3.5cm)`，否则编号会重置为 1。

#let calc-counter = counter("calc-item")

/// 计算/证明题：自动递增题号，下方预留答题空白（`below` 间距）。
#let calc-item(score, body, lang: "zh") = {
  calc-counter.step()
  let score-tag = if lang == "en" {
    [(#score pts)]
  } else {
    [（#score 分）]
  }
  block(breakable: true, below: sp-exam-calc-below)[
    #context calc-counter.display() #score-tag #body
  ]
}

#let calc-counter-reset() = {
  calc-counter.update(0)
}

/// 填空题答题横线。勿用 `#h()`（仅空白无横线），勿用句号 `。` 占位。
#let fill-blank(width: 3cm) = box(width: width, baseline: 0.65em)[
  #line(length: 100%, stroke: stroke-rule + color-ink)
]

#let field-line(label, width: 4.2cm) = {
  [#label #fill-blank(width: width)]
}

#let exam-header-zh(course, term, duration, total) = {
  align(center)[
    #text(size: fs-h1, font: font-heading-zh, weight: "bold", fill: color-ink)[#course]
    #v(sp-xs)
    #text(size: fs-lead, fill: color-muted)[#term · 考试时间 #duration · 满分 #total 分]
  ]
  v(sp-sm)
  grid(
    columns: (1fr, 1fr, 1fr),
    field-line([学院：]),
    field-line([姓名：]),
    field-line([学号：]),
  )
  v(sp-md)
  line(length: 100%, stroke: stroke-rule + color-rule)
  v(sp-sm)
}

#let exam-header-en(course, term, duration, total) = {
  align(center)[
    #text(size: fs-h1, font: font-heading-en, weight: "bold", fill: color-ink)[#course]
    #v(sp-xs)
    #text(size: fs-lead, fill: color-muted)[#term · Duration: #duration · Total: #total points]
  ]
  v(sp-sm)
  grid(
    columns: (1fr, 1fr, 1fr),
    field-line([Name: ], width: 4.5cm),
    field-line([ID: ], width: 4.5cm),
    field-line([Section: ], width: 4.5cm),
  )
  v(sp-md)
  line(length: 100%, stroke: stroke-rule + color-rule)
  v(sp-sm)
}

#let mc-options(a, b, c, d, gap: 1.2cm) = {
  pad(left: 1.5em)[
    A. #a #h(gap) B. #b #h(gap) C. #c #h(gap) D. #d
  ]
}
