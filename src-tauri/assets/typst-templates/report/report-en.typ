// English technical report template
#import "/doc-agent/typst/common/fonts.typ": *
#import "/doc-agent/typst/common/page.typ": *
#import "/doc-agent/typst/common/tokens.typ": *

#let report-theme = make-theme(palette: "academic-blue")
#show: apply-en-body.with(theme: report-theme)
#page-a4()
#footer-page-no()

#apply-en-title(
  [Technical Investigation Report],
  subtitle: [Sample template — edit title and body after copying],
  theme: report-theme,
)

#align(center)[
  #grid(
    columns: (1fr, 1fr),
    gutter: sp-md,
    [Author: Jane Doe],
    [Department: R&D],
    [Date: #datetime.today().display()],
    [Classification: Internal],
  )
]

#v(sp-lg)
#outline(title: [Table of Contents], indent: auto)
#pagebreak()

= Executive Summary

This report evaluates candidate technical approaches. Replace with your actual summary.

= Background and Objectives

== Context

Describe the business context, pain points, and constraints.

== Goals

+ Survey viable technical routes;
+ Compare performance, cost, and maintainability;
+ Recommend an implementation plan.

= Comparison

#table(
  columns: (auto, 1fr, 1fr, 1fr),
  align: (left, left, left, left),
  table.header([*Dimension*], [*Option A*], [*Option B*], [*Option C*]),
  [Performance], [High], [Medium], [Medium],
  [Cost], [High], [Low], [Medium],
  [Operations], [Low], [Medium], [High],
)

= Conclusion

We recommend *Option B* and a proof-of-concept in the next phase.

#v(sp-xl)
#align(right)[
  #text(size: fs-footnote, fill: color-muted)[doc-agent built-in · report/report-en]
]
