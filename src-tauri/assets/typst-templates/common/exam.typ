#import "/doc-agent/typst/common/fonts.typ": *

// 试卷辅助模块。列表题用 `+` 连续写；不要在两个 `+` 项之间单独插入 `#v(3.5cm)`，否则编号会重置为 1。

#let calc-counter = counter("calc-item")

/// 计算/证明题：自动递增题号，下方预留答题空白（`below` 间距）。
/// - 中文默认：`#calc-item(10)[求 $f'(x)$ …]`
/// - 英文：`#calc-item(10, lang: "en")[Find $f'(x)$ …]`
#let calc-item(score, body, lang: "zh") = {
  // step() 必须在 context 外：同一 context 内 step 后再 display 会读到旧值（从 0 起）。
  calc-counter.step()
  let score-tag = if lang == "en" {
    [(#score pts)]
  } else {
    [（#score 分）]
  }
  block(breakable: true, below: 3.5cm)[
    #context calc-counter.display() #score-tag #body
  ]
}

#let calc-counter-reset() = {
  calc-counter.update(0)
}

/// 填空题答题横线。勿用 `#h()`（仅空白无横线），勿用句号 `。` 占位。
/// 调用：`#fill-blank()` 或 `#fill-blank(width: 2.5cm)`（带参数须写 `width:`）。
#let fill-blank(width: 3cm) = box(width: width, baseline: 0.65em)[
  #line(length: 100%, stroke: 0.75pt)
]

/// 表头信息行：标签 + 答题横线。
#let field-line(label, width: 4.2cm) = {
  [#label #fill-blank(width: width)]
}

/// 中文试卷页眉：课程名、学期、时长、满分 + 学院/姓名/学号。
#let exam-header-zh(course, term, duration, total) = {
  align(center)[
    #text(size: 16pt, font: font-sans-zh, weight: "bold")[#course]
    #v(0.3em)
    #text(size: 12pt)[#term · 考试时间 #duration · 满分 #total 分]
  ]
  v(0.8em)
  grid(
    columns: (1fr, 1fr, 1fr),
    field-line([学院：]),
    field-line([姓名：]),
    field-line([学号：]),
  )
  v(1em)
  line(length: 100%)
  v(0.8em)
}

/// 英文试卷页眉。
#let exam-header-en(course, term, duration, total) = {
  align(center)[
    #text(size: 16pt, font: font-sans-en, weight: "bold")[#course]
    #v(0.3em)
    #text(size: 12pt)[#term · Duration: #duration · Total: #total points]
  ]
  v(0.8em)
  grid(
    columns: (1fr, 1fr, 1fr),
    field-line([Name: ], width: 4.5cm),
    field-line([ID: ], width: 4.5cm),
    field-line([Section: ], width: 4.5cm),
  )
  v(1em)
  line(length: 100%)
  v(0.8em)
}

/// 选择题四选项排版（选项间用 `#h` 留空即可，不是答题横线）。
#let mc-options(a, b, c, d, gap: 1.2cm) = {
  pad(left: 1.5em)[
    A. #a #h(gap) B. #b #h(gap) C. #c #h(gap) D. #d
  ]
}
