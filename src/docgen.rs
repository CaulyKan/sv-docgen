use crate::comment_parser::parse_comment;
use crate::comment_parser::CommentItem;
use std::cell::RefCell;
use std::collections::HashMap;
use std::default;
use std::env;
use std::error::Error;
use std::fmt;
use std::fs;
use std::future::pending;
use std::hash::Hash;
use std::ops::DerefMut;
use std::path::PathBuf;
use std::thread::panicking;
use sv_parser::parse_sv_str;
use sv_parser::port_declarations;
use sv_parser::unwrap_node;
use sv_parser::AnsiPortDeclaration;
use sv_parser::AnsiPortDeclarationNet;
use sv_parser::Comment;
use sv_parser::DataTypeOrImplicit;
use sv_parser::Define;
use sv_parser::Locate;
use sv_parser::NetPortHeaderOrInterfacePortHeader;
use sv_parser::NodeEvent;
use sv_parser::ParamAssignment;
use sv_parser::RefNode;
use sv_parser::RefNodes;
use sv_parser::SyntaxTree;

pub struct Docgen {
    tree: SyntaxTree,
}

#[derive(Debug, Clone)]
pub struct SvPort {
    pub name: String,
    pub port_type: Option<String>,
    pub direction: Option<String>,
    pub dimensions: Option<String>,
    pub comment: String,
}

#[derive(Debug, Clone)]
pub struct SvParam {
    pub name: String,
    pub param_type: Option<String>,
    pub default: Option<String>,
    pub dimensions: Option<String>,
    pub comment: String,
}

#[derive(Debug, Clone)]
pub struct SvState {
    pub name: String,
    pub transits: HashMap<String, String>,
    pub comment: Vec<CommentItem>,
}

#[derive(Debug, Clone)]
pub enum CommentDocument {
    Module {
        name: String,
        brief: String,
        ports: Vec<SvPort>,
        params: Vec<SvParam>,
        comment: Vec<CommentItem>,
    },
    Function {
        name: String,
        brief: String,
        ports: Vec<SvPort>,
        params: Vec<SvParam>,
        comment: Vec<CommentItem>,
    },
    Task {
        name: String,
        brief: String,
        ports: Vec<SvPort>,
        params: Vec<SvParam>,
        comment: Vec<CommentItem>,
    },
    Signal {
        name: String,
        brief: String,
        signal_type: String,
        width: String,
        dimensions: String,
        comment: Vec<CommentItem>,
    },
    StateMachine {
        name: String,
        brief: String,
        states: Vec<SvState>,
    },
    File {
        name: String,
        brief: String,
        author: String,
        rev: Vec<String>,
        comment: Vec<CommentItem>,
    },
}

impl CommentDocument {
    pub fn new_module(name: String, items: Vec<CommentItem>) -> Self {
        CommentDocument::Module {
            name,
            brief: String::from(""),
            ports: vec![],
            params: vec![],
            comment: items,
        }
    }

    pub fn refine(&self) -> Self {
        match self {
            CommentDocument::Module {
                name,
                ports,
                params,
                comment,
                ..
            } => {
                // let brief = comment.iter().fold(String::new(), |acc, x| match x {
                //     CommentItem::Brief(s) => acc + "\n" + s,
                //     _ => acc,
                // });
                let mut brief = Vec::new();
                let mut ports = ports.clone();
                let mut params = params.clone();
                let mut comments = Vec::new();
                for c in comment {
                    match c {
                        CommentItem::Brief(s) => brief.push(s.clone()),
                        CommentItem::Port { name, desc } => {
                            if let Some(p) = ports.iter_mut().find(|x| &x.name == name) {
                                p.comment = desc.clone();
                            }
                        }
                        CommentItem::Param { name, desc } => {
                            if let Some(p) = params.iter_mut().find(|x| &x.name == name) {
                                p.comment = desc.clone();
                            }
                        }
                        _ => comments.push(c),
                    }
                }
                CommentDocument::Module {
                    name: name.clone(),
                    brief: brief.join("\n"),
                    ports: ports.clone(),
                    params: params.clone(),
                    comment: comment.clone(),
                }
            }
            _ => self.clone(),
        }
    }
}

trait GetBrief {
    fn get_brief(&self) -> String;
}
impl GetBrief for Vec<CommentItem> {
    fn get_brief(&self) -> String {
        let briefs: Vec<String> = self
            .iter()
            .filter_map(|x| match x {
                CommentItem::Brief(s) => Some(s.clone()),
                _ => None,
            })
            .collect();
        briefs.join("\n")
    }
}

