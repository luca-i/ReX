use dimensions::{Pixels, Float};
use layout::{LayoutNode, LayoutVariant, Alignment, Style, LayoutSettings};
use parser::parse;
use layout::engine::layout;

#[derive(Clone)]
pub struct RenderSettings {
    pub font_size:    Float,
    pub font_src:     String,
    pub horz_padding: Float,
    pub vert_padding: Float,
    pub strict:       bool,
    pub style:        Style,
    pub debug:        bool
}

impl Default for RenderSettings {
    fn default() -> Self {
        RenderSettings {
            font_size:    48.0,
            font_src:     "http://rex.breeden.cc/rex-xits.otf".into(),
            horz_padding: 12.0,
            vert_padding: 5.0,
            strict:       true,
            style:        Style::Display,
            debug:        false
        }
    }
}

impl RenderSettings {
    pub fn font_size(self, size: Float) -> Self {
        RenderSettings {
            font_size: size,
            ..self
        }
    }
    
    pub fn font_src(self, src: &str) -> Self {
        RenderSettings {
            font_src: src.into(),
            ..self
        }
    }
    
    pub fn horz_padding(self, size: Float) -> RenderSettings {
        RenderSettings {
            horz_padding: size,
            ..self
        }
    }

    pub fn vert_padding(self, size: Float) -> RenderSettings {
        RenderSettings {
            vert_padding: size,
            ..self
        }
    }

    pub fn style(self, style: Style) -> RenderSettings {
        RenderSettings {
            style: style,
            ..self
        }
    }

    pub fn debug(self, debug: bool) -> RenderSettings {
        RenderSettings {
            debug: debug,
            ..self
        }
    }
    
    pub fn layout_settings(&self) -> LayoutSettings {
        LayoutSettings {
            font_size: self.font_size,
            style:     self.style
        }
    }
}

pub trait Renderer {
    type Out;

    fn g<F>(&self, out: &mut Self::Out, off_x: Pixels, off_y: Pixels, contents: F)
    where F: FnMut(&Self, &mut Self::Out);

    fn bbox(&self, _out: &mut Self::Out, _width: Pixels, _height: Pixels) {}

    fn symbol(&self, out: &mut Self::Out, symbol: u32, scale: Float);
    
    fn rule(&self, out: &mut Self::Out, x: Pixels, y: Pixels, width: Pixels, height: Pixels);

    fn color<F>(&self, out: &mut Self::Out, color: &str, contents: F)
    where F: FnMut(&Self, &mut Self::Out);
    
    fn render_hbox(&self, out: &mut Self::Out,
        nodes: &[LayoutNode],
        height: Pixels,
        nodes_width: Pixels,
        alignment: Alignment)
    {
        let mut width = Pixels(0.0);

        if let Alignment::Centered(w) = alignment {
            width += (nodes_width - w)/2.0;
        }

        self.bbox(out, nodes_width, height);

        for node in nodes {
            match node.node {
                LayoutVariant::Glyph(ref gly) =>
                    self.g(out, width, height,
                        |r, out| r.symbol(out, gly.unicode, gly.scale)
                    ),

                LayoutVariant::Rule =>
                    self.rule(out,
                        width, height - node.height,
                        node.width, node.height
                    ),

                LayoutVariant::VerticalBox(ref vbox) =>
                    self.g(out, width, height - node.height,
                        |r, out| r.render_vbox(out, &vbox.contents)
                    ),

                LayoutVariant::HorizontalBox(ref hbox) =>
                    self.g(out, width, height - node.height, |r, out| {
                        r.render_hbox(out,
                            &hbox.contents, node.height,
                            node.width, hbox.alignment
                        )
                    }),

                LayoutVariant::Color(ref clr) =>
                    self.color(out, &clr.color, |r, out| {
                        r.render_hbox(out, &clr.inner,
                            node.height, node.width, Alignment::Default
                        );
                    }),

                LayoutVariant::Kern => { }
            } // End macth

            width += node.width;
        }
    }

    fn render_vbox(&self, out: &mut Self::Out, nodes: &[LayoutNode]) {
        let mut height = Pixels(0.0);
        let width      = Pixels(0.0);

        for node in nodes {
            match node.node {
                LayoutVariant::Rule =>
                    self.rule(out, width, height, node.width, node.height),

                LayoutVariant::HorizontalBox(ref hbox) =>
                    self.g(out, width, height, |r, out| {
                        r.render_hbox(out,
                            &hbox.contents, node.height,
                            node.width, hbox.alignment
                        )
                    }),

                LayoutVariant::VerticalBox(ref vbox) =>
                    self.g(out, width, height, |r, out| r.render_vbox(out, &vbox.contents)),

                LayoutVariant::Glyph(ref gly) =>
                    self.g(out, width, height + node.height, |r, out| {
                        r.symbol(out, gly.unicode, gly.scale)
                    }),

                LayoutVariant::Color(_) =>
                    panic!("Shouldn't have a color in a vertical box???"),

                LayoutVariant::Kern => { }
            }

            height += node.height;
        }
    }
    
    fn prepare(&self, _out: &mut Self::Out, _width: Pixels, _height: Pixels) {}
    fn finish(&self, _out: &mut Self::Out) {}
    fn settings(&self) -> &RenderSettings;
    
    fn render_to(&self, out: &mut Self::Out, tex: &str) {
        let mut parse = match parse(&tex) {
                Ok(res)  => res,
                Err(err) => {
                    println!("Error -- {}", err);
                    return;
                }
            };

        let layout = layout(&mut parse, self.settings().layout_settings());

        if self.settings().debug {
            println!("Parse: {:?}\n", parse);
            println!("Layout: {:?}", layout);
        }
        
        let padding = (
            self.settings().horz_padding,
            self.settings().vert_padding
        );
        
        self.prepare(out,
            // Left and right padding
            layout.width  + 2.0 * padding.0,
            // Top and bot padding
            layout.height + 2.0 * padding.1 - layout.depth
        );

        let x = Pixels(padding.0);
        let y = Pixels(padding.1);
        self.g(out, x, y, |r, out| {
            r.render_hbox(out,
                &layout.contents, layout.height,
                layout.width, Alignment::Default
            )
        });
        
        self.finish(out);
    }
    
    fn render(&self, tex: &str) -> Self::Out where Self::Out: Default {
        let mut out = Self::Out::default();
        self.render_to(&mut out, tex);
        out
    }
}

pub mod svg;
pub use self::svg::SVGRenderer;
