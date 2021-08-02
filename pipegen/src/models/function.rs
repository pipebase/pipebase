use super::{
    data_ty_to_literal,
    meta::{meta_to_literal, Meta},
    utils::indent_literal,
    DataField, DataType, Entity,
};

pub enum Rhs {
    Expr(String),
    Block(Block),
}

fn rhs_to_lit(rhs: &Rhs, indent: usize) -> String {
    let indent_lit = indent_literal(indent);
    match rhs {
        Rhs::Expr(expr) => format!("{}{}", indent_lit, expr),
        Rhs::Block(block) => block.to_literal(indent),
    }
}

pub struct Statement {
    lhs: Option<String>,
    rhs: Rhs,
}

impl Statement {
    pub fn new(lhs: Option<String>, rhs: Rhs) -> Self {
        Statement { lhs, rhs }
    }

    pub fn to_literal(&self, indent: usize) -> String {
        let lhs = match self.lhs {
            Some(ref lhs) => lhs,
            None => return rhs_to_lit(&self.rhs, indent),
        };
        let indent_lit = indent_literal(indent);
        match self.rhs {
            Rhs::Expr(_) => {
                let rhs_lit = rhs_to_lit(&self.rhs, 0);
                return format!("{}let {} = {}", indent_lit, lhs, rhs_lit);
            }
            Rhs::Block(_) => {
                let rhs_lit = rhs_to_lit(&self.rhs, indent + 1);
                return format!(
                    "{}let {} = {{\n{}\n{}}}",
                    indent_lit, lhs, rhs_lit, indent_lit
                );
            }
        }
    }
}

pub struct Block {
    statements: Vec<Statement>,
}

impl Block {
    pub fn new(statements: Vec<Statement>) -> Self {
        Block { statements }
    }

    pub fn to_literal(&self, indent: usize) -> String {
        let mut statement_lits: Vec<String> = Vec::new();
        for statement in &self.statements {
            statement_lits.push(statement.to_literal(indent));
        }
        statement_lits.join(";\n")
    }
}

pub struct Function {
    // TODO: Support generics
    name: String,
    meta: Option<Meta>,
    is_public: bool,
    is_async: bool,
    args: Vec<DataField>,
    block: Block,
    // return type
    rtype: Option<DataType>,
}

impl Function {
    pub(crate) fn to_literal(&self, indent: usize) -> String {
        let signature_lit = self.get_signature_literal(indent);
        let block_lit = self.block.to_literal(indent + 1);
        let indent_lit = indent_literal(indent);
        let function_lit = format!("{} {{\n{}\n{}}}", signature_lit, block_lit, indent_lit);
        match self.get_metas_literal(indent) {
            Some(meta_lits) => format!("{}\n{}", meta_lits, function_lit),
            None => function_lit,
        }
    }

    fn get_signature_literal(&self, indent: usize) -> String {
        let mut lits: Vec<String> = Vec::new();
        if self.is_public {
            lits.push("pub".to_owned())
        }
        if self.is_async {
            lits.push("async".to_owned())
        }
        lits.push("fn".to_owned());
        let input_lit = self.get_input_literal();
        lits.push(format!("{}({})", self.name, input_lit));
        if let Some(rtype_lit) = self.get_rtype_literal() {
            lits.push(rtype_lit)
        }
        let indent_lit = indent_literal(indent);
        format!("{}{}", indent_lit, lits.join(" "))
    }

    fn get_input_literal(&self) -> String {
        let mut lits: Vec<String> = Vec::new();
        for arg in &self.args {
            lits.push(arg.to_literal(0));
        }
        lits.join(", ")
    }

    fn get_rtype_literal(&self) -> Option<String> {
        let rtype = match self.rtype {
            Some(ref rtype) => rtype,
            None => return None,
        };
        Some(format!("-> {}", data_ty_to_literal(rtype)))
    }

    fn get_metas_literal(&self, indent: usize) -> Option<String> {
        let meta = match self.meta {
            Some(ref meta) => meta,
            None => return None,
        };
        Some(meta_to_literal(meta, indent))
    }
}

#[derive(Default)]
pub struct FunctionBuilder {
    // TODO: Support generics
    name: Option<String>,
    meta: Option<Meta>,
    is_public: bool,
    is_async: bool,
    args: Vec<DataField>,
    block: Option<Block>,
    // return type
    rtype: Option<DataType>,
}

impl FunctionBuilder {
    pub fn new() -> Self {
        FunctionBuilder::default()
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn meta(mut self, meta: Meta) -> Self {
        self.meta = Some(meta);
        self
    }

    pub fn public(mut self, is_public: bool) -> Self {
        self.is_public = is_public;
        self
    }

    pub fn asynchronous(mut self, is_async: bool) -> Self {
        self.is_async = is_async;
        self
    }

    pub fn args(mut self, args: Vec<DataField>) -> Self {
        self.args = args;
        self
    }

    pub fn block(mut self, block: Block) -> Self {
        self.block = Some(block);
        self
    }

    pub fn rtype(mut self, rtype: DataType) -> Self {
        self.rtype = Some(rtype);
        self
    }

    pub fn build(self) -> Function {
        Function {
            name: self.name.expect("function name not inited"),
            meta: self.meta,
            is_public: self.is_public,
            is_async: self.is_async,
            args: self.args,
            block: self.block.expect("function block not inited"),
            rtype: self.rtype,
        }
    }
}
