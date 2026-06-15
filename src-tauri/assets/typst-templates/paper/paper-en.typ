// English academic paper template
#import "/doc-agent/typst/common/fonts.typ": *
#import "/doc-agent/typst/common/page.typ": page-paper, footer-page-no
#import "/doc-agent/typst/common/tokens.typ": *

#let paper-theme = make-theme(palette: "slate")
#show: apply-en-body.with(theme: paper-theme)
#page-paper()
#footer-page-no()

#set heading(numbering: "1.1")

#align(center)[
  #text(size: fs-h1, font: font-heading-en, weight: "bold", fill: paper-theme.accent)[
    Mathematical Document Typesetting with Typst
  ]
  #v(sp-sm)
  #text(size: fs-lead)[
    Alice Smith#super[1]　Bob Jones#super[2]
  ]
  #v(sp-xs)
  #text(size: fs-small, fill: color-muted)[
    #super[1]Dept. of Mathematics　#super[2]Dept. of Computer Science
  ]
]

#v(sp-lg)
#block(fill: paper-theme.fill, inset: sp-md, radius: 4pt, stroke: stroke-hair + color-rule)[
  #text(weight: "bold", fill: paper-theme.accent)[Abstract]　
  We study practical Typst workflows for documents with heavy mathematical notation,
  including font stacks, theorem environments, and cross-references.
  #v(sp-xs)
  #text(weight: "bold", fill: paper-theme.accent)[Keywords]　Typst; mathematical typesetting; PDF; templates
]

#v(sp-md)
= Introduction

Mathematical manuscripts demand consistent equation layout and numbering. Typst offers a modern syntax and fast compilation suitable for offline desktop agents.

= Methods

== Fonts

Body text uses Times New Roman; math uses New Computer Modern Math:

$ integral_a^b f(x) dif x = F(b) - F(a) $

== Theorem statement

#block(fill: paper-theme.fill, inset: sp-sm, radius: 3pt, stroke: stroke-hair + color-rule)[
  *Theorem (Mean Value Theorem)*　
  If $f in C[a,b]$ and differentiable on $(a,b)$, then there exists $xi in (a,b)$ such that
  $ f'(xi) = (f(b) - f(a)) / (b - a) $.
]

= Experiments

We compare HTML-to-PDF printing with Typst compilation on identical content; Typst yields more stable pagination and equation numbering.

= Conclusion

We recommend Typst for formula-heavy PDF generation.

= References

#pad(left: 2em)[
  [1] Typst Contributors. *Typst: A new markup-based typesetting system*. 2024.
]
