use anyhow::{Context, Error};
use clap::Parser;
use enquote;
use regex::Regex;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::{env, process};
use sv_parser::Error as SvParserError;
use sv_parser::{parse_sv, unwrap_locate, Define, DefineText, Locate, NodeEvent, RefNode};
use svlint::config::Config;
use svlint::linter::Linter;
use svlint::printer::Printer;
use verilog_filelist_parser;

// -------------------------------------------------------------------------------------------------
// Opt
// -------------------------------------------------------------------------------------------------

#[derive(Debug, Parser)]
#[clap(name = "svlint")]
#[clap(long_version(option_env!("LONG_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"))))]
pub struct Opt {
    /// Source file
    #[clap(required_unless_present_any = &["filelist", "example", "update-config"])]
    pub files: Vec<PathBuf>,

    /// File list
    #[clap(short = 'f', long = "filelist", conflicts_with = "files")]
    pub filelist: Vec<PathBuf>,

    /// Define
    #[clap(
        short = 'd',
        long = "define",
        multiple_occurrences = true,
        number_of_values = 1
    )]
    pub defines: Vec<String>,

    /// Include path
    #[clap(
        short = 'i',
        long = "include",
        multiple_occurrences = true,
        number_of_values = 1
    )]
    pub includes: Vec<PathBuf>,

    /// Config file
    #[clap(short = 'c', long = "config", default_value = ".svlint.toml")]
    pub config: PathBuf,

    /// Plugin file
    #[clap(
        short = 'p',
        long = "plugin",
        multiple_occurrences = true,
        number_of_values = 1
    )]
    pub plugins: Vec<PathBuf>,

    /// Ignore any include
    #[clap(long = "ignore-include")]
    pub ignore_include: bool,

    /// Prints results by single line
    #[clap(short = '1')]
    pub single: bool,

    /// Suppresses message
    #[clap(short = 's', long = "silent")]
    pub silent: bool,

    /// Prints verbose message
    #[clap(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// Prints message for GitHub Actions
    #[clap(long = "github-actions")]
    pub github_actions: bool,

    /// Updates config
    #[clap(long = "update")]
    pub update_config: bool,

    /// Prints config example
    #[clap(long = "example")]
    pub example: bool,
}

// -------------------------------------------------------------------------------------------------
// Main
// -------------------------------------------------------------------------------------------------

#[cfg_attr(tarpaulin, skip)]
pub fn main() {
    let opt = Parser::parse();
    let exit_code = match run_opt(&opt) {
        Ok(pass) => {
            if pass {
                0
            } else {
                1
            }
        }
        Err(x) => {
            let mut printer = Printer::new();
            let _ = printer.print_error_type(x);
            2
        }
    };

    process::exit(exit_code);
}

#[cfg_attr(tarpaulin, skip)]
pub fn run_opt(opt: &Opt) -> Result<bool, Error> {
    if opt.example {
        let config = Config::new();
        println!("{}", toml::to_string(&config).unwrap());
        return Ok(true);
    }

    let config = search_config(&opt.config);

    let config = if let Some(config) = config {
        let mut f = File::open(&config)
            .with_context(|| format!("failed to open '{}'", config.to_string_lossy()))?;
        let mut s = String::new();
        let _ = f.read_to_string(&mut s);
        let ret = toml::from_str(&s)
            .with_context(|| format!("failed to parse toml '{}'", config.to_string_lossy()))?;

        if opt.update_config {
            let mut f = OpenOptions::new()
                .write(true)
                .open(&config)
                .with_context(|| format!("failed to open '{}'", config.to_string_lossy()))?;
            write!(f, "{}", toml::to_string(&ret).unwrap())
                .with_context(|| format!("failed to write '{}'", config.to_string_lossy()))?;
            return Ok(true);
        }

        ret
    } else {
        println!(
            "Config file '{}' is not found. Enable all rules",
            opt.config.to_string_lossy()
        );
        Config::new().enable_all()
    };

    run_opt_config(opt, config)
}

