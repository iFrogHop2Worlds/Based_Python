use pest::Parser;
use pest::iterators::{Pair, Pairs};
use crate::ast::{Program, Statement, Expression, Block, Operator};
use std::error::Error;
use std::fmt;
use pest_derive::Parser;
use lazy_static::lazy_static;
use pest::pratt_parser::{Assoc, Op, PrattParser};

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct BythonParser;


#[derive(Debug)]
pub struct BythonParseError {
    message: String,
}

impl fmt::Display for BythonParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for BythonParseError {}

lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        PrattParser::new()
            .op(Op::infix(Rule::eq, Assoc::Left))
            .op(Op::infix(Rule::neq, Assoc::Left))
            .op(Op::infix(Rule::gt, Assoc::Left))
            .op(Op::infix(Rule::lt, Assoc::Left))
            .op(Op::infix(Rule::gte, Assoc::Left))
            .op(Op::infix(Rule::lte, Assoc::Left))

            .op(Op::infix(Rule::add, Assoc::Left))
            .op(Op::infix(Rule::subtract, Assoc::Left))

            .op(Op::infix(Rule::multiply, Assoc::Left))
            .op(Op::infix(Rule::divide, Assoc::Left))
    };
}

/// Parses the input string and returns the AST.
pub fn parse_bython_code(input: &str) -> Result<Program, BythonParseError> {
    let parse_result = BythonParser::parse(Rule::program, input);

    match parse_result {
        Ok(pairs) => {
            let mut statements = Vec::new();
            for pair in pairs {
                if pair.as_rule() == Rule::program {
                    for inner_pair in pair.into_inner() {
                        if inner_pair.as_rule() == Rule::statement {
                            statements.push(parse_statement(inner_pair)?);
                        }
                    }
                }
            }
            Ok(Program { statements })
        }
        Err(e) => Err(BythonParseError {
            message: format!("Parsing error: {}", e),
        }),
    }
}

