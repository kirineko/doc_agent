// 中文试卷模板 · 复制到项目后修改元信息与题目
#import "/doc-agent/typst/common/fonts.typ": *
#import "/doc-agent/typst/common/page.typ": page-exam, footer-page-no
#import "/doc-agent/typst/common/exam.typ": *
#import "/doc-agent/typst/common/tokens.typ": *

#show: apply-zh-body.with(theme: exam-theme())
#page-exam()
#footer-page-no()

#exam-header-zh(
  [高等数学（上）期中考试],
  [2025–2026 学年第一学期],
  [120 分钟],
  [100],
)

= 一、填空题（每空 4 分，共 20 分）

+ $lim_(x -> 0) (sin x) / x = $ #fill-blank()
+ 函数 $f(x) = x^2 - 2x + 1$ 在 $x = 1$ 处的导数为 #fill-blank()
+ $integral_0^1 x dif x = $ #fill-blank()
+ 矩阵 $mat(1, 2; 3, 4)$ 的行列式为 #fill-blank()
+ 级数 $sum_(n=1)^oo 1/n^2$ #text[（收敛 / 发散）] #fill-blank(width: 2.5cm)

= 二、选择题（每题 5 分，共 25 分）

+ 下列函数中在 $RR$ 上可导的是 \
  #mc-options(
    [$abs(x)$],
    [$x abs(x)$],
    [$x^2$],
    [$sqrt(abs(x))$],
  )

+ $f(x) = ln x$ 的定义域为 \
  #mc-options(
    [$(0, +oo)$],
    [$[0, +oo)$],
    [$RR$],
    [$(-oo, 0)$],
  )

+ 若 $f'(x_0) = 0$ 且 $f''(x_0) > 0$，则 $x_0$ 是 \
  #mc-options(
    [极大值点],
    [极小值点],
    [拐点],
    [无法判断],
  )

#pagebreak()

= 三、计算与证明题（共 55 分）

#calc-counter-reset()

#calc-item(10)[求 $f(x) = x^3 - 3x + 1$ 的极值点并判定极大极小]
#calc-item(15)[计算 $display(integral e^x sin x dif x)$]
#calc-item(15)[证明：若 $f$ 在 $[a, b]$ 连续、在 $(a, b)$ 可导且 $f(a) = f(b)$，则存在 $xi in (a, b)$ 使 $f'(xi) = 0$]
#calc-item(15)[讨论级数 $display(sum_(n=1)^oo (-1)^n / sqrt(n))$ 的收敛性]
#calc-item(15)[设 $f(x) = cases(
  (2 x^3) / (1 + x^2), & x <= 1,
  1, & x > 1,
)$，计算 $display(integral_(-1)^4 f(x) dif x)$]

#v(sp-xl)
#align(center)[
  #text(size: fs-footnote, fill: color-muted)[—— 试卷结束 ——]
]