impl Docgen {
    pub fn from_file<'a>(
        file: &str,
        defines: &HashMap<String, Option<Define>>,
        includes: &Vec<PathBuf>,
    ) -> Result<Docgen, impl Error> {
        let content = fs::read_to_string(file).expect(&format!("unable to open {}", file));
        return Self::new(content.as_str(), defines, includes);
    }

    pub fn new<'a>(
        verilog: &str,
        defines: &HashMap<String, Option<Define>>,
        includes: &Vec<PathBuf>,
    ) -> Result<Docgen, impl Error> {
        let parsed = parse_sv_str(
            &verilog,
            &PathBuf::from(""),
            &defines,
            &includes,
            false,
            false,
        );
        match parsed {
            Ok((syntax_tree, _defines)) => Ok(Docgen { tree: syntax_tree }),
            Err(x) => Err(x),
        }
    }

    pub fn parse_tree(&self) -> Vec<CommentDocument> {
        let mut result = vec![];
        let mut doc_stack: Vec<CommentDocument> = vec![];
        let mut pending_items: Vec<CommentItem> = vec![];

        for event in self.tree.into_iter().event() {
            match event {
                NodeEvent::Enter(node) => match node {
                    RefNode::ModuleDeclaration(_) => {
                        let id = self.get_identifier(&node).unwrap();
                        let module = CommentDocument::new_module(id, pending_items);
                        doc_stack.push(module);
                        pending_items = vec![];
                    }
                    RefNode::AnsiPortDeclaration(x) => {
                        let port = match x {
                            AnsiPortDeclaration::Net(x) => {
                                let header = x.nodes.0.as_ref().and_then(|x| match x {
                                    NetPortHeaderOrInterfacePortHeader::NetPortHeader(y) => Some(y),
                                    _ => None,
                                });
                                let direction = header
                                    .and_then(|x| x.nodes.0.as_ref())
                                    .map(|x| self.get_str(x));
                                let port_type = header.map(|x| self.get_str(&x.nodes.1));
                                let dimensions = Some(
                                    x.nodes
                                        .2
                                        .iter()
                                        .map(|x| self.get_str(x))
                                        .collect::<String>(),
                                );
                                SvPort {
                                    name: self.get_str(&x.nodes.1),
                                    port_type,
                                    direction,
                                    dimensions,
                                    comment: "".to_string(),
                                }
                            }
                            AnsiPortDeclaration::Variable(x) => {
                                let header = x.nodes.0.clone();
                                let direction = header
                                    .as_ref()
                                    .and_then(|x| x.nodes.0.as_ref())
                                    .map(|x| self.get_str(x));
                                let port_type = header.map(|x| self.get_str(&x.nodes.1));
                                let dimensions = Some(
                                    x.nodes
                                        .2
                                        .iter()
                                        .map(|x| self.get_str(x))
                                        .collect::<String>(),
                                );
                                SvPort {
                                    name: self.get_str(&x.nodes.1),
                                    port_type,
                                    direction,
                                    dimensions,
                                    comment: "".to_string(),
                                }
                            }
                            AnsiPortDeclaration::Paren(x) => {
                                let direction = x.nodes.0.as_ref().map(|x| self.get_str(x));
                                SvPort {
                                    name: self.get_str(&x.nodes.2),
                                    port_type: None,
                                    direction,
                                    dimensions: None,
                                    comment: "".to_string(),
                                }
                            }
                        };

                        if let Some(item) = doc_stack
                            .iter_mut()
                            .rfind(|x| matches!(x, CommentDocument::Module { .. }))
                        {
                            match item {
                                CommentDocument::Module { ports, .. } => {
                                    ports.push(port);
                                }
                                _ => (),
                            }
                        }
                    }
                    RefNode::PortDeclaration(x) => {
                        for x in x.clone().into_iter() {
                            let mut port_type = String::new();
                            let mut direction = String::new();
                            let mut names = Vec::new();
                            let mut dimentions = Vec::new();
                            let mut ok = false;
                            match x {
                                RefNode::InputDeclarationNet(x) => {
                                    direction = self.get_str(&x.nodes.0).to_string();
                                    port_type = self.get_str(&x.nodes.1).to_string();
                                    let a = &x.nodes.2;
                                    let b = &a.nodes.0;
                                    let c = &b.nodes.1;
                                    let d = &b.nodes.0;
                                    names.push(self.get_str(&d.0));
                                    dimentions.push(self.get_str(&d.1));
                                    c.iter().for_each(|x| {
                                        names.push(self.get_str(&x.1 .0));
                                        dimentions.push(self.get_str(&x.1 .1))
                                    });
                                    ok = true;
                                }
                                RefNode::InputDeclarationVariable(x) => {
                                    direction = self.get_str(&x.nodes.0).to_string();
                                    port_type = self.get_str(&x.nodes.1).to_string();
                                    let a = &x.nodes.2;
                                    let b = &a.nodes.0;
                                    let c = &b.nodes.1;
                                    let d = &b.nodes.0;
                                    names.push(self.get_str(&d.0));
                                    dimentions.push(self.get_str(&d.1));
                                    c.iter().for_each(|x| {
                                        names.push(self.get_str(&x.1 .0));
                                        dimentions.push(self.get_str(&x.1 .1))
                                    });
                                    ok = true;
                                }
                                RefNode::OutputDeclarationNet(x) => {
                                    direction = self.get_str(&x.nodes.0).to_string();
                                    port_type = self.get_str(&x.nodes.1).to_string();
                                    let a = &x.nodes.2;
                                    let b = &a.nodes.0;
                                    let c = &b.nodes.1;
                                    let d = &b.nodes.0;
                                    names.push(self.get_str(&d.0));
                                    dimentions.push(self.get_str(&d.1));
                                    c.iter().for_each(|x| {
                                        names.push(self.get_str(&x.1 .0));
                                        dimentions.push(self.get_str(&x.1 .1))
                                    });
                                    ok = true;
                                }
                                RefNode::OutputDeclarationVariable(x) => {
                                    direction = self.get_str(&x.nodes.0).to_string();
                                    port_type = self.get_str(&x.nodes.1).to_string();
                                    let a = &x.nodes.2;
                                    let b = &a.nodes.0;
                                    let c = &b.nodes.1;
                                    let d = &b.nodes.0;
                                    names.push(self.get_str(&d.0));
                                    dimentions.push(self.get_str(&d.1));
                                    c.iter().for_each(|x| {
                                        names.push(self.get_str(&x.1 .0));
                                        dimentions.push(self.get_str(&x.1 .1))
                                    });
                                    ok = true;
                                }
                                RefNode::RefDeclaration(x) => {
                                    direction = self.get_str(&x.nodes.0).to_string();
                                    port_type = self.get_str(&x.nodes.1).to_string();
                                    let a = &x.nodes.2;
                                    let b = &a.nodes.0;
                                    let c = &b.nodes.1;
                                    let d = &b.nodes.0;
                                    names.push(self.get_str(&d.0));
                                    dimentions.push(self.get_str(&d.1));
                                    c.iter().for_each(|x| {
                                        names.push(self.get_str(&x.1 .0));
                                        dimentions.push(self.get_str(&x.1 .1))
                                    });
                                    ok = true;
                                }
                                //TODO: InterfacePortDeclaration
                                _ => ok = false,
                            }
                            if ok {
                                let mut new_ports: Vec<SvPort> = (0..names.len())
                                    .map(|i| SvPort {
                                        name: names[i].clone(),
                                        port_type: Some(port_type.clone()),
                                        direction: Some(direction.clone()),
                                        dimensions: Some(dimentions[i].clone()),
                                        comment: "".to_string(),
                                    })
                                    .collect();
                                if let Some(item) = doc_stack
                                    .iter_mut()
                                    .rfind(|x| matches!(x, CommentDocument::Module { .. }))
                                {
                                    match item {
                                        CommentDocument::Module { ports, .. } => {
                                            ports.append(&mut new_ports)
                                        }
                                        _ => (),
                                    }
                                }
                            }
                        }
                    }
                    RefNode::Comment(x) => {
                        let mut comment_items = parse_comment(self.get_str(x).as_str());
                        pending_items.append(&mut comment_items);
                    }
                    RefNode::ParameterDeclarationParam(x) => {
                        let param_type = if let DataTypeOrImplicit::DataType(dt) = x.nodes.1.clone()
                        {
                            Some(self.get_str(&dt))
                        } else {
                            None
                        };
                        let assignment0 = vec![&x.nodes.2.nodes.0.nodes.0];
                        let assignments: Vec<&ParamAssignment> =
                            x.nodes.2.nodes.0.nodes.1.iter().map(|x| &x.1).collect();
                        let mut new_params: Vec<SvParam> = assignment0
                            .iter()
                            .chain(assignments.iter())
                            .map(|x| SvParam {
                                name: self.get_str(&x.nodes.0),
                                dimensions: Some(self.get_str(&x.nodes.1)),
                                default: x.nodes.2.as_ref().map(|x| self.get_str(&x.1)),
                                param_type: param_type.clone(),
                                comment: pending_items.get_brief(),
                            })
                            .collect();

                        if let Some(item) = doc_stack
                            .iter_mut()
                            .rfind(|x| matches!(x, CommentDocument::Module { .. }))
                        {
                            match item {
                                CommentDocument::Module { params, .. } => {
                                    params.append(&mut new_params);
                                }
                                _ => (),
                            }
                        }
                        pending_items.clear();
                    }
                    _ => {
                        //pending_items.clear();
                    }
                },
                NodeEvent::Leave(node) => match node {
                    RefNode::ModuleDeclaration(_) => {
                        if let Some(d) = doc_stack.pop() {
                            result.push(d);
                        }
                    }
                    _ => (),
                },
            }
        }

        result
    }

    fn get_identifier(&self, node: &RefNode) -> Option<String> {
        let mut location: Option<Locate> = None;
        for x in node.clone().into_iter() {
            match x {
                RefNode::SimpleIdentifier(x) => {
                    location = Some(x.nodes.0);
                    break;
                }
                RefNode::EscapedIdentifier(x) => {
                    location = Some(x.nodes.0);
                    break;
                }
                _ => (),
            }
        }
        if location.is_some() {
            Some(self.tree.get_str(&location).unwrap().to_string())
        } else {
            None
        }
    }

    fn get_str<'a, T: Into<RefNodes<'a>>>(&self, node: T) -> String {
        self.tree.get_str(node).unwrap_or("").trim().to_string()
    }
}
