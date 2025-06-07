use pest::Parser;
use pest::iterators::{Pair, Pairs};
use crate::ast::{Program, Statement, Expression, Block};
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
            // Comparison operators
            .op(Op::infix(Rule::expression, Assoc::Left))
            // .op(Op::infix(Rule::CompOp, Assoc::Left))
            // // Addition and subtraction
            // .op(Op::infix(Rule::AddOp, Assoc::Left))
            // .op(Op::infix(Rule::SubOp, Assoc::Left))
            // // Multiplication and division
            // .op(Op::infix(Rule::MulOp, Assoc::Left))
            // .op(Op::infix(Rule::DivOp, Assoc::Left))
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
            let name = inner.next().unwrap().as_str().to_string();
            let value_pair = inner.next().unwrap();
            let value = parse_expression(value_pair)?;
            Ok(Statement::Assignment { name, value })
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
            let condition = parse_expression(inner.next().unwrap())?;
            let consequence = parse_block(inner.next().unwrap())?;
            let alternative = inner.next().map(|p| parse_block(p)).transpose()?;
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
            let name = inner.next().unwrap().as_str().to_string();
            let mut args = Vec::new();
            if let Some(arg_list_pair) = inner.next() {
                for arg_pair in arg_list_pair.into_inner() {
                    if arg_pair.as_rule() == Rule::ident {
                        args.push(arg_pair.as_str().to_string());
                    }
                }
            }
            let body = parse_block(inner.next().unwrap())?;
            Ok(Statement::FunctionDef { name, args, body })
        }
        Rule::class_def => {
            let mut inner = pair.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let body = parse_block(inner.next().unwrap())?;
            Ok(Statement::ClassDef { name, body })
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
            let first = parts.next().unwrap();
            let mut expr = Expression::Identifier(first.as_str().to_string());

            for member in parts {
                expr = Expression::MemberAccess {
                    object: Box::new(expr),
                    member: member.as_str().to_string(),
                };
            }
            Ok(expr)
        },
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
        _ => Err(BythonParseError {
            message: format!("Unexpected rule for term: {:?}", pair.as_rule()),
        }),
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