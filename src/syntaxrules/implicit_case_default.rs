use std::collections::{HashMap, HashSet};
use crate::config::ConfigOption;
use crate::linter::{SyntaxRule, SyntaxRuleResult};
use sv_parser::{unwrap_locate, unwrap_node, Locate, NodeEvent, RefNode, SyntaxTree};

#[derive(Default)]
pub struct ImplicitCaseDefault {
    under_always_construct: bool,
    under_case_item: bool,
    under_case_default: bool,
    has_default: bool,
    case_default_vars: Vec<String>,
    lhs_variables: Vec<String>,

    locate_vars: HashMap<String, Locate>,
    lhs_default_vars : HashSet<String>,
    case_item_vars : HashSet<String>,
}

impl SyntaxRule for ImplicitCaseDefault {
    fn check(
        &mut self,
        syntax_tree: &SyntaxTree,
        event: &NodeEvent,
        _option: &ConfigOption,
    ) -> SyntaxRuleResult {
        let node = match event {
            NodeEvent::Enter(x) => {
                match x {
                    RefNode::AlwaysConstruct(_) => {
                        self.under_always_construct = true;
                        self.has_default = false;
                    }

                    RefNode::CaseItemNondefault(_) => {
                        self.under_case_item = true;
                    }

                    RefNode::CaseItemDefault(_) => {
                        self.under_case_default = true;
                    }

                    _ => (),
                }
                x
            }

            NodeEvent::Leave(x) => {
                match x {
                    RefNode::AlwaysConstruct(_) => {
                        println!("{:?}", self.lhs_default_vars);
                        println!("{:?}", self.case_item_vars);
                        println!("{:?}", self.locate_vars);
                        println!("{}", self.case_item_vars.is_subset(&self.lhs_default_vars));
                        println!("{:?}", self.case_item_vars.difference(&self.lhs_default_vars));
                        for var in self.case_item_vars.difference(&self.lhs_default_vars) {
                            //println!("{:?}", self.locate_vars.get(var).unwrap());
                            let loc = self.locate_vars.get(var).unwrap();
                        }
                        println!("----------------\n");

                        self.lhs_default_vars.clear();
                        self.case_item_vars.clear();
                        self.locate_vars.clear();

                        self.under_always_construct = false;
                        self.has_default = false;
                        self.lhs_variables.clear();
                        self.case_default_vars.clear();
                    }

                    RefNode::CaseItemNondefault(_) => {
                        self.under_case_item = false;
                    }

                    RefNode::CaseItemDefault(_) => {
                        self.under_case_default = false;
                    }

                    _ => (),
                }
                return SyntaxRuleResult::Pass;
            }
        };

        // if has implicit vars, collect all implicit declarations
        if let (true, false, RefNode::BlockItemDeclaration(x)) =
            (self.under_always_construct, self.under_case_item, node)
        {
            let var = unwrap_node!(*x, VariableDeclAssignment).unwrap();
            let id = get_identifier(var, syntax_tree);
            self.lhs_default_vars.insert(id);
        }

        // if has default case, collect all explicit default vars
        match (self.under_always_construct, self.under_case_default, node) 
        {
            (true, true, RefNode::BlockingAssignment(x)) => {
                let var = unwrap_node!(*x, VariableLvalueIdentifier);
                if var.is_some() {
                    let id = get_identifier(var.unwrap(), syntax_tree);
                    self.lhs_default_vars.insert(id.clone());
                }
            }

            (true, true, RefNode::BlockItemDeclaration(x)) => {
                let var = unwrap_node!(*x, VariableDeclAssignment);
                if var.is_some() {
                    let id = get_identifier(var.unwrap(), syntax_tree);
                    self.lhs_default_vars.insert(id.clone());
                }
            }

            _ => ()
        }

        // if a case item, collect all variable declarations
        match (
            self.under_always_construct,
            self.under_case_item,
            !self.has_default || !self.case_default_vars.is_empty(),
            node,
        ) {
            (true, true, true, RefNode::BlockingAssignment(x)) => {
                let var = unwrap_node!(*x, VariableLvalueIdentifier).unwrap();
                let loc = unwrap_locate!(var.clone()).unwrap();
                let id = get_identifier(var, syntax_tree);
                self.locate_vars.insert(id.clone(), *loc);
                self.case_item_vars.insert(id);
            }

            (true, true, true, RefNode::BlockItemDeclaration(x)) => {
                let var = unwrap_node!(*x, VariableDeclAssignment).unwrap();
                let loc = unwrap_locate!(var.clone()).unwrap();
                let id = get_identifier(var, syntax_tree);
                self.locate_vars.insert(id.clone(), *loc);
                self.case_item_vars.insert(id);
            }

            _ => (),
        }

        SyntaxRuleResult::Pass
    }

    fn name(&self) -> String {
        String::from("implicit_case_default")
    }

    fn hint(&self, _option: &ConfigOption) -> String {
        String::from("Signal driven in `case` statement does not have a default value.")
    }

    fn reason(&self) -> String {
        String::from("Default values ensure that signals are never metastable.")
    }
}

fn get_identifier(node: RefNode, syntax_tree: &SyntaxTree) -> String {
    let id = match unwrap_node!(node, SimpleIdentifier, EscapedIdentifier) {
        Some(RefNode::SimpleIdentifier(x)) => Some(x.nodes.0),
        Some(RefNode::EscapedIdentifier(x)) => Some(x.nodes.0),
        _ => None,
    };

    String::from(syntax_tree.get_str(&id).unwrap())
}
