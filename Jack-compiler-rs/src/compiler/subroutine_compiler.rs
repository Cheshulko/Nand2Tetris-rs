use crate::{
    compiler::{
        ClassCompiler, Pad,
        symbol_table::{SubroutineSymbolTableState, SymbolTable},
    },
    parser::{
        DoStatement, Expression, ExpressionList, IfStatement, KeywordConstant, LetStatement, Op,
        ParameterList, ReturnStatement, Statement, Statements, SubroutineCall, SubroutineDec,
        SubroutineDecType, Term, Type, UnaryOp, VarDec, WhileStatement,
    },
    tokenizer::{Constant, Identifier},
};
use std::fmt::Write;

macro_rules! write_pad {
    ($dst:expr, $($arg:tt)*) => {
        $dst.write_pad(format_args!($($arg)*))
    };
}

pub(super) struct SubroutineCompiler<'de, 'a> {
    class_compiler: &'a mut ClassCompiler<'de>,
    symbol_table: SymbolTable<'de, SubroutineSymbolTableState>,
    output: Vec<String>,

    pad: Pad,
}

impl<'de, 'a> SubroutineCompiler<'de, 'a> {
    fn write_pad(&mut self, args: std::fmt::Arguments) -> anyhow::Result<()> {
        let mut result = String::new();
        write!(&mut result, "{}{}", self.pad, args)?;
        self.output.push(result);

        Ok(())
    }

    pub fn compile(
        class_compiler: &'a mut ClassCompiler<'de>,
        subroutine_dec: &'de SubroutineDec<'_>,
    ) -> anyhow::Result<Vec<String>> {
        let mut compiler = Self {
            class_compiler,
            symbol_table: SymbolTable::new_subroutine_symbol_table(),
            output: vec![],
            pad: Pad::None,
        };

        let class_name = compiler.class_compiler.get_class().class_name.0;
        compiler.compile_subroutine_dec(class_name, subroutine_dec)?;

        Ok(compiler.output)
    }

    fn compile_subroutine_dec(
        &mut self,
        class_name: &str,
        subroutine_dec: &'de SubroutineDec<'_>,
    ) -> anyhow::Result<()> {
        {
            let subroutine_name = subroutine_dec.subroutine_name.0;
            let local_args_cnt = subroutine_dec
                .subroutine_body
                .var_decs
                .iter()
                .map(|var_dec| var_dec.var_names.len())
                .sum::<usize>();

            match subroutine_dec.subroutine_dec_type {
                SubroutineDecType::Constructor => {
                    write_pad!(
                        self,
                        "function {class_name}.{subroutine_name} {local_args_cnt}"
                    )?;

                    {
                        self.pad = Pad::One;
                        // calculate the size of an instance
                        let cnt = self.class_compiler.get_fields_cnt();
                        write_pad!(self, "push constant {cnt}")?;
                        write_pad!(self, "call Memory.alloc 1")?;
                        // pop the allocated memory into `this`
                        write_pad!(self, "pop pointer 0")?;
                        self.pad = Pad::None;
                    }
                }
                SubroutineDecType::Function => write_pad!(
                    self,
                    "function {class_name}.{subroutine_name} {local_args_cnt}"
                )?,
                SubroutineDecType::Method => {
                    write_pad!(
                        self,
                        "function {class_name}.{subroutine_name} {local_args_cnt}"
                    )?;
                    self.pad = Pad::One;
                    // Push a fake `this`` parameter to shift the other arguments so they start at index 1
                    self.symbol_table
                        .insert_argument(&Identifier("this"), &Type::Boolean);
                    write_pad!(self, "push argument 0")?;
                    // pop the allocated memory into `this`
                    write_pad!(self, "pop pointer 0")?;
                    self.pad = Pad::None;
                }
            };
        }

        self.compile_parameter_list(&subroutine_dec.parameter_list)?;

        {
            self.pad = Pad::One;
            for var_dec in subroutine_dec.subroutine_body.var_decs.iter() {
                self.compile_var_dec(var_dec)?;
            }
            self.pad = Pad::None;
        }

        {
            self.pad = Pad::One;
            let statements = &subroutine_dec.subroutine_body.statements;
            self.compile_statements(statements)?;
            self.pad = Pad::None;
        }

        Ok(())
    }

