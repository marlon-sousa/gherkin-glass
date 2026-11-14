//! Entity: a feature page put in its folder by the site tree's builder, its blocks below it, a rule's background and scenarios in the rule, the scenarios outside every rule in their block when the feature has a rule, an outline's examples and examples as scenarios in the outline, each step in its block, and a step's table or doc string in the step, or copied from the last site tree with them.

use ego_tree::{NodeId, NodeMut, NodeRef};

use crate::entities::{
    BackgroundContent, ExampleScenario, ExamplesAsScenarios, PageContent, RuleContent,
    ScenarioContent, SiteNode, StepContent,
};

/// The page of `content` put in `folder`, its blocks below it; the page node's id.
pub(super) fn place_feature_page(
    folder: &mut NodeMut<'_, SiteNode>,
    content: PageContent,
) -> NodeId {
    let mut page = folder.append(SiteNode::FeaturePage(content.page));
    if let Some(background) = content.background {
        place_background(&mut page, background);
    }
    match content.outside_rules {
        Some(heading) => place_scenarios(
            &mut page.append(SiteNode::OutsideRules(heading)),
            content.scenarios,
        ),
        None => place_scenarios(&mut page, content.scenarios),
    }
    for rule in content.rules {
        place_rule(&mut page, rule);
    }
    page.id()
}

/// `page`, a feature page of another site tree, put in `folder` with every node below it, which
/// that tree's builder placed as `place_feature_page` does; the page node's id.
pub(super) fn copy_feature_page(
    folder: &mut NodeMut<'_, SiteNode>,
    page: NodeRef<'_, SiteNode>,
) -> NodeId {
    let mut copy = folder.append(page.value().clone());
    copy_children(&mut copy, page);
    copy.id()
}

/// The nodes below `node`, put below `copy`, in their order.
fn copy_children(copy: &mut NodeMut<'_, SiteNode>, node: NodeRef<'_, SiteNode>) {
    for child in node.children() {
        let mut child_copy = copy.append(child.value().clone());
        copy_children(&mut child_copy, child);
    }
}

/// `background` put in `parent`, a feature page or a rule, its steps below it.
fn place_background(parent: &mut NodeMut<'_, SiteNode>, background: BackgroundContent) {
    let mut node = parent.append(SiteNode::Background(background.block));
    place_steps(&mut node, background.steps);
}

/// `rule` put in `page`, its background and its scenarios below it.
fn place_rule(page: &mut NodeMut<'_, SiteNode>, rule: RuleContent) {
    let mut node = page.append(SiteNode::Rule(rule.block));
    if let Some(background) = rule.background {
        place_background(&mut node, background);
    }
    place_scenarios(&mut node, rule.scenarios);
}

/// `scenarios` put in `parent`, in their order.
fn place_scenarios(parent: &mut NodeMut<'_, SiteNode>, scenarios: Vec<ScenarioContent>) {
    for scenario in scenarios {
        place_scenario(parent, scenario);
    }
}

/// `scenario` put in `parent`, a feature page, a rule or the block of the
/// scenarios outside every rule: its steps below it, then an outline's
/// examples blocks and its examples as scenarios.
fn place_scenario(parent: &mut NodeMut<'_, SiteNode>, scenario: ScenarioContent) {
    let mut node = parent.append(SiteNode::Scenario(scenario.block));
    place_steps(&mut node, scenario.steps);
    for examples in scenario.examples {
        node.append(SiteNode::Examples(examples));
    }
    if let Some(as_scenarios) = scenario.as_scenarios {
        place_examples_as_scenarios(&mut node, as_scenarios);
    }
}

fn place_examples_as_scenarios(
    outline: &mut NodeMut<'_, SiteNode>,
    as_scenarios: ExamplesAsScenarios,
) {
    let mut node = outline.append(SiteNode::ExamplesAsScenarios(as_scenarios.heading));
    for ExampleScenario { heading, steps } in as_scenarios.scenarios {
        let mut scenario = node.append(SiteNode::ExampleScenario(heading));
        place_steps(&mut scenario, steps);
    }
}

/// `steps` put in `block`, each with its data table or its doc string below it.
fn place_steps(block: &mut NodeMut<'_, SiteNode>, steps: Vec<StepContent>) {
    for StepContent {
        step,
        table,
        doc_string,
    } in steps
    {
        let mut node = block.append(SiteNode::Step(step));
        if let Some(table) = table {
            node.append(SiteNode::StepTable(table));
        }
        if let Some(doc_string) = doc_string {
            node.append(SiteNode::DocString(doc_string));
        }
    }
}
