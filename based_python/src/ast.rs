#[derive(Debug)]
pub enum Statement {
    Assignment {
        name: String,
        value: Expression,
    },
    Print {
        content: Expression,
    },
    Return {
        value: Expression,
    },
    If {
        condition: Expression,
        consequence: Block,
        alternative: Option<Block>,
    },
    For {
        iterator: Expression,
        body: Block,
    },
    While {
        condition: Expression,
        body: Block,
    },
    FunctionDef {
        name: String,
        args: Vec<String>,
        body: Block,
    },
    ClassDef {
        name: String,
        body: Block,
    },
}

#[derive(Debug)]
pub enum Expression {
    Identifier(String),
    Number(f64),
    String(String),
    BinaryOp {
        left: Box<Expression>,
        operator: String,
        right: Box<Expression>,
    },
    MemberAccess {
        object: Box<Expression>,
        member: String,
    },
}

#[derive(Debug)]
pub struct Block {
    pub statements: Vec<Statement>,
}

#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
}