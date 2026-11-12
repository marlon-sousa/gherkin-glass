//! Policy: the steps of a page's blocks, with their data tables and doc strings, as written or filled with an example's values.

use gherkin::{Scenario, Step, StepType, Table};

use crate::entities::{DocStringView, ParsedFeature, StepContent, StepKind, StepView, TableView};
use crate::policies::site::feature_pages::{ExampleRow, highlight_step};

/// `scenario` holds the steps, or is `None` for a background's.
pub(super) fn step_contents(
    parsed: &ParsedFeature,
    steps: &[Step],
    scenario: Option<&Scenario>,
) -> Vec<StepContent> {
    steps
        .iter()
        .map(|step| step_content(parsed, step, scenario))
        .collect()
}

fn step_content(parsed: &ParsedFeature, step: &Step, scenario: Option<&Scenario>) -> StepContent {
    StepContent {
        step: StepView {
            keyword: step.keyword.clone(),
            parts: highlight_step(step, scenario).parts,
            kind: kind_of(step.ty),
            line: step.position.line,
        },
        table: step.table.as_ref().map(table),
        doc_string: doc_string_view(parsed, step),
    }
}

fn doc_string_view(parsed: &ParsedFeature, step: &Step) -> Option<DocStringView> {
    Some(DocStringView {
        media_type: parsed.media_type(step).map(str::to_string),
        content: step.docstring.clone()?,
    })
}

/// A produced scenario holds no placeholders, so none is highlighted.
pub(super) fn filled_step_contents(
    parsed: &ParsedFeature,
    steps: &[Step],
    row: &ExampleRow<'_>,
) -> Vec<StepContent> {
    steps
        .iter()
        .map(|step| filled_step(parsed, step, row))
        .collect()
}

fn filled_step(parsed: &ParsedFeature, step: &Step, row: &ExampleRow<'_>) -> StepContent {
    let filled = Step {
        value: row.fill(&step.value),
        ..step.clone()
    };
    StepContent {
        step: StepView {
            keyword: filled.keyword.clone(),
            parts: highlight_step(&filled, None).parts,
            kind: kind_of(step.ty),
            line: step.position.line,
        },
        table: step
            .table
            .as_ref()
            .map(|written| filled_table(written, row)),
        doc_string: filled_doc_string(parsed, step, row),
    }
}

/// The step's doc string with the row's values in place of its placeholders.
fn filled_doc_string(
    parsed: &ParsedFeature,
    step: &Step,
    row: &ExampleRow<'_>,
) -> Option<DocStringView> {
    Some(DocStringView {
        media_type: parsed.media_type(step).map(|text| row.fill(text)),
        content: row.fill(step.docstring.as_ref()?),
    })
}

/// An outline's data table with the row's values in place of its placeholders.
fn filled_table(written: &Table, row: &ExampleRow<'_>) -> TableView {
    let view = table(written);
    TableView {
        header: filled_row(&view.header, row),
        rows: view
            .rows
            .iter()
            .map(|cells| filled_row(cells, row))
            .collect(),
    }
}

fn filled_row(cells: &[String], row: &ExampleRow<'_>) -> Vec<String> {
    cells.iter().map(|cell| row.fill(cell)).collect()
}

/// The first row is the header; a table has at least one row.
pub(super) fn table(table: &Table) -> TableView {
    let mut rows = table.rows.iter().cloned();
    TableView {
        header: rows.next().unwrap_or_default(),
        rows: rows.collect(),
    }
}

fn kind_of(ty: StepType) -> StepKind {
    match ty {
        StepType::Given => StepKind::Given,
        StepType::When => StepKind::When,
        StepType::Then => StepKind::Then,
    }
}
