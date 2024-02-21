use crate::display::{simple_expression_data_format, DataInfoProvider};
use garnish_traits::{
    GarnishLangRuntimeContext, GarnishLangRuntimeData, GarnishLangRuntimeState, GarnishRuntime,
    Instruction, TypeConstants,
};

pub fn create_execution_dump<Runtime, Data, Context>(
    runtime: &mut Runtime,
    context: &mut Context,
) -> String
where
    Data: GarnishLangRuntimeData,
    Runtime: GarnishRuntime<Data>,
    Context: GarnishLangRuntimeContext<Data> + DataInfoProvider<Data>,
{
    let mut lines = vec![];

    loop {
        let instruction = runtime
            .get_data()
            .get_instruction(runtime.get_data().get_instruction_cursor());
        let line = match instruction {
            None => String::from("End of Instructions"),
            Some((instruction, data)) => {
                let info = match instruction {
                    Instruction::Invalid => String::from("[Invalid Instruction"),
                    Instruction::Resolve | Instruction::Put => match data {
                        None => format!("[No data addr provided]"),
                        Some(addr) => {
                            simple_expression_data_format(addr, runtime.get_data(), context, 0)
                        }
                    },
                    Instruction::JumpTo | Instruction::JumpIfTrue | Instruction::JumpIfFalse => {
                        match data {
                            None => format!("[No data addr provided]"),
                            Some(addr) => runtime
                                .get_data()
                                .get_jump_point(addr)
                                .map(|point| format!("{}", point))
                                .unwrap_or(format!("[No jump point at index {}]", addr)),
                        }
                    }
                    Instruction::PutValue => runtime
                        .get_data()
                        .get_current_value()
                        .and_then(|i| {
                            Some(simple_expression_data_format(
                                i,
                                runtime.get_data(),
                                context,
                                0,
                            ))
                        })
                        .unwrap_or(format!("[No current value]")),
                    Instruction::EndExpression => runtime
                        .get_data()
                        .get_current_value()
                        .and_then(|i| {
                            Some(simple_expression_data_format(
                                i,
                                runtime.get_data(),
                                context,
                                0,
                            ))
                        })
                        .unwrap_or(format!("[No current value]")),
                    Instruction::StartSideEffect | Instruction::EndSideEffect => String::new(),
                    Instruction::UpdateValue | Instruction::PushValue => {
                        let mut register_iterator = runtime.get_data().get_register_iter().rev();
                        format_register(
                            runtime.get_data().get_register(
                                register_iterator.next().unwrap_or(Data::Size::zero()),
                            ),
                            runtime.get_data(),
                            context,
                        )
                    }
                    Instruction::Opposite => {
                        format_unary_prefix_register("--", runtime.get_data(), context)
                    }
                    Instruction::AbsoluteValue => {
                        format_unary_prefix_register("++", runtime.get_data(), context)
                    }
                    Instruction::BitwiseNot => {
                        format_unary_prefix_register("!", runtime.get_data(), context)
                    }
                    Instruction::Not => {
                        format_unary_prefix_register("!!", runtime.get_data(), context)
                    }
                    Instruction::Tis => {
                        format_unary_prefix_register("??", runtime.get_data(), context)
                    }
                    Instruction::EmptyApply => {
                        format_unary_suffix_register("~~", runtime.get_data(), context)
                    }
                    Instruction::AccessLeftInternal => {
                        format_unary_prefix_register("_.", runtime.get_data(), context)
                    }
                    Instruction::AccessRightInternal => {
                        format_unary_suffix_register("._", runtime.get_data(), context)
                    }
                    Instruction::AccessLengthInternal => {
                        format_unary_suffix_register(".|", runtime.get_data(), context)
                    }
                    Instruction::TypeOf => {
                        format_unary_prefix_register("#", runtime.get_data(), context)
                    }
                    Instruction::Add => format_top_two_registers("+", runtime.get_data(), context),
                    Instruction::Subtract => {
                        format_top_two_registers("-", runtime.get_data(), context)
                    }
                    Instruction::Multiply => {
                        format_top_two_registers("*", runtime.get_data(), context)
                    }
                    Instruction::Divide => {
                        format_top_two_registers("/", runtime.get_data(), context)
                    }
                    Instruction::IntegerDivide => {
                        format_top_two_registers("//", runtime.get_data(), context)
                    }
                    Instruction::Power => {
                        format_top_two_registers("**", runtime.get_data(), context)
                    }
                    Instruction::Remainder => {
                        format_top_two_registers("%", runtime.get_data(), context)
                    }
                    Instruction::BitwiseAnd => {
                        format_top_two_registers("&", runtime.get_data(), context)
                    }
                    Instruction::BitwiseOr => {
                        format_top_two_registers("|", runtime.get_data(), context)
                    }
                    Instruction::BitwiseXor => {
                        format_top_two_registers("^", runtime.get_data(), context)
                    }
                    Instruction::BitwiseShiftLeft => {
                        format_top_two_registers("<<", runtime.get_data(), context)
                    }
                    Instruction::BitwiseShiftRight => {
                        format_top_two_registers(">>", runtime.get_data(), context)
                    }
                    Instruction::And => format_top_two_registers("&&", runtime.get_data(), context),
                    Instruction::Or => format_top_two_registers("||", runtime.get_data(), context),
                    Instruction::Xor => format_top_two_registers("^^", runtime.get_data(), context),
                    Instruction::ApplyType => {
                        format_top_two_registers("~#", runtime.get_data(), context)
                    }
                    Instruction::TypeEqual => {
                        format_top_two_registers("#=", runtime.get_data(), context)
                    }
                    Instruction::Equal => {
                        format_top_two_registers("==", runtime.get_data(), context)
                    }
                    Instruction::NotEqual => {
                        format_top_two_registers("!=", runtime.get_data(), context)
                    }
                    Instruction::LessThan => {
                        format_top_two_registers("<", runtime.get_data(), context)
                    }
                    Instruction::LessThanOrEqual => {
                        format_top_two_registers("<=", runtime.get_data(), context)
                    }
                    Instruction::GreaterThan => {
                        format_top_two_registers(">", runtime.get_data(), context)
                    }
                    Instruction::GreaterThanOrEqual => {
                        format_top_two_registers(">=", runtime.get_data(), context)
                    }
                    Instruction::MakePair => {
                        format_top_two_registers("=", runtime.get_data(), context)
                    }
                    Instruction::Apply => {
                        format_top_two_registers("~", runtime.get_data(), context)
                    }
                    Instruction::Reapply => {
                        format_top_two_registers("^~", runtime.get_data(), context)
                    }
                    Instruction::Access => {
                        format_top_two_registers(".", runtime.get_data(), context)
                    }
                    Instruction::MakeRange => {
                        format_top_two_registers("..", runtime.get_data(), context)
                    }
                    Instruction::MakeStartExclusiveRange => {
                        format_top_two_registers(">..", runtime.get_data(), context)
                    }
                    Instruction::MakeEndExclusiveRange => {
                        format_top_two_registers("..<", runtime.get_data(), context)
                    }
                    Instruction::MakeExclusiveRange => {
                        format_top_two_registers(">..<", runtime.get_data(), context)
                    }
                    Instruction::Concat => {
                        format_top_two_registers("<>", runtime.get_data(), context)
                    }
                    Instruction::MakeList => match data {
                        None => format!("[No data addr provided]"),
                        Some(len) => {
                            let mut future_items = vec![];
                            let mut register_iter = runtime.get_data().get_register_iter().rev();
                            let item_iter = Data::make_size_iterator_range(Data::Size::zero(), len);

                            for _ in item_iter {
                                match register_iter.next() {
                                    None => future_items.push(String::from("[register empty]")),
                                    Some(r) => future_items.push(format_register_depth(
                                        runtime.get_data().get_register(r),
                                        runtime.get_data(),
                                        context,
                                        1,
                                    )),
                                }
                            }

                            match future_items.is_empty() {
                                true => String::from("(,)"),
                                false => format!(
                                    "({}) {}",
                                    len,
                                    future_items
                                        .into_iter()
                                        .rev()
                                        .collect::<Vec<String>>()
                                        .join(", ")
                                ),
                            }
                        }
                    },
                };

                format!(
                    "({}) {:?}: {}",
                    runtime.get_data().get_instruction_cursor(),
                    instruction,
                    info,
                )
            }
        };

        lines.push(line);

        match runtime.execute_current_instruction(Some(context)) {
            Err(e) => {
                lines.push(format!("[Error: {}]", e));
                return lines.join("\n");
            }
            Ok(data) => match data.get_state() {
                GarnishLangRuntimeState::Running => (),
                GarnishLangRuntimeState::End => {
                    let result_line = runtime
                        .get_data()
                        .get_current_value()
                        .and_then(|i| {
                            Some(simple_expression_data_format(
                                i,
                                runtime.get_data(),
                                context,
                                0,
                            ))
                        })
                        .unwrap_or(format!("[No resulting value]"));

                    lines.push(format!("=== Execution Ended Successfully ==="));
                    lines.push(result_line);

                    return lines.join("\n")
                },
            },
        }

        let register_info = runtime
            .get_data()
            .get_register_iter()
            .map(|i| {
                format_register(
                    runtime.get_data().get_register(i),
                    runtime.get_data(),
                    context,
                )
            })
            .collect::<Vec<String>>()
            .join(" | ");

        lines.push(format!("    {}", register_info));
    }
}