fn parse_statement(pair: Pair<Rule>) -> Result<Statement, BythonParseError> {
    match pair.as_rule() {
        Rule::assignment_statement => {
            let mut inner = pair.into_inner();
            let target = inner.next().unwrap();
            let value_pair = inner.next().unwrap();
            
            let value = match value_pair.as_rule() {
                Rule::function_call_stmt => {
                    let func_call_inner = value_pair.into_inner().next().unwrap();
                    parse_expression(func_call_inner)?
                },
                Rule::term => parse_expression(value_pair)?,
                _ => parse_expression(value_pair)?
            };

            match target.as_rule() {
                Rule::ident => Ok(Statement::Assignment {
                    name: target.as_str().to_string(),
                    value
                }),
                Rule::member_access => {
                    let mut parts = target.into_inner();
                    let object = parts.next().unwrap().as_str().to_string();
                    let member = parts.next().unwrap().as_str().to_string();
                    println!("MemberAcess {:?}", member);
                    Ok(Statement::Assignment {
                        name: format!("{}.{}", object, member),
                        value
                    })
                },
                _ => Err(BythonParseError {
                    message: format!("Invalid assignment target: {:?}", target.as_rule())
                })
            }
        }
        Rule::print_statement => {
            let mut inner = pair.into_inner();
            let content_pair = inner.next().unwrap();
            let content = parse_expression(content_pair.into_inner().next().unwrap())?;
            Ok(Statement::Print { content })
        }
        Rule::return_statement => {
            let mut inner = pair.into_inner();
            let value_pair = inner.next().unwrap();
            let value = parse_expression(value_pair)?;
            Ok(Statement::Return { value })
        }
        Rule::if_statement => {
            let mut inner = pair.into_inner();
            inner.next();
            let condition = parse_expression(inner.next().unwrap())?;
            let consequence = parse_block(inner.next().unwrap())?;
            let alternative = if let Some(else_token) = inner.next() {
                if else_token.as_rule() == Rule::KEYWORD_ELSE {
                    Some(parse_block(inner.next().unwrap())?)
                } else {
                    None
                }
            } else {
                None
            };
            Ok(Statement::If {
                condition,
                consequence,
                alternative,
            })
        }
        Rule::for_statement => {
            let mut inner = pair.into_inner();
            let iterator = parse_expression(inner.next().unwrap())?;
            let body = parse_block(inner.next().unwrap())?;
            Ok(Statement::For { iterator, body })
        }
        Rule::while_statement => {
            let mut inner = pair.into_inner();
            let condition = parse_expression(inner.next().unwrap())?;
            let body = parse_block(inner.next().unwrap())?;
            Ok(Statement::While { condition, body })
        }
        Rule::function_def => {
            let mut inner = pair.into_inner();

            inner.next(); // Skip keyword

            let name_pair = inner.next().unwrap();
            let name = match name_pair.as_rule() {
                Rule::ident | Rule::dunder_ident => name_pair.as_str().to_string(),
                _ => return Err(BythonParseError {
                    message: format!("Expected function name, got {:?}", name_pair.as_rule())
                })
            };

            let mut args = Vec::new();
            let arg_list_pair = inner.next().unwrap();
            
            match arg_list_pair.as_rule() {
                Rule::arg_list => {
                    for arg_pair in arg_list_pair.into_inner() {
                        if arg_pair.as_rule() == Rule::ident {
                            args.push(arg_pair.as_str().to_string());
                        }
                    }
                },
                Rule::param_list => {
                    for arg_pair in arg_list_pair.into_inner() {
                        if arg_pair.as_rule() == Rule::ident {
                            args.push(arg_pair.as_str().to_string());
                        }
                    }
                },
                _ => return Err(BythonParseError {
                    message: format!("Expected argument list, got {:?}", arg_list_pair.as_rule())
                })
            }

            let body = parse_block(inner.next().unwrap())?;

            Ok(Statement::FunctionDef { name, args, body })
        }
        Rule::function_call => {
            let mut inner = pair.into_inner();
            let mut name = String::new();

            let name_part = inner.next().unwrap();
            match name_part.as_rule() {
                Rule::ident => name = name_part.as_str().to_string(),
                Rule::member_access => {
                    name = name_part.as_str().to_string();
                },
                _ => return Err(BythonParseError {
                    message: format!("Unexpected rule for function name: {:?}", name_part.as_rule()),
                })
            }

            let mut arguments = Vec::new();
            for arg_list in inner {
                if arg_list.as_rule() == Rule::param_list {
                    for arg in arg_list.into_inner() {
                        arguments.push(parse_expression(arg)?);
                    }
                }
            }

            Ok(Statement::FunctionCall { name, arguments })
        },
        Rule::function_call_stmt => {
            // Handle function call statements
            let inner = pair.into_inner().next().unwrap();
            match inner.as_rule() {
                Rule::function_call => parse_statement(inner),
                Rule::class_instantiation => {
                    // Convert class instantiation to function call statement
                    let mut class_inner = inner.into_inner();
                    let class_name = class_inner.next().unwrap().as_str().to_string();
                    let mut arguments = Vec::new();

                    if let Some(arg_pair) = class_inner.next() {
                        if arg_pair.as_rule() == Rule::term {
                            arguments.push(parse_expression(arg_pair)?);
                        }
                    }

                    Ok(Statement::FunctionCall { name: class_name, arguments })
                },
                _ => {
                    // Handle member function calls (e.g., obj.method())
                    let expr = parse_expression(inner.clone())?;
                    match expr {
                        Expression::FunctionCall { name, args } => {
                            Ok(Statement::FunctionCall { name, arguments: args })
                        },
                        _ => Err(BythonParseError {
                            message: format!("Expected function call in statement, got {:?}", inner.as_rule())
                        })
                    }
                }
            }
        },
        Rule::class_def => {
            let mut inner = pair.into_inner();
            inner.next();
            let name = inner.next().unwrap().as_str().to_string();
            let body = parse_block(inner.next().unwrap())?;
            Ok(Statement::ClassDef { name, body })
        }
        Rule::statement => {
            let inner = pair.into_inner().next().ok_or_else(|| BythonParseError {
                message: "Empty statement".to_string(),
            })?;
            parse_statement(inner)
        }
        _ => Err(BythonParseError {
            message: format!("Unexpected rule for statement: {:?}", pair.as_rule()),
        }),
    }
}

