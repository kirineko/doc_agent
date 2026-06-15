// English exam template · edit metadata and questions after copying
#import "/doc-agent/typst/common/fonts.typ": *
#import "/doc-agent/typst/common/page.typ": page-exam, footer-page-no
#import "/doc-agent/typst/common/exam.typ": *
#import "/doc-agent/typst/common/tokens.typ": *

#show: apply-en-body.with(theme: exam-theme())
#page-exam()
#footer-page-no()

#exam-header-en(
  [Calculus I — Midterm Examination],
  [Fall 2025],
  [120 min],
  [100],
)

= Part I — Fill in the blanks (4 pts each, 20 pts total)

+ $lim_(x -> 0) (sin x) / x = $ #fill-blank()
+ If $f(x) = x^2 - 2x + 1$, then $f'(1) = $ #fill-blank()
+ $integral_0^1 x dif x = $ #fill-blank()
+ $det mat(1, 2; 3, 4) = $ #fill-blank()
+ The series $sum_(n=1)^oo 1/n^2$ is #text[(convergent / divergent)] #fill-blank(width: 2.5cm)

= Part II — Multiple choice (5 pts each, 25 pts total)

+ Which function is differentiable on $RR$? \
  #mc-options(
    [$abs(x)$],
    [$x abs(x)$],
    [$x^2$],
    [$sqrt(abs(x))$],
  )

+ The domain of $f(x) = ln x$ is \
  #mc-options(
    [$(0, +oo)$],
    [$[0, +oo)$],
    [$RR$],
    [$(-oo, 0)$],
  )

+ If $f'(x_0) = 0$ and $f''(x_0) > 0$, then $x_0$ is a \
  #mc-options(
    [local maximum],
    [local minimum],
    [inflection point],
    [undetermined],
  )

#pagebreak()

= Part III — Problems (55 pts total)

#calc-counter-reset()

#calc-item(10, lang: "en")[Find and classify the extrema of $f(x) = x^3 - 3x + 1$]
#calc-item(15, lang: "en")[Evaluate $display(integral e^x sin x dif x)$]
#calc-item(15, lang: "en")[State and prove Rolle's theorem]
#calc-item(15, lang: "en")[Determine convergence of $display(sum_(n=1)^oo (-1)^n / sqrt(n))$]
#calc-item(15, lang: "en")[Let $f(x) = cases(
  (2 x^3) / (1 + x^2), & x <= 1,
  1, & x > 1,
)$. Evaluate $display(integral_(-1)^4 f(x) dif x)$]

#v(sp-xl)
#align(center)[
  #text(size: fs-footnote, fill: color-muted)[— End of examination —]
]
