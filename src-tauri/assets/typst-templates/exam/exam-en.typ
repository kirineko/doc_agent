#import "/doc-agent/typst/common/fonts.typ": *
#import "/doc-agent/typst/common/page.typ": *
#import "/doc-agent/typst/common/exam.typ": calc-item, calc-counter-reset

#show: apply-en-body
#page-a4-compact(margin: 1.8cm)
#footer-page-no()

#let exam-meta(course, term, duration, total) = {
  align(center)[
    #text(size: 16pt, font: font-sans-en, weight: "bold")[#course]
    #v(0.3em)
    #text(size: 12pt)[#term · Duration: #duration · Total: #total points]
  ]
  v(0.8em)
  grid(
    columns: (1fr, 1fr, 1fr),
    [Name: __________],
    [ID: __________],
    [Section: __________],
  )
  v(1em)
  line(length: 100%)
  v(0.8em)
}

#exam-meta(
  [Calculus I — Midterm Examination],
  [Fall 2025],
  [120 min],
  [100],
)

= Part I — Fill in the blanks (4 pts each, 20 pts total)

+ $lim_(x -> 0) (sin x) / x = $ #h(3cm)

+ If $f(x) = x^2 - 2x + 1$, then $f'(1) = $ #h(3cm)

+ $integral_0^1 x dif x = $ #h(3cm)

+ $det mat(1, 2; 3, 4) = $ #h(3cm)

+ The series $sum_(n=1)^oo 1/n^2$ is #text[(convergent / divergent)] #h(2cm)

= Part II — Multiple choice (5 pts each, 25 pts total)

+ Which function is differentiable on $RR$? \
  #pad(left: 1.5em)[
    A. $abs(x)$ #h(1.5cm) B. $x abs(x)$ #h(1.5cm) C. $x^2$ #h(1.5cm) D. $sqrt(abs(x))$
  ]

+ The domain of $f(x) = ln x$ is \
  #pad(left: 1.5em)[
    A. $(0, +oo)$ #h(1cm) B. $[0, +oo)$ #h(1cm) C. $RR$ #h(1cm) D. $(-oo, 0)$
  ]

#pagebreak()

= Part III — Problems (55 pts total)

#calc-counter-reset()

#calc-item(10)[Find and classify the extrema of $f(x) = x^3 - 3x + 1$.]

#calc-item(15)[Evaluate $display(integral e^x sin x dif x)$.]

#calc-item(15)[State and prove Rolle's theorem.]

#calc-item(15)[Determine convergence of $display(sum_(n=1)^oo (-1)^n / sqrt(n))$.]

#calc-item(15)[Let $f(x) = cases(
  (2 x^3) / (1 + x^2), & x <= 1,
  1, & x > 1,
)$. Evaluate $display(integral_(-1)^4 f(x) dif x)$.]

#v(2em)
#align(center)[
  #text(size: 9pt)[— End of examination —]
]
