#import "/doc-agent/typst/common/fonts.typ": *

#let page-a4(margin: 2.5cm) = {
  set page(paper: "a4", margin: margin)
}

#let page-a4-compact(margin: 2cm) = {
  set page(paper: "a4", margin: margin)
}

#let page-a4-landscape(margin: 2cm) = {
  set page(paper: "a4", margin: margin, flipped: true)
}

#let page-letter(margin: 1in) = {
  set page(paper: "us-letter", margin: margin)
}

#let footer-page-no() = {
  set page(footer: context {
    align(center)[#counter(page).display("1")]
  })
}
