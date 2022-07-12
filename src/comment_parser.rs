use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not, tag, take_until},
    character::complete::multispace0,
    combinator::opt,
    multi::many0,
    sequence::{delimited, terminated},
    IResult,
};
pub type Span<'a> = nom_locate::LocatedSpan<&'a str>;

pub(crate) const AZ_: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_";
//pub(crate) const AZ09_: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_";
pub(crate) const AZ09_DOLLAR: &str =
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_$";

#[derive(Debug, Clone, PartialEq)]
pub enum CommentItem {
    Plain(String),
    Brief(String),
    Note(String),
    Ref(String),
    See(String),
    Example(String),
    Wave(String),
    Author(String),
    Rev {
        name: String,
        desc: String,
    },
    Port {
        name: String,
        desc: String,
    },
    Param {
        name: String,
        desc: String,
    },
    Return(String),
    FSM(String),
    State {
        name: String,
        desc: String,
    },
    Transit {
        from: String,
        to: String,
        desc: String,
    },
}

impl CommentItem {
    fn append_str(&mut self, s: &str) {
        match self {
            CommentItem::Plain(x) => x.push_str(s),
            CommentItem::Brief(x) => x.push_str(s),
            CommentItem::Note(x) => x.push_str(s),
            CommentItem::Ref(x) => x.push_str(s),
            CommentItem::See(x) => x.push_str(s),
            CommentItem::Example(x) => x.push_str(s),
            CommentItem::Wave(x) => x.push_str(s),
            CommentItem::Author(x) => x.push_str(s),
            CommentItem::Rev { name: _, desc } => desc.push_str(s),
            CommentItem::Port { name: _, desc } => desc.push_str(s),
            CommentItem::Param { name: _, desc } => desc.push_str(s),
            CommentItem::Return(x) => x.push_str(s),
            CommentItem::FSM(x) => x.push_str(s),
            CommentItem::State { name: _, desc } => desc.push_str(s),
            CommentItem::Transit {
                from: _,
                to: _,
                desc,
            } => desc.push_str(s),
        }
    }
}

fn parse_comment_item_plain(s: Span) -> IResult<Span, CommentItem> {
    let (s, _) = opt(delimited(multispace0, opt(tag("*")), multispace0))(s)?;
    let (s, plain) = is_not("\n")(s)?;
    let (s, _) = opt(tag("\n"))(s)?;

    Ok((s, CommentItem::Plain(plain.to_string())))
}

fn parse_command_item_simple<'a>(
    cmd: &'a str,
    p: impl Fn(String) -> CommentItem + 'a,
) -> Box<dyn Fn(Span) -> IResult<Span, CommentItem> + 'a> {
    Box::new(move |s: Span| {
        let (s, _) = opt(delimited(multispace0, opt(tag("*")), multispace0))(s)?;
        let (s, _) = (terminated(tag(cmd), alt((tag(" "), tag(":")))))(s)?;
        let (s, text) = opt(is_not("\n"))(s)?;
        let (s, _) = opt(tag("\n"))(s)?;
        let comment_line = text.unwrap_or(Span::from(""));
        Ok((s, p(comment_line.to_string())))
    })
}

fn identifier(s: Span) -> IResult<Span, String> {
    let (s, a) = is_a(AZ_)(s)?;
    let (s, b) = opt(is_a(AZ09_DOLLAR))(s)?;
    let a = if let Some(b) = b {
        a.to_string() + b.to_string().as_str()
    } else {
        a.to_string()
    };
    Ok((s, a))
}

fn parse_command_item_pair<'a>(
    cmd: &'a str,
    p: impl Fn(String, String) -> CommentItem + 'a,
) -> Box<dyn Fn(Span) -> IResult<Span, CommentItem> + 'a> {
    Box::new(move |s: Span| {
        let (s, _) = opt(delimited(multispace0, opt(tag("*")), multispace0))(s)?;
        let (s, _) = (terminated(tag(cmd), tag(" ")))(s)?;
        let (s, name) = terminated(identifier, alt((tag(" "), tag(":"))))(s)?;
        let (s, desc) = opt(is_not("\n"))(s)?;
        let (s, _) = opt(tag("\n"))(s)?;
        let desc = desc.unwrap_or(Span::from(""));
        Ok((s, p(name, desc.to_string())))
    })
}

fn parse_command_item_transit(s: Span) -> IResult<Span, CommentItem> {
    let (s, _) = opt(delimited(multispace0, opt(tag("*")), multispace0))(s)?;
    let (s, _) = tag("@")(s)?;
    let (s, from) = terminated(identifier, delimited(multispace0, tag("->"), multispace0))(s)?;
    let (s, to) = terminated(identifier, alt((tag(" "), tag(":"))))(s)?;
    let (s, desc) = opt(is_not("\n"))(s)?;
    let (s, _) = opt(tag("\n"))(s)?;
    Ok((
        s,
        CommentItem::Transit {
            from,
            to,
            desc: desc.map(|x| x.to_string()).unwrap_or("".to_string()),
        },
    ))
}

