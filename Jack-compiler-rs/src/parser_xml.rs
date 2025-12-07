use serde::{
    Serialize, Serializer,
    ser::{SerializeMap, SerializeStruct},
};

use crate::{
    parser::{
        Class, ClassVarDec, ClassVarDecKind, DoStatement, Expression, ExpressionList, IfStatement,
        KeywordConstant, LetStatement, Op, ParameterList, ReturnStatement, Statement, Statements,
        SubroutineBody, SubroutineCall, SubroutineDec, SubroutineDecReturn, SubroutineDecType,
        Term, Type, UnaryOp, VarDec, WhileStatement,
    },
    tokenizer::{Constant, Identifier},
};

impl<'de> Serialize for Class<'de> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("class", 3 + self.class_var_decs.len())?;

        s.serialize_field("keyword", &"class")?;
        s.serialize_field("identifier", &self.class_name)?;
        s.serialize_field("symbol", &"{")?;
        for class_var_dec in self.class_var_decs.iter() {
            s.serialize_field("classVarDec", &class_var_dec)?;
        }
        for subroutine_dec in self.subroutine_decs.iter() {
            s.serialize_field("subroutineDec", &subroutine_dec)?;
        }
        s.serialize_field("symbol", &"}")?;

        s.end()
    }
}

impl<'de> Serialize for Identifier<'de> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_newtype_struct("identifier", &self.0)
    }
}

impl<'de> Serialize for ClassVarDec<'de> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("ClassVarDec", 2 + 2 * self.var_names.len())?;
        s.serialize_field("keyword", &self.class_var_dec_kind)?;

        match &self.class_var_dec_type {
            Type::Class { .. } => s.serialize_field("identifier", &self.class_var_dec_type)?,
            _ => s.serialize_field("keyword", &self.class_var_dec_type)?,
        }

        for (i, var_name) in self.var_names.iter().enumerate() {
            s.serialize_field("identifier", &var_name)?;

            if i + 1 == self.var_names.len() {
                s.serialize_field("symbol", &";")?;
            } else {
                s.serialize_field("symbol", &",")?;
            }
        }

        s.end()
    }
}

impl Serialize for ClassVarDecKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_newtype_struct("identifier", &format!("{:?}", self).to_lowercase())
    }
}

impl<'de> Serialize for Type<'de> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Type::Class { name } => name.serialize(serializer),
            elsewise => serializer.serialize_str(&format!("{:?}", elsewise).to_lowercase()),
        }
    }
}

impl<'de> Serialize for SubroutineDec<'de> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("SubroutineDec", 2)?;
        s.serialize_field("keyword", &self.subroutine_dec_type)?;

        match &self.subroutine_dec_return_type {
            SubroutineDecReturn::Void => {
                s.serialize_field("keyword", &self.subroutine_dec_return_type)?
            }
            SubroutineDecReturn::Type(r#type) => match r#type {
                Type::Class { name } => s.serialize_field("identifier", r#name)?,
                _ => s.serialize_field("keyword", r#type)?,
            },
        }
        s.serialize_field("identifier", &self.subroutine_name)?;

        s.serialize_field("symbol", &"(")?;
        if self.parameter_list.parameters.is_empty() {
            s.serialize_field("parameterList", &"\n")?;
        } else {
            s.serialize_field("parameterList", &self.parameter_list)?;
        }
        s.serialize_field("symbol", &")")?;

        s.serialize_field("subroutineBody", &self.subroutine_body)?;

        s.end()
    }
}

impl Serialize for SubroutineDecType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{:?}", self).to_lowercase())
    }
}

impl<'de> Serialize for SubroutineDecReturn<'de> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{:?}", self).to_lowercase())
    }
}

impl<'de> Serialize for ParameterList<'de> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("ParameterList", 0)?;
        for (i, (t, parameter)) in self.parameters.iter().enumerate() {
            match &t {
                Type::Class { name } => s.serialize_field("identifier", name)?,
                _ => s.serialize_field("keyword", t)?,
            };
            s.serialize_field("identifier", parameter)?;

            if i + 1 != self.parameters.len() {
                s.serialize_field("symbol", ",")?;
            }
        }
        s.end()
    }
}

impl<'de> Serialize for SubroutineBody<'de> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("SubroutineBody", 0)?;

        s.serialize_field("symbol", &"{")?;
        for var_dec in self.var_decs.iter() {
            s.serialize_field("varDec", &var_dec)?;
        }
        s.serialize_field("statements", &self.statements)?;
        s.serialize_field("symbol", &"}")?;

        s.end()
    }
}

