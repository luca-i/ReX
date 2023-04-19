//! Draw nodes laid out in space (as defined in the `Layout` module) onto a `Backend`, such as a screen, a PNG image, etc. 
//! 
//! To do this, a `Renderer` must first be created using the `Renderer::new` function 
//! and then `Renderer::render` must be called on the `Layout` and the desired `Backend`. 
//!
//! ## Backends
//! 
//! The `Backend` trait represents all graphical operations that are needed to render a formula: 
//!
//!   - setting colors: `GraphicsBackend::begin_color` and `GraphicsBackend::end_color`
//!   - drawing a filled rectangle: `GraphicsBackend::rule`
//!   - drawing a glyph from a given font (`FontBackend::symbol`). 
//!
//! A number of common [`Backend`] have been implemented and can be activated using some features of the crates:
//!
//!  - Cairo backend :   `cairo-renderer` (render to screen, png or svg)
//!  - FemtoVG backend : `femtovg-renderer` (render to screen using OpenGL)
//!  - Raqote backend : `raqote-renderer` (render to screen, png)
//! 
//! ## Caveat on coordinate systems
//! 
//! The top is oriented along -Y. So in particular, the Y coordinate of the position of a superscript is less than the Y coordinate of its base.
//! Glyph outlines in font files are often given with the opposite convention: the top of the glyph has the highest Y coordinate. Some adjustment needs to be made when implementing e.g. [`FontBackend`].


use crate::error::Error;
use crate::dimensions::*;
use crate::font::MathFont;
use crate::font::common::GlyphId;
use crate::layout::{LayoutNode, LayoutVariant, Alignment, LayoutSettings, Layout, Grid};
pub use crate::parser::color::RGBA;

/// Context used for rendering.
pub struct Renderer {
    /// When set to true, the renderer additionally calls [`GraphicsBackend::bbox`] to draw boxes
    /// around every glyph, horizontal and vertical boxes of the layout.
    pub debug: bool,
}

/// Position of the cursor in space. 
#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Cursor {
    /// x-coordinate
    pub x: f64,
    /// y-coordinate (NB: `cursor1.y` < `cursor2.y`  means `cursor1` is above `cursor2` on the screen)
    pub y: f64,
}

impl Cursor {
    /// Adds `dx` and `dy` to the x- and y- coordinates resp. of the cursor
    pub fn translate(self, dx: f64, dy: f64) -> Cursor {
        Cursor {
            x: self.x + dx,
            y: self.y + dy,
        }
    }

    /// Moves cursor by `dx` in the direction -X
    pub fn left(self, dx: f64) -> Cursor {
        Cursor {
            x: self.x - dx,
            y: self.y,
        }
    }
    /// Moves cursor by `dx` in the direction +X
    pub fn right(self, dx: f64) -> Cursor {
        Cursor {
            x: self.x + dx,
            y: self.y,
        }
    }
    /// Moves cursor by `dy` in the direction -Y
    pub fn up(self, dy: f64) -> Cursor {
        Cursor {
            x: self.x,
            y: self.y - dy,
        }
    }
    /// Moves cursor by `dy` in the direction +Y
    pub fn down(self, dy: f64) -> Cursor {
        Cursor {
            x: self.x,
            y: self.y + dy,
        }
    }
}

/// A backend that can draw glyphs from fonts of type `F`. One of the two traits needed to implement [`Backend`].
pub trait FontBackend<F> {
    /// Draws glyph with id `gid` at `pos` with scale `scale` with font `ctx`.  
    /// 
    /// **NB:** fonts typically provide the outline with positive Y values representing points above the baseline.
    /// ReX works with the opposite convention so drawing a symbol involves a step of transformation, namely flipping the Y-axis.
    fn symbol(&mut self, pos: Cursor, gid: GlyphId, scale: f64, ctx: &F);
}


/// A backend that can draw filled rectangles and has some support for colors. One of the two traits needed to implement [`Backend`].
///
/// Implementing the function [`GraphicsBackend::bbox`] is optional (if not implemented, this function does nothing).
/// This function is only used in the debug mode of [`Renderer`] to draw rectangles around glyphs and layout boxes.
pub trait GraphicsBackend {
    /// Only called by [`Renderer`] when [`Renderer::debug`] is true (debug mode). 
    /// Draws a rectangle whose top-left corner is at `_pos` with the dimensions specified by `_width` and `_height`
    /// This function is closed in debug mode to show the bound box of various object.
    /// The parameter `_role` specifies the type of objects that the rectanlge encloses: a glyph, a vertical box or a horizontal box.
    /// One can use this parameter to style the rectangles differently, e.g. red for glyph bounding boxs, green for vertical boxes, etc.
    fn bbox(&mut self, _pos: Cursor, _width: f64, _height: f64, _role: Role) {}
    /// Draws a filled rectangle whose top-left corner is at `pos`. Used to draw fraction bars and radicals.
    fn rule(&mut self, pos: Cursor, width: f64, height: f64);
    /// Makes `color` the current used color. The color previously in use is restored with [`GraphicsBackend::end_color`].
    fn begin_color(&mut self, color: RGBA);
    /// Restores the previously used color. If there were no previous color, this function should return silently and not panic.
    fn end_color(&mut self);
}

/// A conjunction of the font-specific draw commands of [`FontBackend`] and the general draw commands [`GraphicsBackend`]
/// This is the trait that needs to be implemented for something to be a backend.
pub trait Backend<F> : FontBackend<F> + GraphicsBackend {
}


/// The type of things enclosed by a debug rectangle (cf [`Renderer::debug`] for debug mode).
pub enum Role {
    /// glyph
    Glyph,
    /// vertical box
    VBox,
    /// horizontal box
    HBox,
}