fn comment_item(s: Span) -> IResult<Span, CommentItem> {
    alt((
        parse_command_item_simple("@brief", |x| CommentItem::Brief(x)),
        parse_command_item_simple("@note", |x| CommentItem::Note(x)),
        parse_command_item_simple("@ref", |x| CommentItem::Ref(x)),
        parse_command_item_simple("@see", |x| CommentItem::See(x)),
        parse_command_item_simple("@example", |x| CommentItem::Example(x)),
        parse_command_item_simple("@wave", |x| CommentItem::Wave(x)),
        parse_command_item_simple("@author", |x| CommentItem::Author(x)),
        parse_command_item_simple("@return", |x| CommentItem::Return(x)),
        parse_command_item_simple("@fsm", |x| CommentItem::FSM(x)),
        parse_command_item_pair("@rev", |x, y| CommentItem::Rev { name: x, desc: y }),
        parse_command_item_pair("@port", |x, y| CommentItem::Port { name: x, desc: y }),
        parse_command_item_pair("@param", |x, y| CommentItem::Param { name: x, desc: y }),
        parse_command_item_pair("@state", |x, y| CommentItem::State { name: x, desc: y }),
        parse_command_item_transit,
        parse_comment_item_plain,
    ))(s)
}

fn oneline_comment(s: Span) -> IResult<Span, Vec<CommentItem>> {
    let (s, _) = tag("//*")(s)?;
    let (s, item) = comment_item(s)?;
    Ok((s, vec![item]))
}

fn multiline_comment(s: Span) -> IResult<Span, Vec<CommentItem>> {
    let (s, _) = tag("/**")(s)?;
    let (s, comment) = take_until("*/")(s)?;
    let (s, _) = tag("*/")(s)?;
    let (_, items) = many0(comment_item)(comment)?;
    Ok((s, items))
}

fn post_process(v: Vec<CommentItem>) -> Vec<CommentItem> {
    let mut result: Vec<CommentItem> = vec![];

    for item in v {
        match item {
            CommentItem::Plain(x) => {
                if let Some(last) = result.last_mut() {
                    last.append_str("\n");
                    last.append_str(x.as_str());
                } else {
                    result.push(CommentItem::Brief(x));
                }
            }
            _ => {
                result.push(item);
            }
        }
    }

    result
}

pub fn parse_comment(comment_str: &str) -> Vec<CommentItem> {
    let s = Span::from(comment_str);

    if let Ok((_, items)) = alt((oneline_comment, multiline_comment))(s) {
        post_process(items)
    } else {
        vec![]
    }
}

#[test]
fn test_parse_comment1() {
    assert_eq!(
        parse_comment("//* @brief test"),
        vec![CommentItem::Brief("test".to_string())]
    );
}

#[test]
fn test_parse_comment2() {
    assert_eq!(
        parse_comment("//* test"),
        vec![CommentItem::Brief("test".to_string())]
    );
}

#[test]
fn test_parse_comment3() {
    assert_eq!(
        parse_comment("/** test */"),
        vec![CommentItem::Brief("test ".to_string())]
    );
}

#[test]
fn test_parse_comment4() {
    let input = "/**
    * @brief test
    * @note noooooote
    */";
    assert_eq!(
        parse_comment(input),
        vec![
            CommentItem::Brief("test".to_string()),
            CommentItem::Note("noooooote".to_string())
        ]
    );
}

#[test]
fn test_parse_comment5() {
    let input = "/**
    * @brief test
    * second line
    * @param a:aaa
      @param b bbb
    */";
    assert_eq!(
        parse_comment(input),
        vec![
            CommentItem::Brief("test\nsecond line".to_string()),
            CommentItem::Param {
                name: "a".to_string(),
                desc: "aaa".to_string()
            },
            CommentItem::Param {
                name: "b".to_string(),
                desc: "bbb".to_string()
            },
        ]
    );
}

#[test]
fn test_parse_comment6() {
    let input = "/**
    * @fsm test
    * @a-> b transit1
    * @b ->c:transit2
    */";
    assert_eq!(
        parse_comment(input),
        vec![
            CommentItem::FSM("test".to_string()),
            CommentItem::Transit {
                from: "a".to_string(),
                to: "b".to_string(),
                desc: "transit1".to_string()
            },
            CommentItem::Transit {
                from: "b".to_string(),
                to: "c".to_string(),
                desc: "transit2".to_string()
            },
        ]
    );
}