    fn compile_parameter_list(
        &mut self,
        parameter_list: &'de ParameterList<'de>,
    ) -> anyhow::Result<()> {
        for (r#type, identifier) in parameter_list.parameters.iter() {
            self.symbol_table.insert_argument(identifier, r#type);
        }

        Ok(())
    }

    fn compile_var_dec(&mut self, var_dec: &'de VarDec<'de>) -> anyhow::Result<()> {
        let r#type = &var_dec.var_type;

        for var_name in var_dec.var_names.iter() {
            self.symbol_table.insert_var(var_name, r#type);
        }

        Ok(())
    }

    fn compile_statements(&mut self, statements: &'de Statements<'_>) -> anyhow::Result<()> {
        for statement in statements.statements.iter() {
            self.compile_statement(statement)?;
        }

        Ok(())
    }

    fn compile_statement(&mut self, statement: &'de Statement<'_>) -> anyhow::Result<()> {
        match statement {
            Statement::LetStatement(let_statement) => self.compile_let_statement(let_statement),
            Statement::IfStatement(if_statement) => self.compile_if_statement(if_statement),
            Statement::WhileStatement(while_statement) => {
                self.compile_while_statement(while_statement)
            }
            Statement::DoStatement(do_statement) => self.compile_do_statement(do_statement),
            Statement::ReturnStatement(return_statement) => {
                self.compile_return_statement(return_statement)
            }
        }
    }

    fn search_var(
        &self,
        var_name: &'de Identifier<'_>,
    ) -> anyhow::Result<(&'static str, usize, Option<&'de str>)> {
        let (var_segment_name, var_segment_index, r#type) = 
        // searching in the class's `fields` symbol table
        if let Some(&(r#type, field_index)) = self.class_compiler.get_field(var_name) {
            println!(
                "[debug] Found {:?} in the class's `fields` table",
                var_name
            );

            ("this", field_index, r#type)
        } else {
            // Searching in the coroutine's `vars` symbol table
            if let Some(&(r#type, var_index)) = self.symbol_table.get_var(var_name) {
                println!(
                    "[debug] Found {:?} in the subroutine's `vars` table",
                    var_name
                );

                ("local", var_index, r#type)
            } else {
                // Searching in the coroutine's `args` symbol table
                if let Some(&(r#type, arg_index)) =
                    self.symbol_table.get_argument(var_name)
                {
                    println!(
                        "[debug] Found {:?} in the subroutine's `args` table",
                        var_name
                    );

                    ("argument", arg_index, r#type)
                } else {
                    // Searching in the class's `statics` symbol table
                    if let Some(&(r#type, static_index)) =
                        self.class_compiler.get_static(var_name)
                    {
                        println!(
                            "[debug] Found {:?} in the subroutine's `statics` table",
                            var_name
                        );

                        ("static", static_index, r#type)
                    } else {
                        println!("[debug] Could not complete assignment for the let statement: {:?}. Ok. It's either a class constructor or a class function call", var_name);

                        anyhow::bail!(
                            "Could not find {} in any symbol table",
                            &var_name.0,
                        );
                    }
                }
            }
        };

        let var_segment_type = match r#type {
            Type::Class { name } => Some(name.0),
            _ => None,
        };

        Ok((var_segment_name, var_segment_index, var_segment_type))
    }

    fn compile_let_statement(
        &mut self,
        let_statement: &'de LetStatement<'_>,
    ) -> anyhow::Result<()> {
        let (var_segment_name, var_segment_index, _) = self.search_var(&let_statement.var_name)?;

        if let Some(expression_1) = &let_statement.expression_1 {
            self.compile_expression(expression_1)?;
            write_pad!(self, "push {} {}", var_segment_name, var_segment_index)?;
            write_pad!(self, "add")?;

            self.compile_expression(&let_statement.expression_2)?;
            write_pad!(self, "pop temp 0")?;

            write_pad!(self, "pop pointer 1")?;
            write_pad!(self, "push temp 0")?;
            write_pad!(self, "pop that 0")
        } else {
            self.compile_expression(&let_statement.expression_2)?;

            write_pad!(self, "pop {} {}", var_segment_name, var_segment_index)
        }
    }

    fn compile_if_statement(&mut self, if_statement: &'de IfStatement<'_>) -> anyhow::Result<()> {
        self.compile_expression(&if_statement.condition)?;
        write_pad!(self, "not")?;

        let label_then = self.class_compiler.create_new_label();
        let label_else = self.class_compiler.create_new_label();

        write_pad!(self, "if-goto {label_else}")?;
        self.compile_statements(&if_statement.then_branch)?;
        write_pad!(self, "goto {label_then}")?;
        {
            self.pad = Pad::None;
            write_pad!(self, "label {label_else}")?;
            self.pad = Pad::One;
        }
        if let Some(else_branch) = &if_statement.else_branch {
            self.compile_statements(else_branch)?;
        }
        {
            self.pad = Pad::None;
            write_pad!(self, "label {label_then}")?;
            self.pad = Pad::One;
        }

        Ok(())
    }

    fn compile_while_statement(
        &mut self,
        while_statement: &'de WhileStatement<'_>,
    ) -> anyhow::Result<()> {
        let label_yes = self.class_compiler.create_new_label();
        let label_no = self.class_compiler.create_new_label();

        {
            self.pad = Pad::None;
            write_pad!(self, "label {label_yes}")?;
            self.pad = Pad::One;
        }
        self.compile_expression(&while_statement.condition)?;
        write_pad!(self, "not")?;
        write_pad!(self, "if-goto {label_no}")?;
        self.compile_statements(&while_statement.body)?;
        write_pad!(self, "goto {label_yes}")?;
        {
            self.pad = Pad::None;
            write_pad!(self, "label {label_no}")?;
            self.pad = Pad::One;
        }

        Ok(())
    }

    fn compile_do_statement(&mut self, do_statement: &'de DoStatement<'_>) -> anyhow::Result<()> {
        self.compile_subroutine_call(&do_statement.subroutine_call)?;

        write_pad!(self, "pop temp 0")?;

        Ok(())
    }

    fn compile_return_statement(
        &mut self,
        return_statement: &'de ReturnStatement<'_>,
    ) -> anyhow::Result<()> {
        if let Some(expression) = &return_statement.expression {
            self.compile_expression(expression)?;
        } else {
            // Add a constant 0 as return value
            write_pad!(self, "push constant 0")?;
        }

        write_pad!(self, "return")?;

        Ok(())
    }

    fn compile_expression_list(
        &mut self,
        expression_list: &'de ExpressionList<'_>,
    ) -> anyhow::Result<()> {
        for expression in expression_list.expressions.iter() {
            self.compile_expression(expression)?;
        }

        Ok(())
    }

    fn compile_expression(&mut self, expression: &'de Expression<'_>) -> anyhow::Result<()> {
        let term = &expression.term;
        self.compile_term(term)?;

        for (op, term) in expression.terms.iter() {
            self.compile_term(term)?;
            self.compile_op(op)?;
        }

        Ok(())
    }

    fn compile_term(&mut self, term: &'de Term<'_>) -> anyhow::Result<()> {
        match term {
            Term::Constant(constant) => match constant {
                Constant::String(cow) => {
                    let len = cow.len();
                    write_pad!(self, "push constant {len}")?;
                    write_pad!(self, "call String.new 1")?;
                    for b in cow.as_bytes() {
                        write_pad!(self, "push constant {}", b)?;
                        write_pad!(self, "call String.appendChar 2")?;
                    }

                    Ok(())
                }
                Constant::Integer(i) => write_pad!(self, "push constant {}", i),
            },
            Term::KeywordConstant(keyword_constant) => match keyword_constant {
                KeywordConstant::True => {
                    write_pad!(self, "push constant 1")?;
                    write_pad!(self, "neg")
                }
                KeywordConstant::False => write_pad!(self, "push constant 0"),
                KeywordConstant::Null => write_pad!(self, "push constant 0"),
                KeywordConstant::This => write_pad!(self, "push pointer 0"),
            },
            Term::VarName(identifier) => {
                let (var_segment_name, var_segment_index, _) = self.search_var(&identifier)?;
                write_pad!(self, "push {} {}", var_segment_name, var_segment_index)
            }
            Term::VarNameExpression {
                var_name,
                expression,
            } => {
                let (var_segment_name, var_segment_index, _) = self.search_var(&var_name)?;

                self.compile_expression(expression)?;
                write_pad!(self, "push {} {}", var_segment_name, var_segment_index)?;
                write_pad!(self, "add")?;
                write_pad!(self, "pop pointer 1")?;
                write_pad!(self, "push that 0")
            }
            Term::Expression(expression) => self.compile_expression(expression),
            Term::UnaryOpTerm { unary_op, term } => {
                self.compile_term(term)?;
                self.compile_unary_op(unary_op)
            }
            Term::SubroutineCall(subroutine_call) => self.compile_subroutine_call(subroutine_call),
        }
    }

    fn compile_unary_op(&mut self, unary_op: &UnaryOp) -> anyhow::Result<()> {
        match unary_op {
            UnaryOp::Minus => write_pad!(self, "neg"),
            UnaryOp::Tilde => write_pad!(self, "not"),
        }
    }

    fn compile_subroutine_call(
        &mut self,
        subroutine_call: &'de SubroutineCall<'_>,
    ) -> anyhow::Result<()> {
        match subroutine_call {
            SubroutineCall::Call {
                subroutine_name,
                expression_list,
            } => {
                write_pad!(self, "push pointer 0")?;
                self.compile_expression_list(expression_list)?;

                let class_name = self.class_compiler.get_class().class_name.0;
                let args_cnt = expression_list.expressions.len() + 1 /* for `this` */;
                write_pad!(
                    self,
                    "call {}.{} {}",
                    class_name,
                    subroutine_name.0,
                    args_cnt
                )?;
            }
            SubroutineCall::ClassCall {
                class_or_var_name,
                subroutine_name,
                expression_list,
            } => {
                let mut args_cnt = expression_list.expressions.len();
                let target_name;

                if let Ok((var_segment_name, var_segment_index, var_segment_type)) =
                    self.search_var(&class_or_var_name)
                {
                    let Some(var_segment_type) = var_segment_type else {
                        panic!("Could not parse `var_segment_type` at `SubroutineCall::ClassCall`");
                    };

                    write_pad!(self, "push {} {}", var_segment_name, var_segment_index)?;

                    target_name = var_segment_type;
                    args_cnt += 1; /* arg0 - is `this` */
                } else {
                    // Ok. It's either a class constructor or a class function call.
                    target_name = class_or_var_name.0;
                }

                self.compile_expression_list(expression_list)?;

                write_pad!(
                    self,
                    "call {}.{} {}",
                    target_name,
                    subroutine_name.0,
                    args_cnt
                )?;
            }
        };

        Ok(())
    }

    fn compile_op(&mut self, op: &Op) -> anyhow::Result<()> {
        match op {
            Op::Plus => write_pad!(self, "add"),
            Op::Minus => write_pad!(self, "sub"),
            Op::Asterisk => write_pad!(self, "call Math.multiply 2"),
            Op::Slash => write_pad!(self, "call Math.divide 2"),
            Op::Ampersand => write_pad!(self, "and"),
            Op::Pipe => write_pad!(self, "or"),
            Op::LessThan => write_pad!(self, "lt"),
            Op::GreaterThan => write_pad!(self, "gt"),
            Op::Equal => write_pad!(self, "eq"),
        }
    }
}
