use crate::config::ConfigOption;
use crate::linter::{SyntaxRule, SyntaxRuleResult};
use sv_parser::{NodeEvent, RefNode, SyntaxTree, unwrap_node};// unwrap_locate, unwrap_node

#[derive(Default)]
pub struct IdentifierMatchesFilename;
impl SyntaxRule for IdentifierMatchesFilename {
    fn check(
        &mut self,
        syntax_tree: &SyntaxTree,
        event: &NodeEvent,
        _option: &ConfigOption,
    ) -> SyntaxRuleResult {
        let node = match event {
            NodeEvent::Enter(x) => x,
            NodeEvent::Leave(_) => {
                return SyntaxRuleResult::Pass;
            }
        };


        match node {
            RefNode::ModuleDeclaration(x) => {
                let a = unwrap_node!(*x, ModuleIdentifier).unwrap();
                match a {
                    RefNode::ModuleIdentifier(module_ident) => {
                        let module_name = syntax_tree.get_str(module_ident).unwrap();
                        let mut found_matching_file = false;
                
                        // Iterate over command line arguments
                        for arg in std::env::args_os() {
                            if let Some(arg_str) = arg.to_str() {
                                let path = std::path::Path::new(arg_str);
                                if let Some(file_name) = path.file_name().and_then(std::ffi::OsStr::to_str) {
                                    if file_name.ends_with(".sv") {
                                        let file_ident = file_name.trim_end_matches(".sv");

                                        if module_name == file_ident {
                                            found_matching_file = true;
                                            break; // We found a match, no need to continue checking other files
                                        }
                                    }
                                }
                            }
                        }
                
                        if found_matching_file {
                            return SyntaxRuleResult::Pass;
                        } else {
                            return SyntaxRuleResult::Fail;
                        }
                    }
                    _ => unreachable!(),
                }
                
            }
            _ => SyntaxRuleResult::Pass,
        }
    }

    fn name(&self) -> String {
        String::from("identifier_matches_filename")
    }

    fn hint(&self, _option: &ConfigOption) -> String {
        String::from("Ensure that the module name matches the file name.")
    }

    fn reason(&self) -> String {
        String::from("The module name does not match the file name.")
    }
}
