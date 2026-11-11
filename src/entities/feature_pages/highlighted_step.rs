//! Entity: a step split into the parts its highlighting tells apart, each with its text as written.

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HighlightedStep {
    /// As written, such as `Given`, `And`, `*` or `Dado`, without the space after it.
    pub(crate) keyword: String,
    /// The step's text after its keyword, in order; joined, they are that text unchanged.
    pub(crate) parts: Vec<StepPart>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum StepPart {
    Plain(String),
    Number(String),
    /// With its angle brackets, such as `<amount>`.
    Placeholder(String),
    /// With its quotes; inside, only [`StepPart::Plain`] and [`StepPart::Placeholder`].
    Parameter(Vec<StepPart>),
}

impl StepPart {
    pub(crate) fn text(&self) -> String {
        match self {
            StepPart::Plain(text) | StepPart::Number(text) | StepPart::Placeholder(text) => {
                text.clone()
            }
            StepPart::Parameter(parts) => parts.iter().map(StepPart::text).collect(),
        }
    }
}