impl<'de> Serialize for VarDec<'de> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("SubroutineBody", 0)?;

        s.serialize_field("keyword", &"var")?;
        match &self.var_type {
            Type::Class { .. } => s.serialize_field("identifier", &self.var_type)?,
            _ => s.serialize_field("keyword", &self.var_type)?,
        }
        for (i, var_name) in self.var_names.iter().enumerate() {
            s.serialize_field("identifier", &var_name)?;

            if i + 1 == self.var_names.len() {
                s.serialize_field("symbol", &";")?;
            } else {
                s.serialize_field("symbol", &",")?;
            }
        }

        s.end()
    }
}

impl<'de> Serialize for Statements<'de> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Statements", 0)?;
        for statement in self.statements.iter() {
            match statement {
                Statement::LetStatement(let_statement) => {
                    s.serialize_field("letStatement", let_statement)?
                }
                Statement::IfStatement(if_statement) => {
                    s.serialize_field("ifStatement", if_statement)?
                }
                Statement::WhileStatement(while_statement) => {
                    s.serialize_field("whileStatement", while_statement)?
                }
                Statement::DoStatement(do_statement) => {
                    s.serialize_field("doStatement", do_statement)?
                }
                Statement::ReturnStatement(return_statement) => {
                    s.serialize_field("returnStatement", return_statement)?
                }
            }
        }
        s.end()
    }
}

impl<'de> Serialize for Statement<'de> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Statement::LetStatement(let_statement) => let_statement.serialize(serializer),
            Statement::IfStatement(if_statement) => if_statement.serialize(serializer),
            Statement::WhileStatement(while_statement) => while_statement.serialize(serializer),
            Statement::DoStatement(do_statement) => do_statement.serialize(serializer),
            Statement::ReturnStatement(return_statement) => return_statement.serialize(serializer),
        }
    }
}

impl<'de> Serialize for WhileStatement<'de> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("IfStatement", 0)?;
        s.serialize_field("keyword", &"while")?;
        s.serialize_field("symbol", &"(")?;
        s.serialize_field("expression", &self.condition)?;
        s.serialize_field("symbol", &")")?;
        s.serialize_field("symbol", &"{")?;
        s.serialize_field("statements", &self.body)?;
        s.serialize_field("symbol", &"}")?;
        s.end()
    }
}

impl<'de> Serialize for IfStatement<'de> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("IfStatement", 0)?;
        s.serialize_field("keyword", &"if")?;
        s.serialize_field("symbol", &"(")?;
        s.serialize_field("expression", &self.condition)?;
        s.serialize_field("symbol", &")")?;
        s.serialize_field("symbol", &"{")?;
        s.serialize_field("statements", &self.then_branch)?;
        s.serialize_field("symbol", &"}")?;
        if let Some(else_branch) = &self.else_branch {
            s.serialize_field("keyword", &"else")?;
            s.serialize_field("symbol", &"{")?;
            s.serialize_field("statements", else_branch)?;
            s.serialize_field("symbol", &"}")?;
        }
        s.end()
    }
}

impl<'de> Serialize for ReturnStatement<'de> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("ReturnStatement", 0)?;
        s.serialize_field("keyword", &"return")?;
        if let Some(expression) = &self.expression {
            s.serialize_field("expression", expression)?;
        }
        s.serialize_field("symbol", &";")?;
        s.end()
    }
}

impl<'de> Serialize for DoStatement<'de> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("DoStatement", 0)?;
        s.serialize_field("keyword", &"do")?;

        match &self.subroutine_call {
            SubroutineCall::Call {
                subroutine_name,
                expression_list,
            } => {
                s.serialize_field("identifier", &subroutine_name)?;
                s.serialize_field("symbol", &"(")?;
                if expression_list.expressions.is_empty() {
                    s.serialize_field("expressionList", &"\n")?;
                } else {
                    s.serialize_field("expressionList", &expression_list)?;
                }
                s.serialize_field("symbol", &")")?;
            }
            SubroutineCall::ClassCall {
                class_or_var_name,
                subroutine_name,
                expression_list,
            } => {
                s.serialize_field("identifier", &class_or_var_name)?;
                s.serialize_field("symbol", &".")?;
                s.serialize_field("identifier", &subroutine_name)?;
                s.serialize_field("symbol", &"(")?;
                if expression_list.expressions.is_empty() {
                    s.serialize_field("expressionList", &"\n")?;
                } else {
                    s.serialize_field("expressionList", &expression_list)?;
                }
                s.serialize_field("symbol", &")")?;
            }
        };
        s.serialize_field("symbol", &";")?;
        s.end()
    }
}

