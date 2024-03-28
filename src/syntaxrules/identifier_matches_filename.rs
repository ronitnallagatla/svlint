use crate::config::ConfigOption;
use crate::linter::{SyntaxRule, SyntaxRuleResult};
use sv_parser::{unwrap_locate, NodeEvent, RefNode, SyntaxTree, unwrap_node};

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
                
                let path = if let Some(x) = unwrap_locate!(node.clone()) {
                    if let Some((path, _beg)) = syntax_tree.get_origin(&x) {
                        Some(path)
                    } else {
                        None
                    }
                } else {
                    println!( "Failing: path is None");
                    return SyntaxRuleResult::Fail;
                };
        
                if path.is_none() { 
                    println!("Failing: path is None2");
                    return SyntaxRuleResult::Fail; }
        
                let a = unwrap_node!(*x, ModuleIdentifier).unwrap();
                match a {
                    RefNode::ModuleIdentifier(module_ident) => {
                        let module_name = syntax_tree.get_str(module_ident).unwrap();
                        let path_str = path.unwrap(); // We already checked it's not None
                        let path = std::path::Path::new(path_str);
                        if let Some(file_name) = path.file_name().and_then(std::ffi::OsStr::to_str) {
                            if file_name.ends_with(".sv") {
                                let file_ident = file_name.trim_end_matches(".sv");
                                if module_name == file_ident {
                                    return SyntaxRuleResult::Pass;
                                }
                            }
                        }
                        return SyntaxRuleResult::Fail;
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
        String::from("Ensure that the module name matches the file name. module Bar should be in some/path/to/Bar.sv")
    }

    fn reason(&self) -> String {
        String::from("The module name does not match the file name.")
    }
}
