#import "/doc-agent/typst/common/fonts.typ": *
#import "/doc-agent/typst/common/page.typ": *

#show: apply-en-body
#page-a4()
#footer-page-no()

#apply-en-title(
  [Technical Investigation Report],
  [Sample template — edit title and body after copying],
)

#align(center)[
  #grid(
    columns: (1fr, 1fr),
    gutter: 1em,
    [Author: Jane Doe],
    [Department: R&D],
    [Date: #datetime.today().display()],
    [Classification: Internal],
  )
]

#v(1.5em)
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
  inset: 8pt,
  stroke: 0.5pt,
  [*Dimension*], [*Option A*], [*Option B*], [*Option C*],
  [Performance], [High], [Medium], [Medium],
  [Cost], [High], [Low], [Medium],
  [Operations], [Low], [Medium], [High],
)

= Conclusion

We recommend *Option B* and a proof-of-concept in the next phase.

#v(2em)
#align(right)[
  #text(size: 9pt, fill: gray)[doc-agent built-in · report/report-en]
]
