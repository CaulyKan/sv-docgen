use std::{
    collections::hash_map::DefaultHasher,
    fs,
    hash::{Hash, Hasher},
    path::Path,
    process::Command,
};

use crate::{
    comment_parser::{parse_comment, CommentItem},
    docgen::SvFile,
    numbered_list::NumberedList,
};

pub trait DocgenGenerator {
    fn generate(&self, items: Vec<SvFile>) -> String;
}

pub struct MarkdownGenerator {
    pub cwd: String,
    pub wavedrom: Option<String>,
    pub graphviz: Option<String>,
}

#[derive(Hash)]
struct FSM {
    name: String,
    transits: Vec<(String, String, String)>,
    states: Vec<(String, String)>,
}

impl MarkdownGenerator {
    pub fn new(cwd: String, wavedrom: Option<String>, graphviz: Option<String>) -> Self {
        MarkdownGenerator {
            cwd,
            wavedrom,
            graphviz,
        }
    }

    fn format_comment(&self, comments: &Vec<CommentItem>) -> String {
        let mut result = String::new();
        let mut current_fsm: Option<FSM> = None;

        for comment in comments {
            match comment {
                CommentItem::Author(s) => {
                    result.push_str(format!("**Author:** {}\n\n", s).as_str())
                }
                CommentItem::Example(s) => {
                    if s.contains("\n") {
                        result.push_str(format!("**Example:** \n```\n{}\n```\n\n", s).as_str());
                    } else {
                        result.push_str(format!("**Example:** `{}`\n\n", s).as_str());
                    }
                }
                CommentItem::Note(s) => {
                    if s.contains("\n") {
                        result.push_str("> **Note:**\n>\n");
                        s.split("\n")
                            .map(|x| "> ".to_owned() + x + "\n>\n")
                            .for_each(|x| result.push_str(x.as_str()));
                        result.push_str("\n");
                    } else {
                        result.push_str(format!("> **Note:** {}\n\n", s).as_str())
                    }
                }
                CommentItem::Ref(s) => {
                    result.push_str(format!("**Ref:** [{}]({})\n\n", s, s).as_str())
                }
                CommentItem::Return(s) => {
                    result.push_str(format!("**Return:** {}\n\n", s).as_str())
                }
                CommentItem::See(s) => {
                    result.push_str(format!("**Ref:** [{}]({})\n\n", s, s).as_str())
                }
                CommentItem::Wave(s) => {
                    result.push_str(
                        format!("**Waveform:** \n\n {}\n\n", self.generate_waveform(s)).as_str(),
                    );
                }
                CommentItem::State { name, desc } => {
                    if let Some(ref mut fsm) = current_fsm {
                        fsm.states.push((name.clone(), desc.clone()));
                    }
                }
                CommentItem::Transit { from, to, desc } => {
                    if let Some(ref mut fsm) = current_fsm {
                        fsm.transits.push((from.clone(), to.clone(), desc.clone()));
                    }
                }
                CommentItem::FSM(s) => {
                    if let Some(fsm) = &current_fsm {
                        result.push_str(
                            format!(
                                "**State Machine:** {}\n\n {}\n\n",
                                &fsm.name.clone(),
                                &self.generate_fsm(fsm)
                            )
                            .as_str(),
                        );
                    }
                    current_fsm = Some(FSM {
                        name: s.clone(),
                        states: Vec::new(),
                        transits: Vec::new(),
                    });
                }
                _ => (),
            }
        }
        if let Some(fsm) = current_fsm {
            result.push_str(
                format!(
                    "**State Machine:** {}\n\n {}\n\n",
                    &fsm.name.clone(),
                    &self.generate_fsm(&fsm)
                )
                .as_str(),
            );
        }
        result
    }

