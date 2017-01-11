#![allow(non_upper_case_globals)]
extern crate rex;

use std::fs::File;
use std::io::Write;

const latin: &'static str = "abcdefghijklmnopqrstuv";
const LATIN: &'static str = "ABCDEFGHIJKLMNOPQRSTUV";
const digit: &'static str = "1234567890";
const greek: &'static str =
    "\\alpha\\beta\\gamma\\delta\\epsilon\\varepsilon\\zeta\
     \\zeta\\eta\\theta\\vartheta\\iota\\kappa\\lambda\\mu\\nu\
     \\xi\\phi\\rho\\varrho\\sigma\\tau\\upsilon\\phi\\varphi\\chi\\psi\\omega";
const GREEK: &'static str =
    "\\Alpha\\Beta\\Gamme\\Delta\\Epsilon\\Zeta\\Eta\\Theta\\Iota\\Kappa\
     \\Lambda\\Mu\\Nu\\Pi\\Rho\\Sigma\\Tau\\Upsilon\\Phi\\Chi\\Psi\\Omega";
const other: &'static str = "\\nabla\\partial";

static styles: [&'static str; 14] = [
    r"\mathrm",
    r"\mathbf",
    r"\mathit",
    r"\mathbf{\mathit",
    r"\mathscr",
    r"\mathbf{\mathscr",
    r"\mathfrak",
    r"\mathbf{\mathfrak",
    r"\mathcal",
    r"\mathsf",
    r"\mathbbsf",
    r"\mathitsf",
    r"\mathbbitsf",
    r"\mathtt",
];

const HEADER: &'static str =
r##"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Testing Things</title>
    <link rel="stylesheet" href="prism.css"/>
    <script src="prism.js"></script>
</head>
<body>
<h1><center>Font Styles Rendering Tests</center></h1>
"##;

const END: &'static str = r"</body></html>";

#[test]
fn font_styles_render() {
    let svg  = rex::SVGRenderer::new().font_src("rex-xits.woff2").font_size(32.0);
    let mut file = File::create("tests/out/font_styles.html")
        .expect("Unable to create `font_styles.html`");
    let mut result = String::from(HEADER);

    for &style in styles.iter() {
        result += &format!(
            "<h2><center>{}</center></h2>\n\
             <center>{}</center>\n\
             <center>{}</center>\n\
             <center>{}</center>\n\
             <center>{}</center>\n\
             <center>{}</center>\n\
             <center>{}</center>\n",
             style,
             svg.render(&tex(style, latin)),
             svg.render(&tex(style, LATIN)),
             svg.render(&tex(style, greek)),
             svg.render(&tex(style, GREEK)),
             svg.render(&tex(style, digit)),
             svg.render(&tex(style, other)));
    }

    result += END;
    file.write_all(&result.as_bytes())
        .expect("Unable to write to `font_styles.html`");
}

fn tex(style: &str, source: &str) -> String {
    // count the number of { and match
    let num = style.chars().filter(|&c| c == '{').count() + 1;
    let out = format!("{}{{{}{}",
        style,
        source,
        (0..num).map(|_| "}").collect::<String>());
    println!("{}", out);
    out
}