impl Renderer {
    /// Creates new renderer.
    pub fn new() -> Self {
        Renderer {
            debug: false,
        }
    }

    /// Parses and lays out the given string
    pub fn layout<'s, 'a, 'f, F : MathFont>(&self, tex: &'s str, layout_settings: LayoutSettings<'a, 'f, F>) -> Result<Layout<'f, F>, Error<'s>> {
        use crate::parser::parse;
        use crate::layout::engine::layout;

        let mut parse = parse(tex)?;
        Ok(layout(&mut parse, layout_settings)?)
    }
    /// Bounding box for the given layout. Returns (x_min, y_min, x_max, y_max) the minimal and maximal for both coordinates.
    pub fn size<F>(&self, layout: &Layout<F>) -> (f64, f64, f64, f64) {
        // TODO: why the dependency on renderer
        // TODO: is there truly nothing before 0?
        (
            0.0,
            layout.depth / Px,
            layout.width / Px,
            layout.height / Px
        )
    }

    /// Renders the given layout onto `out` the provided backend.
    pub fn render<F>(&self, layout: &Layout<F>, out: &mut impl Backend<F>) {
        let pos = Cursor {
            x: 0.0,
            y: 0.0,
        };
        self.render_hbox(out, pos, &layout.contents, layout.height / Px, layout.width / Px, Alignment::Default);
    }

    fn render_grid<F>(&self, out: &mut impl Backend<F>, pos: Cursor, width: f64, height: f64, grid: &Grid<F>) {
        let x_offsets = grid.x_offsets();
        let y_offsets = grid.y_offsets();
        for (&(row, column), node) in grid.contents.iter() {
            let width = grid.columns[column];
            let (height, depth) = grid.rows[row];

            self.render_node(
                out,
                pos.translate(x_offsets[column] / Px, (y_offsets[row] + height) / Px),
                node
            );
        }
    }

    fn render_hbox<F>(&self, out: &mut impl Backend<F>, mut pos: Cursor, nodes: &[LayoutNode<F>], height: f64, nodes_width: f64, alignment: Alignment) {
        if self.debug {
            out.bbox(pos.up(height), nodes_width, height, Role::HBox);
        }
        if let Alignment::Centered(w) = alignment {
            pos.x += (nodes_width - w / Px) * 0.5;
        }

        for node in nodes {
            self.render_node(out, pos, node);

            pos.x += node.width / Px;
        }
    }
    fn render_vbox<F>(&self, out: &mut impl Backend<F>, mut pos: Cursor, nodes: &[LayoutNode<F>]) {
        for node in nodes {
            match node.node {
                LayoutVariant::Rule => out.rule(pos, node.width / Px, node.height / Px),
                LayoutVariant::Grid(ref grid) => self.render_grid(out, pos, node.height / Px, node.width / Px, grid),
                LayoutVariant::HorizontalBox(ref hbox) => {
                    self.render_hbox(out,
                                     pos.down(node.height / Px),
                                     &hbox.contents,
                                     node.height / Px,
                                     node.width / Px,
                                     hbox.alignment)
                }

                LayoutVariant::VerticalBox(ref vbox) => {
                    if self.debug {
                        out.bbox(pos, node.width / Px, (node.height - node.depth) / Px, Role::VBox);
                    }
                    self.render_vbox(out, pos, &vbox.contents);
                }

                LayoutVariant::Glyph(ref gly) => {
                    if self.debug {
                        out.bbox(pos, node.width / Px, (node.height - node.depth) / Px, Role::Glyph);
                    }
                    out.symbol(pos.down(node.height / Px), gly.gid, gly.size / Px, gly.font);
                }

                LayoutVariant::Color(_) => panic!("Shouldn't have a color in a vertical box???"),

                LayoutVariant::Kern => { /* NOOP */ }
            }

            pos.y += node.height / Px;
        }
    }

    fn render_node<'a, F>(&self, out: &mut impl Backend<F>, pos: Cursor, node: &LayoutNode<'a, F>) {
        match node.node {
            LayoutVariant::Glyph(ref gly) => {
                if self.debug {
                    out.bbox(pos.up(node.height / Px), node.width / Px, (node.height - node.depth) / Px, Role::Glyph);
                }
                out.symbol(pos, gly.gid, gly.size / Px, gly.font);
            }

            LayoutVariant::Rule => out.rule(pos.up(node.height / Px), node.width / Px, node.height / Px),

            LayoutVariant::VerticalBox(ref vbox) => {
                if self.debug {
                    out.bbox(pos.up(node.height / Px), node.width / Px, (node.height - node.depth) / Px, Role::VBox);
                }
                self.render_vbox(out, pos.up(node.height / Px), &vbox.contents);
            }

            LayoutVariant::HorizontalBox(ref hbox) => {
                self.render_hbox(out, pos, &hbox.contents, node.height / Px, node.width / Px, hbox.alignment);
            }
            LayoutVariant::Grid(ref grid) => self.render_grid(out, pos, node.height / Px, node.width / Px, grid),

            LayoutVariant::Color(ref clr) => {
                out.begin_color(clr.color);
                self.render_hbox(out, pos, &clr.inner, node.height / Px, node.width / Px, Alignment::Default);
                out.end_color();
            }

            LayoutVariant::Kern => { /* NOOP */ }
        } // End macth

    }
}

#[cfg(feature="pathfinder-renderer")]
pub mod pathfinder;
#[cfg(feature="femtovg-renderer")]
pub mod femtovg;
#[cfg(feature="cairo-renderer")]
pub mod cairo;
#[cfg(feature="raqote-renderer")]
pub mod raqote;
