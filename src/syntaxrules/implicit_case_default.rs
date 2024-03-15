use crate::config::ConfigOption;
use crate::linter::{SyntaxRule, SyntaxRuleResult};
use sv_parser::{unwrap_node, Locate, NodeEvent, RefNode, SyntaxTree};

#[derive(Default)]
pub struct ImplicitCaseDefault {
    under_always_construct: bool,
    under_case_statement: bool,

    lhs_variables: Vec<String>,
    case_variables: Vec<String>
}

impl SyntaxRule for ImplicitCaseDefault {
    fn check(
        &mut self,
        syntax_tree: &SyntaxTree,
        event: &NodeEvent,
        _option: &ConfigOption,
    ) -> SyntaxRuleResult {
        //println!("{}", syntax_tree);

        let node = match event {
            NodeEvent::Enter(x) => {
                match x {
                    RefNode::AlwaysConstruct(_) => {
                        self.under_always_construct = true;
                    }

                    RefNode::CaseItemNondefault(_) => {
                        self.under_case_statement = true;
                    }
                    
                    _ => (),
                }
                x
            }

            NodeEvent::Leave(x) => {
                match x {
                    RefNode::AlwaysConstruct(_) => {
                        self.under_always_construct = false;
                        self.lhs_variables.clear();
                        self.case_variables.clear();

                    }

                    RefNode::CaseItemNondefault(_) => {
                        self.under_case_statement = false;
                    }

                    _ => ()
                }
                return SyntaxRuleResult::Pass;
            }
        };

        //println!("{}", node);
        
        // match implicit declarations
        match (self.under_always_construct, self.under_case_statement, node) {
            (true, false, RefNode::BlockItemDeclaration(x)) => {
                let var = unwrap_node!(*x, VariableDeclAssignment).unwrap();
                let id = get_identifier(var);
                let id = syntax_tree.get_str(&id).unwrap();
                self.lhs_variables.push(String::from(id));
            }

            _ => ()
        }

        // match case statement declarations
        match (self.under_always_construct, self.under_case_statement, node) {
            (true, true, RefNode::BlockingAssignment(x)) => {
                let var = unwrap_node!(*x, VariableLvalueIdentifier).unwrap();
                let id = get_identifier(var);
                let id = syntax_tree.get_str(&id).unwrap();
                
                if self.lhs_variables.contains(&id.to_string()) {
                    return SyntaxRuleResult::Pass
                } else {
                    return SyntaxRuleResult::Fail
                }
            }
            
            (true, true, RefNode::BlockItemDeclaration(x)) => {
                let var = unwrap_node!(*x, VariableDeclAssignment).unwrap();
                let id = get_identifier(var);
                let id = syntax_tree.get_str(&id).unwrap();
               
                if self.lhs_variables.contains(&id.to_string()) {
                    return SyntaxRuleResult::Pass
                } else {
                    return SyntaxRuleResult::Fail
                }
            }
            
            _ => ()
        }

        /*
        match (self.under_always_construct, node) {
            (true, RefNode::CaseStatementNormal(x)) => {
                let a = unwrap_node!(*x, CaseItemDefault);
                if a.is_some() {
                    SyntaxRuleResult::Pass
                } else {
                    // check if lvalues of case statement have an implicit definition
                    let var = unwrap_node!(*x, VariableLvalueIdentifier).unwrap();
                    let id = get_identifier(var);
                    let id = syntax_tree.get_str(&id).unwrap();

                    println!("Case variable: {id}");

                    // check if id is in lhs_variables
                    if self.lhs_variables.contains(&id.to_string()) {
                        SyntaxRuleResult::Pass
                    } else {
                        SyntaxRuleResult::Fail
                    }
                }
            }

            _ => {
                SyntaxRuleResult::Pass
            }
        }*/

        return SyntaxRuleResult::Pass
    }

    fn name(&self) -> String {
        String::from("implicit_case_default")
    }

    fn hint(&self, _option: &ConfigOption) -> String {
        String::from("Signal driven in `case` statement does not have a default value. Define a default case or implicitly define before `case` statement.")
    }

    fn reason(&self) -> String {
        String::from("Default values ensure that signals are always driven.")
    }
}

fn get_identifier(node: RefNode) -> Option<Locate> {
    match unwrap_node!(node, SimpleIdentifier, EscapedIdentifier) {
        Some(RefNode::SimpleIdentifier(x)) => Some(x.nodes.0),
        Some(RefNode::EscapedIdentifier(x)) => Some(x.nodes.0),
        _ => None,
    }
}