    fn generate_fsm(&self, fsm: &FSM) -> String {
        if let Some(graphviz) = &self.graphviz {
            let mut gv = String::from("digraph G {\n");

            let mut hasher = DefaultHasher::new();
            fsm.hash(&mut hasher);
            let hash = hasher.finish();
            let file_name = format!("docgen_fsm_{}.png", hash);
            let file_path = Path::new(&self.cwd).join(&file_name);
            let temp_file = Path::new(&self.cwd).join(".temp.gv");

            for (from, to, desc) in &fsm.transits {
                gv.push_str(format!("{}->{}[label=\"{}\"]\n", from, to, desc).as_str());
            }

            if fsm.states.len() > 0 {
                let s = fsm
                    .states
                    .iter()
                    .map(|(state, desc)| format!("{{ {} | {} }}", state, desc))
                    .collect::<Vec<String>>()
                    .join("|");
                gv.push_str(
                    format!(
                        "xxxxxxxxxxxxx[shape=record,label=\" {{ States | {} }} \"]\n",
                        s
                    )
                    .as_str(),
                );
            }

            gv.push_str("}");

            fs::write(&temp_file, gv).unwrap();

            Command::new(graphviz)
                .arg("-Tpng")
                .arg(temp_file.to_str().unwrap())
                .arg("-o")
                .arg(file_path.to_str().unwrap())
                .output()
                .unwrap();

            fs::remove_file(temp_file).unwrap();
            format!("![fsm]({})", file_name)
        } else {
            let mut result = String::from("");
            for (from, to, desc) in &fsm.transits {
                result.push_str(format!("* {}->{}: {}\n", from, to, desc).as_str());
            }
            result.push_str("\n");
            result
        }
    }

    fn generate_waveform(&self, s: &String) -> String {
        if let Some(wavedrom) = &self.wavedrom {
            let mut hasher = DefaultHasher::new();
            s.hash(&mut hasher);
            let hash = hasher.finish();
            let file_name = format!("docgen_wave_{}.png", hash);
            let file_path = Path::new(&self.cwd).join(&file_name);
            let temp_file = Path::new(&self.cwd).join(".temp.json");
            fs::write(&temp_file, s).unwrap();

            Command::new(wavedrom)
                .arg("-i")
                .arg(temp_file.to_str().unwrap())
                .arg("-p")
                .arg(file_path.to_str().unwrap())
                .output()
                .unwrap();

            fs::remove_file(temp_file).unwrap();

            format!("![wave]({})", file_name)
        } else {
            s.clone()
        }
    }
}

impl DocgenGenerator for MarkdownGenerator {
    fn generate(&self, items: Vec<SvFile>) -> String {
        let mut result = String::new();
        let mut index = NumberedList::new();

        for file in items {
            result.push_str(
                format!(
                    "# {}. File {}\n\n",
                    index.recall_and_go_downstairs(),
                    Path::new(file.name.as_str())
                        .file_name()
                        .and_then(|x| x.to_str())
                        .unwrap_or("")
                )
                .as_str(),
            );

            if file.modules.len() > 0 {
                for module in file.modules.iter() {
                    result.push_str(
                        format!(
                            "## {}. module {}\n\n",
                            index.recall_and_go_downstairs(),
                            module.name
                        )
                        .as_str(),
                    );
                    if let Some(brief) = &module.brief {
                        if brief != "" {
                            result.push_str(format!("{}\n\n", brief).as_str());
                        }
                    }
                    let s = self.format_comment(&module.comment);
                    result.push_str(s.as_str());
                    if module.params.len() > 0 {
                        result.push_str(
                            format!("### {}. Parameters\n\n", index.recall_and_step_forward())
                                .as_str(),
                        );
                        result.push_str("| name | default | type | dimensions | brief |\n");
                        result.push_str("| ---- | ------- | ---- | ---------- | ----- |\n");
                        for param in &module.params {
                            let v = vec![
                                param.name.as_str(),
                                param.default.as_ref().map(|x| x.as_str()).unwrap_or(""),
                                param.param_type.as_ref().map(|x| x.as_str()).unwrap_or(""),
                                param.dimensions.as_ref().map(|x| x.as_str()).unwrap_or(""),
                                param.comment.as_str(),
                            ];
                            result.push_str(format!("| {} |\n", v.join(" | ")).as_str());
                        }
                        result.push_str("\n");
                    }
                    if module.ports.len() > 0 {
                        result.push_str(
                            format!("### {}. Ports\n\n", index.recall_and_step_forward()).as_str(),
                        );
                        result.push_str("| name | direction | type | dimensions | brief |\n");
                        result.push_str("| ---- | --------- | ---- | ---------- | ----- |\n");
                        for port in &module.ports {
                            let v = vec![
                                port.name.as_str(),
                                port.direction.as_ref().map(|x| x.as_str()).unwrap_or(""),
                                port.port_type.as_ref().map(|x| x.as_str()).unwrap_or(""),
                                port.dimensions.as_ref().map(|x| x.as_str()).unwrap_or(""),
                                port.comment.as_str(),
                            ];
                            result.push_str(format!("| {} |\n", v.join(" | ")).as_str());
                        }
                        result.push_str("\n");
                    }
                    index.go_upstairs();
                }
            }
        }
        result
    }
}
