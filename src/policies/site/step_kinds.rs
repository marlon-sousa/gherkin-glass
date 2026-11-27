//! Policy: the kind of a step, Given, When or Then, as the `gherkin` crate reads it, which an `And` or `But` takes from the step before it and `*` is Given.

use gherkin::StepType;

use crate::entities::StepKind;

pub(crate) fn step_kind(ty: StepType) -> StepKind {
    match ty {
        StepType::Given => StepKind::Given,
        StepType::When => StepKind::When,
        StepType::Then => StepKind::Then,
    }
}
