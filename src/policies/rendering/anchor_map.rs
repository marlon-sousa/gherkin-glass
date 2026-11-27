//! Policy: the map of anchors of a site, a version or a preview, `assets/anchors.json`, whose form is promised to other programs, written from its store: an entry for each block of each feature file, with its file's path, its line, its page and its anchor.

use serde::Serialize;

use crate::entities::{
    BlockContent, BlockKind, DescriptorFeature, ObjectBlock, ObjectExamples, ObjectRun, SiteStore,
};
use crate::policies::rendering::json_bytes;

#[derive(Serialize)]
struct EntryData<'a> {
    path: &'a str,
    kind: &'static str,
    name: &'a str,
    line: usize,
    page: &'a str,
    anchor: &'a str,
    /// `None` for every kind but an examples block.
    rows: Option<Vec<RowData<'a>>>,
}

#[derive(Serialize)]
struct RowData<'a> {
    line: usize,
    anchor: &'a str,
}

/// The compact JSON of the map of anchors of `store`: its features in their order, each
/// followed by its blocks, depth first in file order, each outline by its examples blocks.
pub(crate) fn anchor_map_json(store: &SiteStore) -> Vec<u8> {
    let entries: Vec<EntryData<'_>> = store.features.iter().flat_map(feature_entries).collect();
    json_bytes(&entries)
}

/// The entries of one feature file, which take its lines and its rows' lines in turn.
struct FeatureWalk<'a> {
    feature: &'a DescriptorFeature,
    lines: std::slice::Iter<'a, usize>,
    row_lines: std::slice::Iter<'a, usize>,
    entries: Vec<EntryData<'a>>,
}

fn feature_entries(feature: &DescriptorFeature) -> Vec<EntryData<'_>> {
    let mut walk = FeatureWalk {
        feature,
        lines: feature.lines.iter(),
        row_lines: feature.row_lines.iter(),
        entries: Vec::new(),
    };
    let object = &feature.object;
    walk.push(BlockKind::Feature, &object.text.name, "", None);
    object.blocks.iter().for_each(|block| walk.block(block));
    walk.entries
}

impl<'a> FeatureWalk<'a> {
    fn block(&mut self, block: &'a ObjectBlock) {
        self.push(block.kind(), &block.text.name, &block.anchor, None);
        match &block.content {
            BlockContent::Rule { blocks } => blocks.iter().for_each(|inner| self.block(inner)),
            BlockContent::Outline { examples, .. } => {
                examples.iter().for_each(|examples| self.examples(examples));
            }
            BlockContent::Background { .. } | BlockContent::Scenario { .. } => {}
        }
    }

    fn examples(&mut self, examples: &'a ObjectExamples) {
        let rows = examples.runs.iter().map(|run| self.row(run)).collect();
        self.push(
            BlockKind::Examples,
            &examples.text.name,
            &examples.anchor,
            Some(rows),
        );
    }

    fn push(
        &mut self,
        kind: BlockKind,
        name: &'a str,
        anchor: &'a str,
        rows: Option<Vec<RowData<'a>>>,
    ) {
        let line = self.next_line();
        self.entries.push(EntryData {
            path: self.feature.path.as_str(),
            kind: kind.as_str(),
            name,
            line,
            page: &self.feature.page,
            anchor,
            rows,
        });
    }

    // The store is built with one line for each block and one for each row, in the order this
    // walk takes them, so neither runs out.
    fn next_line(&mut self) -> usize {
        self.lines.next().copied().unwrap_or_default()
    }

    fn row(&mut self, run: &'a ObjectRun) -> RowData<'a> {
        RowData {
            line: self.row_lines.next().copied().unwrap_or_default(),
            anchor: &run.anchor,
        }
    }
}

#[cfg(test)]
mod tests;
