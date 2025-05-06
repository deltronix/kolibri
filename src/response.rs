use crate::interaction::Interaction;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GuiError {
    /// The widget is too large to fit in the bounds with the current constraints
    NoSpaceLeft,
    /// The Drawable returned an error while drawing
    // TODO: (maybe) add better error handling here
    // The rationale for the 'static str is that generics are annoying to implement,
    // and that generic would need to be everywhere, basically, as returning just () as an
    // error would make handling wierd and complicated.
    // The goal of this library is to be trivially easy, not to be 100% generic.
    // If you have a better idea, a PR is much appreciated.
    // (maybe a Box<dyn Error> with alloc feature gate? Or a 'String' (heapless / alloc) and format!()?)
    DrawError(Option<&'static str>),

    /// The requested operation would cause the bounds to be different from the expected size
    BoundsError,
}

impl GuiError {
    pub fn draw_error(msg: &'static str) -> Self {
        GuiError::DrawError(Some(msg))
    }
}

pub type GuiResult<T> = Result<T, GuiError>;

pub struct InternalResponse {
    pub area: Rectangle,
    pub interaction: Interaction,
}

impl InternalResponse {
    pub fn new(area: Rectangle, interaction: Interaction) -> Self {
        Self { area, interaction }
    }

    pub fn empty() -> Self {
        Self {
            area: Rectangle::new(Point::zero(), Size::zero()),
            interaction: Interaction::None,
        }
    }
}

/// Response for UI interaction / space allocation and such
pub struct Response {
    pub internal: InternalResponse,
    /// Whether the widget was clicked (as in successfully interacted with)
    pub click: bool,

    /// Whether the widget is in a "down" state (e.g. a button is pressed, but not yet released)
    ///
    /// Can be used to do things while a button is held down
    pub down: bool,

    /// Marker to tell the UI that this widget was redrawn this frame (if you don't have redraw
    /// / change detection, just set this to `true`, as you are redrawing every frame)
    ///
    /// **The default for this is `true`**.
    pub redraw: bool,

    /// What the underlying data changed?
    ///
    /// e.g. the slider was dragged, etc.
    /// Always `false` for something like a [`Button`](crate::button::Button).
    pub changed: bool,

    /// Whether the widget had an error while drawing
    pub error: Option<GuiError>,
}

// builder pattern
impl Response {
    pub fn new(raw: InternalResponse) -> Response {
        Response {
            internal: raw,
            click: false,
            redraw: true,
            changed: false,
            down: false,
            error: None,
        }
    }

    pub fn from_error(error: GuiError) -> Response {
        Response::new(InternalResponse::empty()).set_error(error)
    }

    pub fn set_clicked(mut self, clicked: bool) -> Self {
        self.click = clicked;
        self
    }

    pub fn set_redraw(mut self, redraw: bool) -> Self {
        self.redraw = redraw;
        self
    }

    pub fn set_changed(mut self, changed: bool) -> Self {
        self.changed = changed;
        self
    }

    pub fn set_error(mut self, error: GuiError) -> Self {
        self.error = Some(error);
        self
    }

    pub fn set_down(mut self, down: bool) -> Self {
        self.down = down;
        self
    }

    /// Check whether the widget was clicked (as in successfully interacted with)
    pub fn clicked(&self) -> bool {
        self.click
    }

    /// Check whether the widget is in a "down" state (e.g. a button is pressed, but not yet released)
    ///
    /// Can be used to do things while a button is held down
    pub fn down(&self) -> bool {
        self.down
    }

    /// Check whether the widget was redrawn this frame
    pub fn redrawn(&self) -> bool {
        self.redraw
    }

    /// Check whether the underlying data changed (e.g. slider was moved)
    pub fn changed(&self) -> bool {
        self.changed
    }

    /// Check whether the widget had an error while drawing
    /// (e.g. the underlying draw target returned an error), no space was left, ...
    pub fn error(&self) -> Option<GuiError> {
        self.error
    }
}
