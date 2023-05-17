use crate::config::ConfigOption;
use crate::linter::{check_regex, SyntaxRule, SyntaxRuleResult};
use regex::Regex;
use sv_parser::{unwrap_node, NodeEvent, RefNode, SyntaxTree};

#[derive(Default)]
pub struct ReRequiredModport {
    re: Option<Regex>,
}

impl SyntaxRule for ReRequiredModport {
    fn check(
        &mut self,
        syntax_tree: &SyntaxTree,
        event: &NodeEvent,
        option: &ConfigOption,
    ) -> SyntaxRuleResult {
        if self.re.is_none() {
            self.re = Some(Regex::new(&option.re_required_modport).unwrap());
        }

        let node = match event {
            NodeEvent::Enter(x) => x,
            NodeEvent::Leave(_) => {
                return SyntaxRuleResult::Pass;
            }
        };

        match node {
            RefNode::ModportDeclaration(x) => {
                check_regex(true, unwrap_node!(*x, ModportIdentifier),
                            &syntax_tree, &self.re.as_ref().unwrap())
            }
            _ => SyntaxRuleResult::Pass,
        }
    }

    fn name(&self) -> String {
        String::from("re_required_modport")
    }

    fn hint(&self, option: &ConfigOption) -> String {
        String::from(format!(
            "Use a modport identifier matching regex `{}`.",
            &option.re_required_modport
        ))
    }

    fn reason(&self) -> String {
        String::from("Identifiers must conform to the naming scheme.")
    }
}
