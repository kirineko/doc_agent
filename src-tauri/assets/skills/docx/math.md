# Word 数学公式（docx.js Math / OMML）

通过 `skill_run` + 内置 `docx` 库生成 **Word 原生公式**（OMML）。公式在 Word / WPS 中可编辑。

> **无 LaTeX 解析器**：不能把 `$...$` 字符串自动转成公式，必须用 `Math*` 类**逐节点拼装** OMML 树。极复杂版式（大矩阵、严格分段函数排版）优先 `typst_to_pdf`。

## 何时阅读

交付物为 `.docx` 且含分数、极限、积分、上下标、根号、三角函数等公式时，**MUST** 先：

```json
{ "skill": "docx", "doc": "math.md" }
```

再编写 `skill_run` 脚本（可同时参考 `SKILL.md` 的中文字体与 A4 设置）。

## 可用 API（全局 `docx`）

| 类 | 用途 |
|----|------|
| `Math` | 公式容器，放在 `Paragraph` 的 `children` 里 |
| `MathRun` | 公式内文字/符号（如 `x`、`+`、`→`） |
| `MathFraction` | 分数 `numerator` / `denominator` |
| `MathSuperScript` / `MathSubScript` | 上标 / 下标 |
| `MathIntegral` | 积分 `∫`，可选 `subScript` / `superScript` |
| `MathSum` | 求和 `∑` |
| `MathRadical` | 根号 |
| `MathFunction` | 函数名 + 自变量，如 `sin(x)` |
| `MathLimitLower` | 下限极限，如 `lim` 下方 `x→0` |
| `MathLimitUpper` | 上限结构 |
| `MathCurlyBrackets` / `MathRoundBrackets` | 花括号 / 圆括号包裹 |
| `MathAngledBrackets` / `MathSquareBrackets` | 尖括号 / 方括号 |

**没有**矩阵、`cases` 分段、`\begin{align}` 等高级结构；可用多个 `MathRun` + 括号近似，或改 Typst PDF。

## 推荐写法：短辅助函数

在 `main()` 内定义（勿污染全局）：

```javascript
function mr(text) {
  return new MathRun(text);
}
function eq(children) {
  return new Math({ children });
}
```

- 公式节点用 `mr('...')`，不要用普通 `TextRun`。
- 中文说明用 `TextRun`，公式用 `eq([...])`，可混排在同一 `Paragraph`。

## 模板 1：行内公式（题干 + 公式 + 文字）

```javascript
const {
  Document, Packer, Paragraph, TextRun, Math, MathRun, MathFraction,
} = docx;

function mr(t) { return new MathRun(t); }
function eq(c) { return new Math({ children: c }); }

async function main() {
  const doc = new Document({
    styles: {
      default: {
        document: {
          run: {
            font: { ascii: "Times New Roman", eastAsia: "宋体", hAnsi: "Times New Roman" },
            size: 24,
          },
        },
      },
    },
    sections: [{
      children: [
        new Paragraph({
          children: [
            new TextRun("1. 求极限 "),
            eq([
              new MathFraction({
                numerator: [mr("x² − 4")],
                denominator: [mr("x − 2")],
              }),
            ]),
            new TextRun("，当 x → 2。"),
          ],
        }),
      ],
    }],
  });
  const b64 = await Packer.toBase64String(doc);
  doc_write("exam.docx", b64);
  return { ok: true };
}
```

## 模板 2：常见高数构件

```javascript
// 极限 lim_{x→2} (sin x)/x
eq([
  new MathLimitLower({ children: [mr("lim")], limit: [mr("x→0")] }),
  mr(" "),
  new MathFraction({ numerator: [mr("sin x")], denominator: [mr("x")] }),
]);

// 不定积分 ∫ tan²x dx
eq([new MathIntegral({ children: [mr("tan²x dx")] })]);

// 定积分 ∫_0^1 x dx
eq([
  new MathIntegral({
    children: [mr("x dx")],
    subScript: [mr("0")],
    superScript: [mr("1")],
  }),
]);

// 求和 ∑_{n=1}^{∞} 1/n²
eq([
  new MathSum({
    children: [mr("1/n²")],
    subScript: [mr("n=1")],
    superScript: [mr("∞")],
  }),
]);

// 指数与三角：e^x + sin(x)
eq([
  new MathSuperScript({ children: [mr("e")], superScript: [mr("x")] }),
  mr(" + "),
  new MathFunction({ name: [mr("sin")], children: [mr("x")] }),
]);

// 根号 √(x²+1)
eq([new MathRadical({ children: [mr("x²+1")] })]);
```