impl<'de> Serialize for LetStatement<'de> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("LetStatement", 0)?;
        s.serialize_field("keyword", &"let")?;
        s.serialize_field("identifier", &self.var_name)?;
        if let Some(expression_1) = &self.expression_1 {
            s.serialize_field("symbol", &"[")?;
            s.serialize_field("expression", &expression_1)?;
            s.serialize_field("symbol", &"]")?;
        }
        s.serialize_field("symbol", &"=")?;
        s.serialize_field("expression", &self.expression_2)?;
        s.serialize_field("symbol", &";")?;
        s.end()
    }
}

impl<'de> Serialize for Expression<'de> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Expression", 0)?;
        s.serialize_field("term", &self.term)?;
        for (op, term) in self.terms.iter() {
            s.serialize_field("symbol", op)?;
            s.serialize_field("term", term)?;
        }
        s.end()
    }
}

impl Serialize for Op {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Op::Plus => serializer.serialize_str("+"),
            Op::Minus => serializer.serialize_str("-"),
            Op::Asterisk => serializer.serialize_str("*"),
            Op::Slash => serializer.serialize_str("/"),
            Op::Ampersand => serializer.serialize_str("&"),
            Op::Pipe => serializer.serialize_str("|"),
            Op::LessThan => serializer.serialize_str("<"),
            Op::GreaterThan => serializer.serialize_str(">"),
            Op::Equal => serializer.serialize_str("="),
        }
    }
}

impl<'de> Serialize for Term<'de> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Term::Constant(constant) => constant.serialize(serializer),
            Term::KeywordConstant(keyword_constant) => keyword_constant.serialize(serializer),
            Term::VarName(identifier) => {
                let mut s = serializer.serialize_map(Some(1))?;
                s.serialize_entry("identifier", identifier)?;
                s.end()
            }
            Term::VarNameExpression {
                var_name,
                expression,
            } => {
                let mut s = serializer.serialize_map(Some(4))?;
                s.serialize_entry("identifier", var_name)?;
                s.serialize_entry("symbol", "[")?;
                s.serialize_entry("expression", expression)?;
                s.serialize_entry("symbol", "]")?;
                s.end()
            }
            Term::Expression(expression) => {
                let mut s = serializer.serialize_map(Some(3))?;
                s.serialize_entry("symbol", "(")?;
                s.serialize_entry("expression", expression)?;
                s.serialize_entry("symbol", ")")?;
                s.end()
            }
            Term::UnaryOpTerm { unary_op, term } => {
                let mut s = serializer.serialize_map(Some(2))?;
                s.serialize_entry("symbol", unary_op)?;
                s.serialize_entry("term", term)?;
                s.end()
            }
            Term::SubroutineCall(subroutine_call) => subroutine_call.serialize(serializer),
        }
    }
}

impl Serialize for UnaryOp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            UnaryOp::Minus => serializer.serialize_str("-"),
            UnaryOp::Tilde => serializer.serialize_str("~"),
        }
    }
}

impl<'de> Serialize for KeywordConstant {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("KeywordConstant", 0)?;
        s.serialize_field("keyword", &format!("{:?}", self).to_lowercase())?;
        s.end()
    }
}

impl<'de> Serialize for Constant<'de> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Constant", 0)?;
        match self {
            Constant::String(cow) => s.serialize_field("stringConstant", &cow)?,
            Constant::Integer(i) => s.serialize_field("integerConstant", i)?,
        };
        s.end()
    }
}

impl<'de> Serialize for SubroutineCall<'de> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("SubroutineCall", 0)?;
        match self {
            SubroutineCall::Call {
                subroutine_name,
                expression_list,
            } => {
                s.serialize_field("identifier", &subroutine_name)?;
                s.serialize_field("symbol", &"(")?;
                if expression_list.expressions.is_empty() {
                    s.serialize_field("expressionList", &"\n")?;
                } else {
                    s.serialize_field("expressionList", &expression_list)?;
                }
                s.serialize_field("symbol", &")")?;
            }
            SubroutineCall::ClassCall {
                class_or_var_name,
                subroutine_name,
                expression_list,
            } => {
                s.serialize_field("identifier", &class_or_var_name)?;
                s.serialize_field("symbol", &".")?;
                s.serialize_field("identifier", &subroutine_name)?;
                s.serialize_field("symbol", &"(")?;
                if expression_list.expressions.is_empty() {
                    s.serialize_field("expressionList", &"\n")?;
                } else {
                    s.serialize_field("expressionList", &expression_list)?;
                }
                s.serialize_field("symbol", &")")?;
            }
        };
        s.end()
    }
}

impl<'de> Serialize for ExpressionList<'de> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("ExpressionList", 1)?;
        for (i, expression) in self.expressions.iter().enumerate() {
            s.serialize_field("expression", expression)?;
            if i + 1 != self.expressions.len() {
                s.serialize_field("symbol", ",")?;
            }
        }
        s.end()
    }
}
