use crate::docgen::CommentDocument;

pub trait DocgenGenerator {
    fn generate(items: Vec<CommentDocument>) -> String;
}

pub struct MarkdownGenerator {}

impl DocgenGenerator for MarkdownGenerator {
    fn generate(items: Vec<CommentDocument>) -> String {
        let mut result = String::new();

        let modules: Vec<CommentDocument> = items
            .iter()
            .filter(|x| matches!(x, CommentDocument::Module { .. }))
            .map(|x| x.clone())
            .collect();
        if modules.len() > 0 {
            result.push_str("# Modules\n\n");
            for module in modules.iter() {
                match module {
                    CommentDocument::Module {
                        name,
                        brief,
                        ports,
                        params,
                        comment,
                    } => {
                        result.push_str(format!("## {}\n\n", name).as_str());
                        if ports.len() > 0 {
                            result.push_str("### Ports\n\n");
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
                    }
                    _ => (),
                }
            }
        }

        result
    }
}
