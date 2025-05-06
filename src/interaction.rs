use embedded_graphics::prelude::*;

/// Interaction with the UI
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Interaction {
    /// A click event (mouse, touch, etc. down)
    Click(Point),
    /// A drag event (mouse, touch, etc. move while clicked)
    Drag(Point),
    /// A release event (mouse, touch, etc. up)
    Release(Point),
    /// A hover event (mouse, touch, etc. move while not clicked).
    /// Generally not applicable to touch screens.
    Hover(Point),
    /// No interaction
    #[default]
    None,
}

impl Interaction {
    /// Gets the point associated with the current interaction, if any.
    ///
    /// This method returns the point associated with the current interaction, such as the click, drag, release, or hover point. If the interaction is [Interaction::None], this method returns [None`.
    pub(crate) fn get_point(&self) -> Option<Point> {
        match self {
            Interaction::Click(p) => Some(*p),
            Interaction::Drag(p) => Some(*p),
            Interaction::Release(p) => Some(*p),
            Interaction::Hover(p) => Some(*p),
            Interaction::None => None,
        }
    }
}
