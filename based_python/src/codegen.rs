use crate::ast::{Program, Statement, Expression, Block};

pub fn generate_python_code(program: &Program) -> String {
    let mut output = String::new();
    for statement in &program.statements {
        generate_statement(statement, 0, &mut output);
    }
    output
}

fn generate_statement(statement: &Statement, indent_level: usize, output: &mut String) {
    let indent_str = "    ".repeat(indent_level); // 4 spaces/level

    match statement {
        Statement::Assignment { name, value } => {
            output.push_str(&format!("{}{} = {}\n", indent_str, name, generate_expression(value)));
        }
        Statement::Print { content } => {
            output.push_str(&format!("{}print({})\n", indent_str, generate_expression(content)));
        }
        Statement::Return { value } => {
            output.push_str(&format!("{}return {}\n", indent_str, generate_expression(value)));
        }
        Statement::If { condition, consequence, alternative } => {
            output.push_str(&format!("{}if {}:\n", indent_str, generate_expression(condition)));
            generate_block(consequence, indent_level + 1, output);
            if let Some(alt_block) = alternative {
                output.push_str(&format!("{}else:\n", indent_str));
                generate_block(alt_block, indent_level + 1, output);
            }
        }
        Statement::For { iterator, body } => {
            output.push_str(&format!("{}for {}:\n", indent_str, generate_expression(iterator)));
            generate_block(body, indent_level + 1, output);
        }
        Statement::While { condition, body } => {
            output.push_str(&format!("{}while {}:\n", indent_str, generate_expression(condition)));
            generate_block(body, indent_level + 1, output);
        }
        Statement::FunctionDef { name, args, body } => {
            let args_str = args.join(", ");
            output.push_str(&format!("{}def {}({}):\n", indent_str, name, args_str));
            generate_block(body, indent_level + 1, output);
        }
        Statement::FunctionCall { name, arguments } => {
            let args_str = arguments
                .iter()
                .map(generate_expression)
                .collect::<Vec<_>>()
                .join(", ");
            output.push_str(&format!("{}{}({})\n", indent_str, name, args_str));
        },
        Statement::ClassDef { name, body } => {
            output.push_str(&format!("{}class {}:\n", indent_str, name));
            generate_block(body, indent_level + 1, output);
        }
    }
}

fn generate_expression(expression: &Expression) -> String {
    match expression {
        Expression::Identifier(name) => name.clone(),
        Expression::Number(num) => num.to_string(),
        Expression::String(s) => format!("\"{}\"", s),
        Expression::BinaryOp { left, operator, right } => {
            format!("{} {} {}",
                    generate_expression(left),
                    operator,
                    generate_expression(right)
            )
        },
        Expression::MemberAccess { object, member } => {
            format!("{}.{}", generate_expression(object), member)
        }
        _ => {println!("Unknown expression: {:?}", expression); "UNKNOWN".to_string()}
    }
}

fn generate_block(block: &Block, indent_level: usize, output: &mut String) {
    if block.statements.is_empty() {
        output.push_str(&format!("{}    pass\n", "    ".repeat(indent_level - 1)));
    } else {
        for statement in &block.statements {
            generate_statement(statement, indent_level, output);
        }
    }
}