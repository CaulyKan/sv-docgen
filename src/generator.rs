use crate::{docgen::CommentDocument, numbered_list::NumberedList};

pub trait DocgenGenerator {
    fn generate(items: Vec<CommentDocument>) -> String;
}

pub struct MarkdownGenerator {}

impl DocgenGenerator for MarkdownGenerator {
    fn generate(items: Vec<CommentDocument>) -> String {
        let mut result = String::new();
        let mut index = NumberedList::new();

        let modules: Vec<CommentDocument> = items
            .iter()
            .filter(|x| matches!(x, CommentDocument::Module { .. }))
            .map(|x| x.clone())
            .collect();
        if modules.len() > 0 {
            result
                .push_str(format!("# {}. Modules\n\n", index.recall_and_go_downstairs()).as_str());
            for module in modules.iter() {
                match module {
                    CommentDocument::Module {
                        name,
                        brief,
                        ports,
                        params,
                        comment,
                    } => {
                        result.push_str(
                            format!(
                                "## {}. module {}\n\n",
                                index.recall_and_go_downstairs(),
                                name
                            )
                            .as_str(),
                        );
                        if brief != "" {
                            result.push_str(format!("{}\n\n", brief).as_str());
                        }
                        if params.len() > 0 {
                            result.push_str(
                                format!("### {}. Parameters\n\n", index.recall_and_step_forward())
                                    .as_str(),
                            );
                            result.push_str("| name | default | type | dimensions | brief |\n");
                            result.push_str("| ---- | ------- | ---- | ---------- | ----- |\n");
                            for param in params {
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
                        if ports.len() > 0 {
                            result.push_str(
                                format!("### {}. Ports\n\n", index.recall_and_step_forward())
                                    .as_str(),
                            );
                            result.push_str("| name | direction | type | dimensions | brief |\n");
                            result.push_str("| ---- | --------- | ---- | ---------- | ----- |\n");
                            for port in ports {
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
                    _ => (),
                }
            }
        }

        result
    }
}
