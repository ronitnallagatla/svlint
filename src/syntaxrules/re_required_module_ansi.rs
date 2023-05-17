use crate::config::ConfigOption;
use crate::linter::{check_regex, SyntaxRule, SyntaxRuleResult};
use regex::Regex;
use sv_parser::{unwrap_node, NodeEvent, RefNode, SyntaxTree};

#[derive(Default)]
pub struct ReRequiredModuleAnsi {
    re: Option<Regex>,
}

impl SyntaxRule for ReRequiredModuleAnsi {
    fn check(
        &mut self,
        syntax_tree: &SyntaxTree,
        event: &NodeEvent,
        option: &ConfigOption,
    ) -> SyntaxRuleResult {
        if self.re.is_none() {
            self.re = Some(Regex::new(&option.re_required_module_ansi).unwrap());
        }

        let node = match event {
            NodeEvent::Enter(x) => x,
            NodeEvent::Leave(_) => {
                return SyntaxRuleResult::Pass;
            }
        };

        match node {
            RefNode::ModuleAnsiHeader(x) => {
                check_regex(true, unwrap_node!(*x, ModuleIdentifier),
                            &syntax_tree, &self.re.as_ref().unwrap())
            }
            _ => SyntaxRuleResult::Pass,
        }
    }

    fn name(&self) -> String {
        String::from("re_required_module_ansi")
    }

    fn hint(&self, option: &ConfigOption) -> String {
        String::from(format!(
            "Use a module identifier matching regex `{}`.",
            &option.re_required_module_ansi
        ))
    }

    fn reason(&self) -> String {
        String::from("Identifiers must conform to the naming scheme.")
    }
}
