//! Defines the interfaces for accessing and querying shapes.

use super::{
    geometry::{Point, Position},
    style::StyleAttr,
};

/// This is the trait that all elements that can be arranged need to implement.
pub trait Visible {
    /// \return the Position of the shape.
    fn position(&self) -> Position;
    /// \return the mutable reference to the Position of the shape.
    fn position_mut(&mut self) -> &mut Position;
    /// Return true if the element is a connector.
    fn is_connector(&self) -> bool;
    /// Swap the coordinates of the location and size.
    fn transpose(&mut self);
    /// Update the size of the shape.
    fn resize(&mut self);
}
/// This is the trait that all elements that can be rendered on a canvas need to
/// implement.
pub trait Renderable {
    /// Render the shape into a canvas.
    /// If \p debug is set then extra markers will be rendered.
    fn render(&self, debug: bool, canvas: &mut dyn RenderBackend);

    /// \Return the coordinate for the connection point of an arrow that's
    /// coming from the direction of \p from.
    /// The format of the path is (x, y, cx, cy), where cx and cy, are the
    /// control points of the bezier curve.
    /// \p force is the magnitude of the edge direction.
    /// \p port is the optional port name (for named records).
    fn get_connector_location(
        &self,
        from: Point,
        force: f64,
        port: &Option<String>,
    ) -> (Point, Point);

    /// Computes the coordinate for the connection point of an arrow that's
    /// passing through this edge.
    /// coming from the direction of \p from.
    /// \returns the bezier path in the format (x, y, cx, cy), where cx and cy,
    /// are the control points for the entry path of the bezier curve. The exit
    /// path is assumed to be the mirror point for the center (first point).
    /// \p force is the magnitude of the edge direction.
    /// This works with the get_connector_location method for drawing edges.
    fn get_passthrough_path(
        &self,
        from: Point,
        to: Point,
        force: f64,
    ) -> (Point, Point);
}

pub type ClipHandle = usize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScopeKind {
    Plain,
    Link,
}

#[derive(Clone, Debug, Default)]
pub struct RenderProperties {
    pub id: Option<String>,
    pub href: Option<String>,
    pub target: Option<String>,
    pub tooltip: Option<String>,
    pub raw: Option<String>,
}

impl RenderProperties {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn raw(raw: impl Into<String>) -> Self {
        Self {
            id: None,
            href: None,
            target: None,
            tooltip: None,
            raw: Some(raw.into()),
        }
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn with_href(mut self, href: impl Into<String>) -> Self {
        self.href = Some(href.into());
        self
    }

    pub fn with_target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }

    pub fn with_tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RectSides {
    pub left: bool,
    pub right: bool,
    pub top: bool,
    pub bottom: bool,
}

impl RectSides {
    pub fn all() -> Self {
        Self {
            left: true,
            right: true,
            top: true,
            bottom: true,
        }
    }

    pub fn none() -> Self {
        Self {
            left: false,
            right: false,
            top: false,
            bottom: false,
        }
    }

    pub fn is_all(&self) -> bool {
        self.left && self.right && self.top && self.bottom
    }

    pub fn is_none(&self) -> bool {
        !self.left && !self.right && !self.top && !self.bottom
    }
}

/// This is the trait that all rendering backends need to implement.
pub trait RenderBackend {
    /// Start a semantic render scope. Backends that support metadata can
    /// serialize the scope as links, titles, IDs, or other target-specific
    /// metadata around the draw calls emitted before `end_scope`.
    fn begin_scope(&mut self, properties: Option<RenderProperties>);

    /// End the most recent semantic render scope.
    fn end_scope(&mut self, kind: ScopeKind);

    /// Draw a rectangle. The top-left point of the rectangle is \p xy. The shape
    /// style (color, edge-width) are passed in \p look. The parameter \p clip
    /// is an optional clip region (see: create_clip). The \p sides parameter
    /// controls which rectangle border sides are stroked.
    fn draw_rect(
        &mut self,
        xy: Point,
        size: Point,
        look: &StyleAttr,
        properties: Option<RenderProperties>,
        clip: Option<ClipHandle>,
        sides: RectSides,
    );

    /// Draw a line between \p start and \p stop.
    fn draw_line(
        &mut self,
        start: Point,
        stop: Point,
        look: &StyleAttr,
        properties: Option<RenderProperties>,
    );

    /// Draw an ellipse with the center \p xy, and size \p size.
    fn draw_circle(
        &mut self,
        xy: Point,
        size: Point,
        look: &StyleAttr,
        properties: Option<RenderProperties>,
    );

    /// Draw a labe.
    fn draw_text(&mut self, xy: Point, text: &str, look: &StyleAttr);

    /// Draw an arrow, with a label, with the style parameters in \p look.
    fn draw_arrow(
        &mut self,
        path: &[(Point, Point)],
        dashed: bool,
        head: (bool, bool),
        look: &StyleAttr,
        properties: Option<RenderProperties>,
        text: &str,
    );

    /// Embeds an image in the canvas.
    fn draw_image(
        &mut self,
        xy: Point,
        size: Point,
        file_path: &str,
        properties: Option<RenderProperties>,
    );

    /// Generate a clip region that shapes can use to create complex shapes.
    fn create_clip(
        &mut self,
        xy: Point,
        size: Point,
        rounded_px: usize,
    ) -> ClipHandle;
}
