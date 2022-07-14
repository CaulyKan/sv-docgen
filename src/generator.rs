use crate::{docgen::SvFile, numbered_list::NumberedList};

pub trait DocgenGenerator {
    fn generate(items: Vec<SvFile>) -> String;
}

pub struct MarkdownGenerator {}

impl DocgenGenerator for MarkdownGenerator {
    fn generate(items: Vec<SvFile>) -> String {
        let mut result = String::new();
        let mut index = NumberedList::new();

        for file in items {
            result.push_str(
                format!(
                    "# {}. File {}\n\n",
                    index.recall_and_go_downstairs(),
                    file.name
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
