// English academic paper template
#import "/doc-agent/typst/common/fonts.typ": *
#import "/doc-agent/typst/common/page.typ": *

#show: apply-en-body
#page-a4(margin: 2.5cm)
#footer-page-no()

#set heading(numbering: "1.1")

#align(center)[
  #text(size: 16pt, font: font-sans-en, weight: "bold")[
    Mathematical Document Typesetting with Typst
  ]
  #v(0.6em)
  #text(size: 12pt)[
    Alice Smith#super[1]　Bob Jones#super[2]
  ]
  #v(0.4em)
  #text(size: 10pt)[
    #super[1]Dept. of Mathematics　#super[2]Dept. of Computer Science
  ]
]

#v(1.2em)
#block(fill: luma(245), inset: 12pt, radius: 4pt)[
  #text(weight: "bold")[Abstract]　
  We study practical Typst workflows for documents with heavy mathematical notation,
  including font stacks, theorem environments, and cross-references.
  #v(0.4em)
  #text(weight: "bold")[Keywords]　Typst; mathematical typesetting; PDF; templates
]

#v(1em)
= Introduction

Mathematical manuscripts demand consistent equation layout and numbering. Typst offers a modern syntax and fast compilation suitable for offline desktop agents.

= Methods

== Fonts

Body text uses Times New Roman; math uses New Computer Modern Math:

$ integral_a^b f(x) dif x = F(b) - F(a) $

== Theorem statement

#block(fill: luma(250), inset: 10pt, radius: 3pt)[
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