fn format_unary_prefix_register<Data, Context>(op: &str, data: &Data, context: &Context) -> String
where
    Data: GarnishLangRuntimeData,
    Context: GarnishLangRuntimeContext<Data> + DataInfoProvider<Data>,
{
    let n = data
        .get_register_iter()
        .rev()
        .next()
        .unwrap_or(Data::Size::zero());
    format!(
        "{}{}",
        op,
        format_register(data.get_register(n), data, context),
    )
}

fn format_unary_suffix_register<Data, Context>(op: &str, data: &Data, context: &Context) -> String
where
    Data: GarnishLangRuntimeData,
    Context: GarnishLangRuntimeContext<Data> + DataInfoProvider<Data>,
{
    let n = data
        .get_register_iter()
        .rev()
        .next()
        .unwrap_or(Data::Size::zero());
    format!(
        "{}{}",
        format_register(data.get_register(n), data, context),
        op
    )
}

fn format_top_two_registers<Data, Context>(sep: &str, data: &Data, context: &Context) -> String
where
    Data: GarnishLangRuntimeData,
    Context: GarnishLangRuntimeContext<Data> + DataInfoProvider<Data>,
{
    let mut register_iterator = data.get_register_iter().rev();
    let right = register_iterator.next().unwrap_or(Data::Size::zero());
    let left = register_iterator.next().unwrap_or(Data::Size::zero());

    format!(
        "{} {} {}",
        format_register_depth(data.get_register(left), data, context, 1),
        sep,
        format_register_depth(data.get_register(right), data, context, 1)
    )
}

fn format_register<Data, Context>(
    index_opt: Option<Data::Size>,
    data: &Data,
    context: &Context,
) -> String
where
    Data: GarnishLangRuntimeData,
    Context: GarnishLangRuntimeContext<Data> + DataInfoProvider<Data>,
{
    format_register_depth(index_opt, data, context, 0)
}

fn format_register_depth<Data, Context>(
    index_opt: Option<Data::Size>,
    data: &Data,
    context: &Context,
    depth: usize,
) -> String
where
    Data: GarnishLangRuntimeData,
    Context: GarnishLangRuntimeContext<Data> + DataInfoProvider<Data>,
{
    match index_opt {
        None => format!("[No data in register]",),
        Some(addr) => simple_expression_data_format(addr, data, context, depth),
    }
}
