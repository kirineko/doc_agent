// English lecture notes template
#import "/doc-agent/typst/common/fonts.typ": *
#import "/doc-agent/typst/common/page.typ": page-lecture, footer-page-no
#import "/doc-agent/typst/common/lecture.typ": definition-en, example-en
#import "/doc-agent/typst/common/tokens.typ": *

#let lecture-theme = make-theme(palette: "forest")
#show: apply-en-body.with(theme: lecture-theme)
#page-lecture()
#footer-page-no()

#apply-en-title(
  [Calculus I — Lecture Notes],
  subtitle: [Limits and Continuity · Sample template],
  theme: lecture-theme,
)

= Course information

#table(
  columns: (auto, 1fr),
  table.header([Item], [Detail]),
  [Instructor], [Prof. Wang],
  [Audience], [First-year engineering],
  [Textbook], [Calculus, 7th ed.],
)

= 1. Limits of sequences

#definition-en(
  [Sequence limit],
  [
    A sequence $(a_n)$ converges to $A$ if for every $epsilon > 0$ there exists $N in NN$ such that
    $abs(a_n - A) < epsilon$ whenever $n > N$. We write $lim_(n->oo) a_n = A$.
  ],
  theme: lecture-theme,
)

#example-en(
  [1],
  [
    Show that $lim_(n->oo) 1/n = 0$.

    *Proof.* Given $epsilon > 0$, choose $N > 1/epsilon$. Then $n > N$ implies $1/n < epsilon$.
  ],
  theme: lecture-theme,
)

= 2. Limits of functions

Standard equivalents as $x -> 0$:

#table(
  columns: (1fr, 1fr),
  table.header([Relation], [Relation]),
  [$sin x tilde x$], [$tan x tilde x$],
  [$ln(1+x) tilde x$], [$e^x - 1 tilde x$],
)

= 3. In-class exercises

+ Evaluate $lim_(x->0) (sin 3x) / x$
+ Discuss continuity of
  $ f(x) = cases(
      x sin(1/x), & x != 0,
      0, & x = 0,
    ) $ at $x = 0$

#v(sp-md)
#align(right)[
  #text(size: fs-footnote, fill: color-muted)[doc-agent built-in · lecture/lecture-en]
]
