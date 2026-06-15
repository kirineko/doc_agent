#import "/doc-agent/typst/common/fonts.typ": *
#import "/doc-agent/typst/common/tokens.typ": *

#let definition-zh(title, body, theme: default-theme) = block(
  fill: theme.fill,
  inset: scaled-sp(sp-sm, theme),
  radius: 4pt,
  width: 100%,
  stroke: stroke-hair + color-rule,
)[
  #text(font: font-heading-zh, weight: "bold", fill: theme.accent)[定义（#title）] #linebreak()
  #body
]

#let example-zh(title, body, theme: default-theme) = block(
  fill: theme.fill,
  inset: scaled-sp(sp-sm, theme),
  radius: 4pt,
  width: 100%,
  stroke: stroke-hair + color-rule,
)[
  #text(font: font-heading-zh, weight: "bold", fill: theme.accent)[例 #title] #linebreak()
  #body
]

#let definition-en(title, body, theme: default-theme) = block(
  fill: theme.fill,
  inset: scaled-sp(sp-sm, theme),
  radius: 4pt,
  width: 100%,
  stroke: stroke-hair + color-rule,
)[
  #text(font: font-heading-en, weight: "bold", fill: theme.accent)[Definition (#title)] #linebreak()
  #body
]

#let example-en(title, body, theme: default-theme) = block(
  fill: theme.fill,
  inset: scaled-sp(sp-sm, theme),
  radius: 4pt,
  width: 100%,
  stroke: stroke-hair + color-rule,
)[
  #text(font: font-heading-en, weight: "bold", fill: theme.accent)[Example #title] #linebreak()
  #body
]
