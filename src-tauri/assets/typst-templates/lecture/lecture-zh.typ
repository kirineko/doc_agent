// 中文讲义模板
#import "/doc-agent/typst/common/fonts.typ": *
#import "/doc-agent/typst/common/page.typ": page-a4, footer-page-no
#import "/doc-agent/typst/common/lecture.typ": definition-zh, example-zh

#show: apply-zh-body
#page-a4(margin: 2cm)
#footer-page-no()

#apply-zh-title(
  [高等数学讲义 · 第一讲],
  [极限与连续 · 示例模板],
)

= 课程信息

#table(
  columns: (auto, 1fr),
  stroke: 0.5pt,
  inset: 8pt,
  [授课教师], [王教授],
  [适用对象], [工科大一],
  [教材], [《高等数学》第七版],
)

= 1. 数列极限

#definition-zh[数列极限][
  设 ${a_n}$ 为实数列。若存在常数 $A$，使得对任意 $epsilon > 0$，存在 $N in NN$，当 $n > N$ 时恒有
  $ abs(a_n - A) < epsilon $，
  则称 ${a_n}$ 收敛于 $A$，记作 $lim_(n->oo) a_n = A$。
]

#example-zh[1][
  证明 $lim_(n->oo) 1/n = 0$。

  *证*　任给 $epsilon > 0$，取 $N > 1/epsilon$，则当 $n > N$ 时有 $1/n < epsilon$。证毕。
]

= 2. 函数极限

常用等价无穷小（$x -> 0$）：

#table(
  columns: (1fr, 1fr),
  stroke: 0.5pt,
  inset: 8pt,
  [$sin x tilde x$], [$tan x tilde x$],
  [$ln(1+x) tilde x$], [$e^x - 1 tilde x$],
)

= 3. 课堂练习

+ 计算 $lim_(x->0) (sin 3x) / x$
+ 讨论 $f(x) = cases(
    x sin(1/x), & x != 0,
    0, & x = 0,
  )$ 在 $x=0$ 处的连续性

#v(1em)
#align(right)[
  #text(size: 9pt, fill: gray)[doc-agent 内置 · lecture/lecture-zh]
]