## 模板 3：独立一行显示公式（居中）

整段仅放一个 `Math`，段落居中：

```javascript
new Paragraph({
  alignment: docx.AlignmentType.CENTER,
  spacing: { before: 120, after: 120 },
  children: [
    eq([
      new MathIntegral({
        children: [mr("e^x sin x dx")],
      }),
    ]),
  ],
}),
```

## 模板 4：带题号的练习/试卷条目（通用版式）

结合 `SKILL.md` 的 numbering 或手写题号；每题一段，题末留空行：

```javascript
const {
  Document, Packer, Paragraph, TextRun, HeadingLevel,
  Math, MathRun, MathFraction, MathIntegral, MathSuperScript, MathFunction,
} = docx;

function mr(t) { return new MathRun(t); }
function eq(c) { return new Math({ children: c }); }

function question(no, textParts) {
  return new Paragraph({
    spacing: { after: 240 },
    children: [new TextRun(no + ". "), ...textParts],
  });
}

async function main() {
  const doc = new Document({
    styles: {
      default: {
        document: {
          run: {
            font: { ascii: "Times New Roman", eastAsia: "宋体", hAnsi: "Times New Roman" },
            size: 24,
          },
        },
      },
    },
    sections: [{
      properties: {
        page: {
          size: { width: 11906, height: 16838 },
          margin: { top: 1440, right: 1800, bottom: 1440, left: 1800 },
        },
      },
      children: [
        new Paragraph({
          heading: HeadingLevel.HEADING_1,
          children: [new TextRun("练习")],
        }),
        question("1", [
          new TextRun("求 "),
          eq([new MathFraction({ numerator: [mr("x²−4")], denominator: [mr("x−2")] })]),
        ]),
        question("2", [
          new TextRun("求导 "),
          eq([
            new MathSuperScript({ children: [mr("e")], superScript: [mr("x")] }),
            mr(" + x² + "),
            new MathFunction({ name: [mr("sin")], children: [mr("x")] }),
          ]),
        ]),
        question("3", [
          new TextRun("计算 "),
          eq([new MathIntegral({ children: [mr("tan²x dx")] })]),
        ]),
        new Paragraph({ children: [new TextRun("")] }), // 答题空白
        new Paragraph({ children: [new TextRun("")] }),
      ],
    }],
  });
  const b64 = await Packer.toBase64String(doc);
  doc_write("worksheet.docx", b64);
  return { ok: true };
}
```

## 模板 5：分段函数（近似）

无原生 `cases`；用花括号 + 多段 `MathRun`（排版较粗糙）：

```javascript
eq([
  new MathFunction({ name: [mr("f")], children: [mr("x")] }),
  mr(" = "),
  new MathCurlyBrackets({
    children: [
      mr("2x/(1+x²),  x≤1"),
      mr("  "),
      mr("1,  x>1"),
    ],
  }),
]);
```

## 限制与分工

| 能力 | docx Math | 建议 |
|------|-----------|------|
| 分数、极限、积分、求和、上下标 | ✅ | 用本页模板 |
| `sin`/`cos`/`ln` 等函数 | ✅ | `MathFunction` |
| 复杂矩阵、对齐方程组 | ❌ / 很繁琐 | `typst_to_pdf` |
| LaTeX 字符串直写 | ❌ | 手工转 `Math*` 树 |
| 生成后公式自检 | ⚠️ | `office_read_to_markdown` **不保留**公式结构；用 `ooxml_pack` 校验 + 请用户 Word 打开确认 |
| 极长脚本 | — | 失败后用 `path` 重跑工具返回的 `script_path` |

**与 Typst 分工**：公式少、必须 Word 交付 → 本页；公式密集、版式严 → `typst_read_template` + `typst_to_pdf`（见系统提示）。

## 校验

1. `skill_run` 返回的 `style_warnings` 若有，按 `SKILL.md` 修正。
2. `ooxml_pack` 等价校验由运行时自动完成。
3. 不要用 `office_read_to_markdown` 判断公式是否正确。
