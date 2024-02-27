use crate::iterate_concatentation;
use garnish_lang_compiler::error::CompilerError;
use garnish_lang_compiler::{build::InstructionMetadata, lex::LexerToken, parse::ParseResult};
use garnish_lang_traits::{
    GarnishContext, GarnishData, GarnishDataType, GarnishNumber, Instruction, TypeConstants,
};

#[derive(Debug, Clone)]
pub struct BuildMetadata<Data>
where
    Data: GarnishData,
{
    name: String,
    build_root: Data::Size,
    source: String,
    // Optional in case of error
    lexing_tokens: Option<Vec<LexerToken>>,
    parse_result: Option<ParseResult>,
    instruction_data: Option<Vec<InstructionMetadata>>,
    error: Option<CompilerError<Data::Error>>,
}

impl<Data> BuildMetadata<Data>
where
    Data: GarnishData,
{
    pub fn new(
        name: String,
        source: String,
        build_root: Data::Size,
        lexing_tokens: Vec<LexerToken>,
        parse_result: ParseResult,
        instruction_data: Vec<InstructionMetadata>,
    ) -> Self {
        Self {
            name,
            source,
            build_root,
            lexing_tokens: Some(lexing_tokens),
            parse_result: Some(parse_result),
            instruction_data: Some(instruction_data),
            error: None,
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn get_input(&self) -> &String {
        &self.source
    }

    pub fn get_root_index(&self) -> Data::Size {
        self.build_root
    }

    pub fn get_lexing_tokens(&self) -> Option<&Vec<LexerToken>> {
        self.lexing_tokens.as_ref()
    }

    pub fn set_lexing_tokens(&mut self, tokens: Vec<LexerToken>) {
        self.lexing_tokens = Some(tokens);
    }

    pub fn get_parse_result(&self) -> Option<&ParseResult> {
        self.parse_result.as_ref()
    }

    pub fn set_parse_result(&mut self, result: ParseResult) {
        self.parse_result = Some(result);
    }

    pub fn get_instruction_data(&self) -> Option<&Vec<InstructionMetadata>> {
        self.instruction_data.as_ref()
    }

    pub fn set_instruction_data(&mut self, data: Vec<InstructionMetadata>) {
        self.instruction_data = Some(data);
    }

    pub fn get_error(&self) -> Option<&CompilerError<Data::Error>> {
        self.error.as_ref()
    }

    pub fn set_error(&mut self, error: CompilerError<Data::Error>) {
        self.error = Some(error);
    }
}

pub trait DataInfoProvider<Data>
where
    Data: GarnishData,
{
    fn get_symbol_name(&self, _sym: Data::Symbol, _data: &Data) -> Option<String> {
        None
    }
    fn get_address_name(&self, _addr: Data::Size, _data: &Data) -> Option<String> {
        None
    }
    fn format_symbol_data(&self, _addr: Data::Symbol, _data: &Data) -> Option<String> {
        None
    }
    fn format_custom_data(&self, _addr: Data::Size, _data: &Data) -> Option<String> {
        None
    }
}

pub fn format_runtime<Data, Context>(
    runtime: &Data,
    context: &Context,
    metadata: &Vec<BuildMetadata<Data>>,
) -> String
where
    Data: GarnishData,
    Context: GarnishContext<Data> + DataInfoProvider<Data>,
    Data::Size: Into<usize>,
{
    let mut lines = vec![];

    let max_addr = runtime.get_data_len().into().to_string().len();
    let max_jump = runtime.get_jump_table_len().into().to_string().len();

    let max_instruction = runtime.get_instruction_len().into().to_string().len();
    let max_data = std::cmp::max(max_jump, max_addr);
    let instruction_len = 25usize;
    let total_len = max_instruction + max_data + instruction_len + 20; // guessing extra space for data column

    let metadata = compile_metadata(&metadata);

    // instructions
    lines.push(format!(
        "{:-^total_len$}",
        format!(" Runtime "),
        total_len = total_len
    ));

    let instructions: Vec<(Data::Size, Instruction, Option<Data::Size>)> = runtime
        .get_instruction_iter()
        .map(|i| {
            let (instruction, data) = runtime
                .get_instruction(i)
                .unwrap_or((Instruction::Invalid, None));
            (i, instruction, data)
        })
        .collect();

    let with_metadata = instructions.iter().zip(metadata);

    for ((index, instruction, instruction_data), meta) in with_metadata {
        let (is_expression, expression_name) =
            instruction_is_start_of_expression(*index, runtime, context);

        let file_str = match meta {
            None => "[]".to_string(),
            Some((name, token)) => format!(
                "{} {}:{}",
                name,
                token.get_line() + 1,
                token.get_column() + 1,
            ),
        };

        let extra_column = match instruction {
            Instruction::Put | Instruction::Resolve => match instruction_data {
                None => format!("[No data addr provided]"),
                Some(addr) => complex_expression_data_format(*addr, runtime, context),
            },
            Instruction::JumpTo
            | Instruction::JumpIfTrue
            | Instruction::JumpIfFalse
            | Instruction::Reapply => match instruction_data {
                None => format!("[No jump point provided]"),
                Some(point) => match runtime.get_jump_point(*point) {
                    None => format!("[No jump point in table at provided index]"),
                    Some(addr) => format!("-> {:?}", addr),
                },
            },
            _ => format!(""),
        };

        let data_str = match instruction_data {
            Some(d) => format!("{:?}", d),
            None => format!(""),
        };

        let extra = format!("{}", extra_column);

        if is_expression {
            lines.push(format!(
                "{:-<total_len$}",
                format!("----- {} ", expression_name),
                total_len = total_len
            ));
        }
        // let extra = if context.get_show_meta() {
        //     format!("{} {}", meta_str, extra_column)
        // } else {
        //     extra_column
        // };

        lines.push(format!(
            "{:<max_instruction$} | {:instruction_len$}| {:max_data$} | {} | {}",
            format!("{:?}", index),
            format!("{:?}", instruction),
            format!("{}", data_str),
            file_str,
            extra,
            max_instruction = max_instruction,
            instruction_len = instruction_len,
            max_data = max_data
        ));

        // display separator if end of expression
        // and next instruction isn't the start of a named expression, to avoid double separators
        let (next_is_start, _) =
            instruction_is_start_of_expression(*index + Data::Size::one(), runtime, context);

        if !next_is_start && *instruction == Instruction::EndExpression {
            lines.push(format!("{:-<total_len$}", "", total_len = total_len));
        }
    }

    // data
    lines.push(format!(
        "{:-^total_len$}",
        format!(" Data "),
        total_len = total_len
    ));

    for i in runtime.get_data_iter() {
        lines.push(format!(
            "{:?} | {}",
            i,
            complex_expression_data_format(i, runtime, context)
        ));
    }

    lines.push(format!("{:-<total_len$}", "", total_len = total_len));

    // expression table
    lines.push(format!(
        "{:-^total_len$}",
        format!(" Jump Table "),
        total_len = total_len
    ));

    for (i, point) in runtime
        .get_jump_table_iter()
        .map(|i| (i, runtime.get_jump_point(i)))
    {
        lines.push(match point {
            None => format!("{} -> [Failed to retrieve]", i),
            Some(p) => match context.get_address_name(p, runtime) {
                None => format!("{} -> {}", i, p),
                Some(name) => format!("{} -> {} @ {}", i, name, p),
            },
        });
    }

    lines.join("\n")
}

fn instruction_is_start_of_expression<Data, Context>(
    start_point: Data::Size,
    data: &Data,
    context: &Context,
) -> (bool, String)
where
    Data: GarnishData,
    Context: GarnishContext<Data> + DataInfoProvider<Data>,
{
    let found = data
        .get_jump_table_iter()
        .map(|index| data.get_jump_point(index))
        .filter(|item| item.is_some())
        .map(|item| item.unwrap())
        .find(|point| *point == start_point);

    match found {
        None => (false, "".to_string()),
        Some(pos) => match context.get_address_name(pos, data) {
            None => (true, format!("[nested]")),
            Some(name) => (true, name.to_string()),
        },
    }
}

pub fn complex_expression_data_format<Data, Context>(
    index: Data::Size,
    data: &Data,
    context: &Context,
) -> String
where
    Data: GarnishData,
    Context: GarnishContext<Data> + DataInfoProvider<Data>,
{
    match data.get_data_type(index) {
        Ok(t) => match t {
            GarnishDataType::Expression => match data.get_expression(index) {
                Ok(value) => {
                    let jump_point = match data.get_jump_point(value) {
                        None => format!("[No jump point in table at {}]", value),
                        Some(pos) => match context.get_address_name(pos, data) {
                            None => format!("{:?}", pos),
                            Some(name) => name.to_string(),
                        },
                    };

                    format!(
                        "{} -> {}",
                        raw_expression_data_format(index, data, context),
                        jump_point
                    )
                }
                Err(_) => format!("[Addr {:?} is not Expression data]", index),
            },
            GarnishDataType::Symbol => match data.get_symbol(index) {
                Ok(value) => match context.format_symbol_data(value, data) {
                    None => match context.get_symbol_name(value, data) {
                        None => raw_expression_data_format(index, data, context),
                        Some(name) => format!("Symbol {:?} with value {:?}", name, value),
                    },
                    Some(referred_data) => referred_data,
                },
                Err(_) => format!("[Addr {:?} is not a symbol value]", index),
            },
            _ => format!(
                "{} - {}",
                raw_expression_data_format(index, data, context),
                simple_expression_data_format(index, data, context, 0)
            ),
        },
        Err(_) => format!("[No data found at addr {:?}]", index),
    }
}

pub fn raw_expression_data_format<Data, Context>(
    index: Data::Size,
    data: &Data,
    context: &Context,
) -> String
where
    Data: GarnishData,
    Context: GarnishContext<Data> + DataInfoProvider<Data>,
{
    match data.get_data_type(index) {
        Err(_) => format!("[No data found at addr {:?}]", index),
        Ok(t) => {
            let value_str = match data.get_data_type(index).unwrap() {
                GarnishDataType::Invalid => String::from("[Addr {:?} is Invalid]"),
                GarnishDataType::Custom => context
                    .format_custom_data(index, data)
                    .unwrap_or(String::from("[No display for Custom data]")),
                GarnishDataType::Type => match data.get_type(index) {
                    Ok(v) => format!("{:?}", v),
                    Err(_) => format!("[Addr {:?} is not a Type value]", index),
                },
                GarnishDataType::Unit => format!(""),
                GarnishDataType::True => format!(""),
                GarnishDataType::False => format!(""),
                GarnishDataType::Number => match data.get_number(index) {
                    Ok(v) => format!("{:?}", v),
                    Err(_) => format!("[Addr {:?} is not an integer value]", index),
                },
                GarnishDataType::Symbol => match data.get_symbol(index) {
                    Ok(value) => format!("{}", value),
                    Err(_) => format!("[Addr {:?} is not a Symbol value]", index),
                },
                GarnishDataType::Pair => match data.get_pair(index) {
                    Ok(value) => {
                        format!("{:?}", value)
                    }
                    Err(_) => format!("[Addr {:?} is not a Pair value]", index),
                },
                GarnishDataType::List => {
                    let mut items = vec![];
                    let mut associations = vec![];

                    let item_iter = data.get_list_items_iter(index);

                    for i in item_iter {
                        items.push(match data.get_list_item(index, i) {
                            Ok(addr) => format!("{:?}", addr),
                            Err(_) => {
                                format!("[List item {:?} at list {:?} not found]", i, index)
                            }
                        })
                    }

                    let item_iter = data.get_list_associations_iter(index);

                    for i in item_iter {
                        associations.push(match data.get_list_association(index, i) {
                            Ok(addr) => format!("{:?}", addr),
                            Err(_) => {
                                format!("[List item {:?} at list {:?} not found]", i, index)
                            }
                        })
                    }

                    format!(
                        "(items: {}, associations: {})",
                        items.join(", "),
                        associations.join(",")
                    )
                }
                GarnishDataType::Expression => {
                    format!("{:?}", data.get_expression(index).unwrap())
                }
                GarnishDataType::External => {
                    format!("{:?}", data.get_external(index).unwrap())
                }
                GarnishDataType::Char => {
                    format!("{:?}", data.get_char(index).unwrap())
                }
                GarnishDataType::CharList => {
                    let mut items = vec![];
                    for i in data.get_char_list_iter(index) {
                        let c = data.get_char_list_item(index, i).unwrap();
                        items.push(format!("{}", c));
                    }

                    format!("{:?}", items.join(""))
                }
                GarnishDataType::Byte => {
                    format!("{:?}", data.get_char(index).unwrap())
                }
                GarnishDataType::ByteList => {
                    let mut s = vec![];
                    for i in data.get_byte_list_iter(index) {
                        let c = data.get_byte_list_item(index, i).unwrap();
                        s.push(c.to_string());
                    }

                    format!("'{}'", s.join(" "))
                }
                GarnishDataType::Range => {
                    let (start, end) = data.get_range(index).unwrap();
                    format!("{:?}..{:?}", start, end)
                }
                GarnishDataType::Slice => {
                    let (start, end) = data.get_slice(index).unwrap();
                    format!("{:?}..{:?}", start, end)
                }
                GarnishDataType::Concatenation => {
                    let mut parts = vec![];

                    match iterate_concatentation(index, data, |item| {
                        parts.push(raw_expression_data_format(item, data, context))
                    }) {
                        Ok(_) => parts.join(", "),
                        Err(_) => format!("[Failed to format Concatenation at {}]", index),
                    }
                }
            };

            if value_str.is_empty() {
                format!("{:?}", t)
            } else {
                format!("{:?} - {}", t, value_str)
            }
        }
    }
}

pub fn simple_expression_data_format<Data, Context>(
    index: Data::Size,
    runtime: &Data,
    context: &Context,
    depth: usize,
) -> String
where
    Data: GarnishData,
    Context: GarnishContext<Data> + DataInfoProvider<Data>,
{
    match runtime.get_data_type(index) {
        Err(_) => format!("[No data found at addr {:?}]", index),
        Ok(t) => match t {
            GarnishDataType::Invalid => format!("[Data at {} is Invalid]", index),
            GarnishDataType::Custom => context
                .format_custom_data(index, runtime)
                .unwrap_or(String::from("[No display for Custom data]")),
            // match runtime
            GarnishDataType::Type => match runtime.get_type(index) {
                Ok(v) => format!("{:?}", v),
                Err(_) => format!("[Addr {:?} is not a Type value]", index),
            },
            GarnishDataType::Unit => format!("()"),
            GarnishDataType::True => format!("True"),
            GarnishDataType::False => format!("False"),
            GarnishDataType::Number => match runtime.get_number(index) {
                Ok(v) => format!("{}", v),
                Err(_) => format!("[Addr {:?} is not an integer value]", index),
            },
            GarnishDataType::Symbol => match runtime.get_symbol(index) {
                Ok(value) => context
                    .get_symbol_name(value, runtime)
                    .unwrap_or(format!("[No name for Symbol {}]", value)),
                Err(_) => format!("[Addr {:?} is not a Symbol value]", index),
            },
            GarnishDataType::Pair => match runtime.get_pair(index) {
                Ok((left_addr, right_addr)) => {
                    let left =
                        simple_expression_data_format(left_addr, runtime, context, depth + 1);
                    let right =
                        simple_expression_data_format(right_addr, runtime, context, depth + 1);
                    format!("{} = {}", left, right)
                }
                Err(_) => format!("[Addr {:?} is not a Pair value]", index),
            },
            GarnishDataType::List => match runtime.get_list_len(index) {
                Ok(item_len) => format_list(
                    runtime,
                    context,
                    index,
                    Data::Number::zero(),
                    Data::size_to_number(item_len),
                    depth,
                ),
                _ => format!("[Addr {:?} is not a List value]", index),
            },
            GarnishDataType::Expression => match runtime.get_expression(index) {
                Err(_) => format!("[Addr {:?} is not an Expression value]", index),
                Ok(addr) => runtime
                    .get_jump_point(addr)
                    .and_then(|point| {
                        context
                            .get_address_name(point, runtime)
                            .or(Some(format!("Expression @ {}", point)))
                    })
                    .unwrap_or(format!("[No jump point at index {}]", addr)),
            },
            GarnishDataType::External => match runtime.get_external(index) {
                Err(_) => format!("[Addr {:?} is not an External value]", index),
                Ok(addr) => format!("{{external - {}}}", addr),
            },
            GarnishDataType::Char => {
                format!("{:?}", runtime.get_char(index).unwrap())
            }
            GarnishDataType::CharList => match runtime.get_char_list_len(index) {
                Ok(len) => format_char_list(
                    runtime,
                    context,
                    index,
                    Data::Number::zero(),
                    Data::size_to_number(len),
                ),
                Err(_) => "[Could not get len of CharList]".to_string(),
            },
            GarnishDataType::Byte => {
                format!("{:?}", runtime.get_byte(index).unwrap())
            }
            GarnishDataType::ByteList => match runtime.get_byte_list_len(index) {
                Ok(len) => format_byte_list(
                    runtime,
                    context,
                    index,
                    Data::Number::zero(),
                    Data::size_to_number(len),
                ),
                Err(_) => "[Could not get length of ByteList]".to_string(),
            },
            GarnishDataType::Range => match runtime.get_range(index) {
                Ok((start, end)) => match (runtime.get_number(start), runtime.get_number(end)) {
                    (Ok(start), Ok(end)) => {
                        format!("{:?}..{:?}", start, end)
                    }
                    _ => "[Could not get start and end of range]".to_string(),
                },
                _ => "[Could not get range]".to_string(),
            },
            GarnishDataType::Slice => match runtime.get_slice(index) {
                Ok((value, range)) => match runtime.get_range(range) {
                    Ok((start, end)) => {
                        match (runtime.get_number(start), runtime.get_number(end)) {
                            (Ok(start), Ok(end)) => match runtime.get_data_type(value) {
                                Ok(GarnishDataType::List) => format_list(
                                    runtime,
                                    context,
                                    value,
                                    start,
                                    end.increment().unwrap_or(Data::Number::zero()),
                                    depth + 1,
                                ),
                                Ok(GarnishDataType::CharList) => format_char_list(
                                    runtime,
                                    context,
                                    value,
                                    start,
                                    end.increment().unwrap_or(start),
                                ),
                                Ok(GarnishDataType::ByteList) => format_byte_list(
                                    runtime,
                                    context,
                                    value,
                                    start,
                                    end.increment().unwrap_or(start),
                                ),
                                Ok(t) => format!("[Invalid value for slice {:?}]", t),
                                Err(_) => format!("[Failed to get data type]"),
                            },
                            _ => "[Could not get start and end of range of slice]".to_string(),
                        }
                    }
                    _ => "[Could not get range of slice]".to_string(),
                },
                Err(_) => "[Failed to get slice]".to_string(),
            },
            GarnishDataType::Concatenation => {
                let mut parts = vec![];

                match iterate_concatentation(index, runtime, |item| {
                    parts.push(simple_expression_data_format(
                        item,
                        runtime,
                        context,
                        depth + 1,
                    ))
                }) {
                    Ok(_) => match depth > 0 {
                        true => format!("({})", parts.join(", ")),
                        false => parts.join(", "),
                    },
                    Err(_) => format!("[Failed to format Concatenation at {}]", index),
                }
            }
        },
    }
}

pub fn format_char_list<Data, Context>(
    runtime: &Data,
    _context: &Context,
    list_addr: Data::Size,
    start: Data::Number,
    end: Data::Number,
) -> String
where
    Data: GarnishData,
    Context: GarnishContext<Data> + DataInfoProvider<Data>,
{
    let mut items = vec![];
    let len = match runtime.get_char_list_len(list_addr) {
        Ok(v) => v,
        Err(_) => return format!("[Could not get len of char list at {}]", list_addr),
    };

    for i in Data::make_number_iterator_range(start, end) {
        if Data::number_to_size(i).unwrap_or(Data::Size::zero()) >= len {
            items.push(Data::Char::default());
        } else {
            let c = runtime
                .get_char_list_item(list_addr, i)
                .unwrap_or(Data::Char::default());
            items.push(c);
        }
    }

    format!(
        "{:?}",
        items
            .iter()
            .map(|c| format!("{}", c))
            .collect::<Vec<String>>()
            .join("")
    )
}

pub fn format_byte_list<Data, Context>(
    runtime: &Data,
    _context: &Context,
    list_addr: Data::Size,
    start: Data::Number,
    end: Data::Number,
) -> String
where
    Data: GarnishData,
    Context: GarnishContext<Data> + DataInfoProvider<Data>,
{
    let mut items = vec![];
    let len = match runtime.get_byte_list_len(list_addr) {
        Ok(v) => v,
        Err(e) => {
            return format!(
                "[Could not get len of byte list at {} - {:?}]",
                list_addr, e
            )
        }
    };

    for i in Data::make_number_iterator_range(start, end) {
        if Data::number_to_size(i).unwrap_or(Data::Size::zero()) >= len {
            items.push(format!("'{}'", char::REPLACEMENT_CHARACTER.to_string()));
        } else {
            let c = runtime.get_byte_list_item(list_addr, i).unwrap();
            items.push(format!("'{}'", c.to_string()));
        }
    }

    format!("{}", items.join(" "))
}

pub fn format_list<Data, Context>(
    runtime: &Data,
    context: &Context,
    list_addr: Data::Size,
    start: Data::Number,
    end: Data::Number,
    depth: usize,
) -> String
where
    Data: GarnishData,
    Context: GarnishContext<Data> + DataInfoProvider<Data>,
{
    let mut items = vec![];

    let len = match runtime.get_list_len(list_addr) {
        Ok(v) => v,
        Err(_) => return format!("[Could not get len of list at {}]", list_addr),
    };

    for i in Data::make_number_iterator_range(start, end) {
        if Data::number_to_size(i).unwrap_or(Data::Size::zero()) >= len {
            items.push("()".to_string());
        } else {
            items.push(match runtime.get_list_item(list_addr, i) {
                Ok(addr) => simple_expression_data_format(addr, runtime, context, depth + 1),
                Err(_) => {
                    format!("[List item {:?} at list {:?} not found]", i, list_addr)
                }
            })
        }
    }

    match items.is_empty() {
        true => String::from("(,)"),
        false => match depth > 0 {
            true => format!("({})", items.join(", ")),
            false => format!("{}", items.join(", ")),
        },
    }
}

pub fn format_build_info<Data>(build_info: &BuildMetadata<Data>) -> String
where
    Data: GarnishData,
{
    let title = format!(
        "------- Build Info - {} -------------------------------------\n-------------------------------------------------------------",
        build_info.get_name()
    );

    let result_str = match build_info.get_error() {
        None => format!("Compilation Ok"),
        Some(e) => format!("Error during compilation {:?}", e),
    };

    let input_str = format!(
        "------- Input -------------------------------------\n{}",
        build_info.get_input()
    );

    let token_str = match build_info.get_lexing_tokens() {
        None => format!("No lexing tokens available to dump."),
        Some(tokens) => format_lexing_tokens(tokens),
    };

    let parse_str = match build_info.get_parse_result() {
        None => format!("No parse result available to dump."),
        Some(result) => format_parse_result_tree(result),
    };

    let footer = "-------------------------------------------------------------".to_string();

    [title, result_str, input_str, token_str, parse_str, footer].join("\n")
}

pub fn format_lexing_tokens(tokens: &Vec<LexerToken>) -> String {
    let tokens_str = tokens
        .iter()
        .map(|t| format!("{:?}", t.get_token_type()))
        .collect::<Vec<String>>()
        .join(" ");

    format!(
        "--------- Tokens ---------------------------------\n{}",
        tokens_str
    )
}

fn format_parse_node(
    result: &ParseResult,
    beginning: &str,
    index: Option<usize>,
    lines: &mut Vec<String>,
    depth: usize,
) -> Result<(), String> {
    if depth > result.get_nodes().len() {
        return Err(format!("Max depth reached"));
    }

    match index {
        None => (),
        Some(i) => match result.get_node(i) {
            None => (),
            Some(node) => {
                let child_depth = depth + 1;

                format_parse_node(result, " ", node.get_left(), lines, child_depth)?;
                lines.push(format!(
                    "{:-<length$}{}{:?}",
                    "",
                    beginning,
                    node.get_definition(),
                    length = depth * 2
                ));
                format_parse_node(result, " ", node.get_right(), lines, child_depth)?;
            }
        },
    }

    Ok(())
}

pub fn format_parse_result_tree(result: &ParseResult) -> String {
    let mut lines = vec![format!(
        "-------- Parse Result ---------------------------------"
    )];

    match format_parse_node(result, "* ", Some(result.get_root()), &mut lines, 0) {
        Ok(_) => lines.join("\n"),
        Err(_) => format_parse_result(result),
    }
}

#[allow(dead_code)]
pub fn format_parse_result(result: &ParseResult) -> String {
    let s = result
        .get_nodes()
        .iter()
        .enumerate()
        .map(|(i, n)| {
            format!(
                "{}: {:?} | Left {:?} | Parent {:?} | Right {:?}",
                i,
                n.get_definition(),
                n.get_left(),
                n.get_parent(),
                n.get_right()
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    format!(
        "-------- Parse Result ---------------------------------\n{}\n{}",
        format!("Root = {}", result.get_root()),
        s
    )
}

// metadata tuple
// (token string, line, column)
pub fn compile_metadata<Data: GarnishData>(
    builds: &Vec<BuildMetadata<Data>>,
) -> Vec<Option<(String, LexerToken)>> {
    // Simple data starts with end execution instruction
    // it gets skipped for display so none will work as placeholder
    let mut data = vec![];

    for build_info in builds {
        for meta in build_info.get_instruction_data().unwrap_or(&vec![]) {
            let d = match (meta.get_parse_node_index(), build_info.get_parse_result()) {
                (Some(index), Some(result)) => match result.get_node(index) {
                    None => None,
                    Some(node) => {
                        Some((build_info.get_name().clone(), node.get_lex_token().clone()))
                    }
                },
                _ => None,
            };

            data.push(d);
        }
    }

    data
}
