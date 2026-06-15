#import "/doc-agent/typst/common/fonts.typ": *
#import "/doc-agent/typst/common/page.typ": *
#import "/doc-agent/typst/common/exam.typ": calc-item, calc-counter-reset

#show: apply-zh-body
#page-a4-compact(margin: 1.8cm)
#footer-page-no()

#let exam-meta(course, term, duration, total) = {
  align(center)[
    #text(size: 16pt, font: font-sans-zh, weight: "bold")[#course]
    #v(0.3em)
    #text(size: 12pt)[#term · 考试时间 #duration · 满分 #total 分]
  ]
  v(0.8em)
  grid(
    columns: (1fr, 1fr, 1fr),
    [学院：__________],
    [姓名：__________],
    [学号：__________],
  )
  v(1em)
  line(length: 100%)
  v(0.8em)
}

#exam-meta(
  [高等数学（上）期中考试],
  [2025–2026 学年第一学期],
  [120 分钟],
  [100],
)

= 一、填空题（每空 4 分，共 20 分）

+ $lim_(x -> 0) (sin x) / x = $ #h(3cm)

+ 函数 $f(x) = x^2 - 2x + 1$ 在 $x = 1$ 处的导数为 #h(3cm)

+ $integral_0^1 x dif x = $ #h(3cm)

+ 矩阵 $mat(1, 2; 3, 4)$ 的行列式为 #h(3cm)

+ 级数 $sum_(n=1)^oo 1/n^2$ #text[（收敛 / 发散）] #h(2cm)

= 二、选择题（每题 5 分，共 25 分）

+ 下列函数中在 $RR$ 上可导的是 \
  #pad(left: 1.5em)[
    A. $abs(x)$ #h(1.5cm) B. $x abs(x)$ #h(1.5cm) C. $x^2$ #h(1.5cm) D. $sqrt(abs(x))$
  ]

+ $f(x) = ln x$ 的定义域为 \
  #pad(left: 1.5em)[
    A. $(0, +oo)$ #h(1cm) B. $[0, +oo)$ #h(1cm) C. $RR$ #h(1cm) D. $(-oo, 0)$
  ]

#pagebreak()

= 三、计算与证明题（共 55 分）

#calc-counter-reset()

#calc-item(10)[求 $f(x) = x^3 - 3x + 1$ 的极值点并判定极大极小。]

#calc-item(15)[计算 $display(integral e^x sin x dif x)$。]

#calc-item(15)[证明：若 $f$ 在 $[a, b]$ 连续、在 $(a, b)$ 可导且 $f(a) = f(b)$，则存在 $xi in (a, b)$ 使 $f'(xi) = 0$。]

#calc-item(15)[讨论级数 $display(sum_(n=1)^oo (-1)^n / sqrt(n))$ 的收敛性。]

#calc-item(15)[设 $f(x) = cases(
  (2 x^3) / (1 + x^2), & x <= 1,
  1, & x > 1,
)$，计算 $display(integral_(-1)^4 f(x) dif x)$。]

#v(2em)
#align(center)[
  #text(size: 9pt)[—— 试卷结束 ——]
]