#[cfg_attr(tarpaulin, skip)]
pub fn run_opt_config(opt: &Opt, config: Config) -> Result<bool, Error> {
    let mut printer = Printer::new();

    let mut not_obsolete = true;
    for (org_rule, renamed_rule) in config.check_rename() {
        printer.print_warning(&format!(
            "Rule \"{}\" is obsolete. Please rename to \"{}\"",
            org_rule, renamed_rule,
        ))?;
        not_obsolete = false;
    }

    let mut linter = Linter::new(config);
    for plugin in &opt.plugins {
        linter.load(&plugin);
    }

    let mut defines = HashMap::new();
    for define in &opt.defines {
        let mut define = define.splitn(2, '=');
        let ident = String::from(define.next().unwrap());
        let text = if let Some(x) = define.next() {
            let x = enquote::unescape(x, None)?;
            Some(DefineText::new(x, None))
        } else {
            None
        };
        let define = Define::new(ident.clone(), vec![], text);
        defines.insert(ident, Some(define));
    }

    let (files, includes) = if !opt.filelist.is_empty() {
        let mut files = opt.files.clone();
        let mut includes = opt.includes.clone();

        for filelist in &opt.filelist {
            let (mut f, mut i, d) = parse_filelist(filelist)?;
            files.append(&mut f);
            includes.append(&mut i);
            for (k, v) in d {
                defines.insert(k, v);
            }
        }

        (files, includes)
    } else {
        (opt.files.clone(), opt.includes.clone())
    };

    let mut all_pass = true;

    for path in &files {
        let mut pass = true;
        match parse_sv(&path, &defines, &includes, opt.ignore_include, false) {
            Ok((syntax_tree, new_defines)) => {
                let re_ctl = Regex::new(r"/\*\s*svlint\s+(on|off)\s+([a-z0-9_]+)\s*\*/").unwrap();

                for node in syntax_tree.into_iter().event() {
                    match node {
                        NodeEvent::Enter(RefNode::Comment(x)) => {
                            let loc: Option<&Locate> = unwrap_locate!(x);
                            let text: Option<&str> = match &loc {
                                Some(x) => syntax_tree.get_str(*x),
                                _ => None,
                            };
                            let caps = re_ctl.captures(text.unwrap());
                            if caps.is_some() {
                                let caps = caps.unwrap();
                                let ctl_name = caps.get(2).unwrap().as_str();
                                if linter.ctl_enabled.contains_key(ctl_name) {
                                    let ctl_enable = match caps.get(1).unwrap().as_str() {
                                        "off" => false,
                                        _ => true,
                                    };
                                    linter.ctl_enabled.insert(ctl_name.to_string(), ctl_enable);
                                    if opt.verbose {
                                        printer.print_info(&format!(
                                            "'{}':{} {} {}",
                                            &path.to_string_lossy(),
                                            loc.unwrap().line,
                                            if ctl_enable { "off" } else { "on" },
                                            &ctl_name
                                        ))?;
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                    for failed in linter.check(&syntax_tree, &node) {
                        pass = false;
                        if !opt.silent {
                            printer.print_failed(&failed, opt.single, opt.github_actions)?;
                        }
                    }
                }
                defines = new_defines;
            }
            Err(x) => {
                print_parse_error(&mut printer, x, opt.single)?;
                pass = false;
            }
        }

        if pass {
            if opt.verbose {
                printer.print_info(&format!("pass '{}'", path.to_string_lossy()))?;
            }
        } else {
            all_pass = false;
        }
    }

    Ok(all_pass && not_obsolete)
}

#[cfg_attr(tarpaulin, skip)]
fn print_parse_error(
    printer: &mut Printer,
    error: SvParserError,
    single: bool,
) -> Result<(), Error> {
    match error {
        SvParserError::Parse(Some((path, pos))) => {
            printer.print_parse_error(&path, pos, single)?;
        }
        SvParserError::Include { source: x } => {
            if let SvParserError::File { path: x, .. } = *x {
                printer.print_error(&format!("failed to include '{}'", x.to_string_lossy()))?;
            }
        }
        SvParserError::DefineArgNotFound(x) => {
            printer.print_error(&format!("define argument '{}' is not found", x))?;
        }
        SvParserError::DefineNotFound(x) => {
            printer.print_error(&format!("define '{}' is not found", x))?;
        }
        x => {
            printer.print_error(&format!("{}", x))?;
        }
    }

    Ok(())
}

#[cfg_attr(tarpaulin, skip)]
fn search_config(rule: &Path) -> Option<PathBuf> {
    if let Ok(current) = env::current_dir() {
        for dir in current.ancestors() {
            let candidate = dir.join(rule);
            if candidate.exists() {
                return Some(candidate);
            }
        }
        None
    } else {
        None
    }
}

#[cfg_attr(tarpaulin, skip)]
fn parse_filelist(
    path: &Path,
) -> Result<(Vec<PathBuf>, Vec<PathBuf>, HashMap<String, Option<Define>>), Error> {
    let filelist = match verilog_filelist_parser::parse_file(path) {
        Ok(f) => f,
        Err(_) => {
            return Err(anyhow::anyhow!(
                "failed to open '{}'",
                path.to_string_lossy()
            ))
        }
    };
    let mut defines = HashMap::new();
    for (d, t) in filelist.defines {
        match t {
            Some(t) => {
                let define_text = DefineText::new(String::from(&t[1..]), None);
                let define = Define::new(String::from(&d), vec![], Some(define_text));
                defines.insert(String::from(&d), Some(define));
            }
            None => {
                defines.insert(String::from(&d), None);
            }
        }
    }

    Ok((filelist.files, filelist.incdirs, defines))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test(name: &str, silent: bool) {
        let s = format!("[rules]\n{} = true", name);
        let config: Config = toml::from_str(&s).unwrap();

        let file = format!("testcases/pass/{}.sv", name);
        let args = if silent {
            vec!["svlint", "--silent", &file]
        } else {
            vec!["svlint", &file]
        };
        let opt = Opt::from_iter(args.iter());
        let ret = run_opt_config(&opt, config.clone());
        assert_eq!(ret.unwrap(), true);

        let file = format!("testcases/fail/{}.sv", name);
        let args = if silent {
            vec!["svlint", "--silent", &file]
        } else {
            vec!["svlint", &file]
        };
        let opt = Opt::from_iter(args.iter());
        let ret = run_opt_config(&opt, config.clone());
        assert_eq!(ret.unwrap(), false);

        let file = format!("testcases/fail/{}.sv", name);
        let args = if silent {
            vec!["svlint", "-1", "--silent", &file]
        } else {
            vec!["svlint", "-1", &file]
        };
        let opt = Opt::from_iter(args.iter());
        let ret = run_opt_config(&opt, config.clone());
        assert_eq!(ret.unwrap(), false);
    }

    include!(concat!(env!("OUT_DIR"), "/test.rs"));
}
