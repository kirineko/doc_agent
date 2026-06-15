// 中文技术报告模板
#import "/doc-agent/typst/common/fonts.typ": *
#import "/doc-agent/typst/common/page.typ": *
#import "/doc-agent/typst/common/tokens.typ": *

#let report-theme = make-theme(palette: "academic-blue")
#show: apply-zh-body.with(theme: report-theme)
#page-a4()
#footer-page-no()

#apply-zh-title(
  [技术调研报告],
  subtitle: [—— 示例模板，复制后修改标题与正文],
  theme: report-theme,
)

#align(center)[
  #grid(
    columns: (1fr, 1fr),
    gutter: sp-md,
    [撰写人：张三],
    [部门：研发中心],
    [日期：#datetime.today().display()],
    [密级：内部],
  )
]

#v(sp-lg)
#outline(title: [目录], indent: auto)
#pagebreak()

= 执行摘要

本报告对某技术方案进行调研与评估，供决策参考。请替换为实际摘要段落。

= 背景与目标

== 业务背景

说明项目背景、痛点与约束条件。

== 调研目标

+ 梳理可选技术路线；
+ 对比性能、成本与可维护性；
+ 给出推荐方案与实施建议。

= 方案对比

#table(
  columns: (auto, 1fr, 1fr, 1fr),
  align: (left, left, left, left),
  table.header([*维度*], [*方案 A*], [*方案 B*], [*方案 C*]),
  [性能], [高], [中], [中],
  [成本], [高], [低], [中],
  [运维复杂度], [低], [中], [高],
)

= 结论与建议

推荐采用 *方案 B*，并在下一阶段开展 PoC 验证。

#v(sp-xl)
#align(right)[
  #text(size: fs-footnote, fill: color-muted)[本模板由 doc-agent 内置 · report/report-zh]
]