fn parse_expression(pair: Pair<Rule>) -> Result<Expression, BythonParseError> {
    match pair.as_rule() {
        Rule::member_access => {
            let mut parts = pair.into_inner();
            let object = Expression::Identifier(parts.next().unwrap().as_str().to_string());
            let member = parts.next().unwrap().as_str().to_string();

            Ok(Expression::MemberAccess {
                object: Box::new(object),
                member
            })
        },
        Rule::BinOperation => {
            let mut parts = pair.into_inner();
            let left = parse_expression(parts.next().unwrap())?;
            let operator = parts.next().unwrap().as_str().to_string();
            let right = parse_expression(parts.next().unwrap())?;

            Ok(Expression::BinaryOp {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            })
        }
        Rule::expression => {
            let pairs = pair.into_inner();
            Ok(PRATT_PARSER
                .map_primary(|pair| parse_term(pair))
                .map_infix(|lhs: Result<Expression, BythonParseError>, op: Pair<Rule>, rhs: Result<Expression, BythonParseError>| {
                    match (lhs, rhs) {
                        (Ok(left), Ok(right)) => Ok(Expression::BinaryOp {
                            left: Box::new(left),
                            operator: op.as_str().to_string(),
                            right: Box::new(right),
                        }),
                        (Err(e), _) => Err(e),
                        (_, Err(e)) => Err(e),
                    }
                })
                .parse(pairs)?)
        },
        Rule::class_instantiation => {
            let mut inner = pair.into_inner();
            let class_name = inner.next().unwrap().as_str().to_string();
            let mut arguments = Vec::new();

            if let Some(arg_pair) = inner.next() {
                if arg_pair.as_rule() == Rule::term {
                    arguments.push(parse_expression(arg_pair)?);
                }
            }

            Ok(Expression::ClassInstantiation {
                class_name,
                arguments
            })
        },
        Rule::function_call => {
            let mut inner = pair.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let mut args = Vec::new();

            for arg_list in inner {
                if arg_list.as_rule() == Rule::param_list {
                    for arg in arg_list.into_inner() {
                        args.push(parse_expression(arg)?);
                    }
                }
            }

            Ok(Expression::FunctionCall { name, args })
        },
        Rule::function_call_stmt => {
            let inner = pair.into_inner().next().unwrap();
            match inner.as_rule() {
                Rule::function_call => parse_expression(inner),
                Rule::class_instantiation => parse_expression(inner),
                _ => {
                    let parts: Vec<&str> = inner.as_str().split('.').collect();
                    if parts.len() == 2 {
                        let obj_name = parts[0].to_string();
                        let method_call = parts[1];
                        
                        if let Some(paren_pos) = method_call.find('(') {
                            let method_name = method_call[..paren_pos].to_string();
                            Ok(Expression::FunctionCall {
                                name: format!("{}.{}", obj_name, method_name),
                                args: vec![]
                            })
                        } else {
                            Err(BythonParseError {
                                message: "Invalid member function call".to_string()
                            })
                        }
                    } else {
                        parse_expression(inner)
                    }
                }
            }
        },
        _ => parse_term(pair),
    }
}

fn parse_term(pair: Pair<Rule>) -> Result<Expression, BythonParseError> {
    match pair.as_rule() {
        Rule::ident => Ok(Expression::Identifier(pair.as_str().to_string())),
        Rule::number => Ok(Expression::Number(pair.as_str().parse().unwrap())),
        Rule::string => {
            let s = pair.as_str();
            Ok(Expression::String(s[1..s.len() - 1].to_string()))
        }
        Rule::paren_expression => parse_expression(pair.into_inner().next().unwrap()),
        Rule::function_call => parse_expression(pair),
        Rule::function_call_term => {
            let mut inner = pair.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let mut args = Vec::new();

            for arg_list in inner {
                if arg_list.as_rule() == Rule::param_list {
                    for arg in arg_list.into_inner() {
                        args.push(parse_expression(arg)?);
                    }
                }
            }

            Ok(Expression::FunctionCall { name, args })
        },
        Rule::function_call_stmt => parse_expression(pair),
        Rule::member_access => {
            let mut parts = pair.into_inner();
            let object = Expression::Identifier(parts.next().unwrap().as_str().to_string());
            let member = parts.next().unwrap().as_str().to_string();

            Ok(Expression::MemberAccess {
                object: Box::new(object),
                member
            })
        },
        _ => Err(BythonParseError {
            message: format!("Unexpected rule for term: {:?}", pair.as_rule()),
        }),
    }
}

fn parse_operator(pair: Pair<Rule>) -> Result<Operator, BythonParseError> {
    match pair.as_str() {
        "+" => Ok(Operator::Add),
        "-" => Ok(Operator::Sub),
        "*" => Ok(Operator::Mul),
        "/" => Ok(Operator::Div),
        "==" => Ok(Operator::Eq),
        "!=" => Ok(Operator::NotEq),
        "<" => Ok(Operator::Lt),
        ">" => Ok(Operator::Gt),
        "<=" => Ok(Operator::LtEq),
        ">=" => Ok(Operator::GtEq),
        "and" => Ok(Operator::And),
        "or" => Ok(Operator::Or),
        "not" => Ok(Operator::Not),
        _ => Err(BythonParseError {
            message: format!("Unknown operator: {}", pair.as_str())
        })
    }
}

fn parse_block(pair: Pair<Rule>) -> Result<Block, BythonParseError> {
    if pair.as_rule() != Rule::block {
        return Err(BythonParseError {
            message: format!("Expected a block rule, got {:?}", pair.as_rule()),
        });
    }
    let mut statements = Vec::new();
    for inner_pair in pair.into_inner() {
        if inner_pair.as_rule() == Rule::statement {
            statements.push(parse_statement(inner_pair)?);
        }
    }
    Ok(Block { statements })
}