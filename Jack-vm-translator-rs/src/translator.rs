use crate::parser::{Node, Segment};

macro_rules! c {
    ($vec:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {
        $vec.push(format!($fmt $(, $arg)*));
    };
    ($vec:expr, $($fmt:expr $(, $arg:expr)*);+ $(;)?) => {
        $(
            $vec.push(format!($fmt $(, $arg)*));
        )+
    };
}

pub struct Translator<'de, I: IntoIterator<Item = Node<'de>>, S: AsRef<str>> {
    filename: S,
    nodes: I,
}

impl<'de, I, S> Translator<'de, I, S>
where
    I: IntoIterator<Item = Node<'de>>,
    S: AsRef<str>,
{
    pub fn new(filename: S, nodes: I) -> Self {
        Self { filename, nodes }
    }

    pub fn translate(self) -> Vec<String> {
        let filename = self.filename;
        let nodes = self.nodes;

        let mut label_cnt = 0;

        nodes.into_iter().fold(vec![], |mut ans, node| match node {
            Node::Push { segment } => match segment {
                Segment::Argument { offset } => {
                    load_mem_with_offset_into_d(&mut ans, "ARG", offset);
                    push_d_onto_stack(&mut ans);

                    ans
                }
                Segment::Local { offset } => {
                    load_mem_with_offset_into_d(&mut ans, "LCL", offset);
                    push_d_onto_stack(&mut ans);

                    ans
                }
                Segment::Static { offset } => {
                    c!(&mut ans, "@{}.{}", filename.as_ref(), offset; "D=M");
                    push_d_onto_stack(&mut ans);

                    ans
                }
                Segment::Constant { value } => {
                    c!(&mut ans, "@{}", value; "D=A");
                    push_d_onto_stack(&mut ans);

                    ans
                }
                Segment::This { offset } => {
                    load_mem_with_offset_into_d(&mut ans, "THIS", offset);
                    push_d_onto_stack(&mut ans);

                    ans
                }
                Segment::That { offset } => {
                    load_mem_with_offset_into_d(&mut ans, "THAT", offset);
                    push_d_onto_stack(&mut ans);

                    ans
                }
                Segment::Pointer { offset } => match offset {
                    0 => {
                        c!(&mut ans, "@THIS"; "D=M");
                        push_d_onto_stack(&mut ans);

                        ans
                    }
                    1 => {
                        c!(&mut ans, "@THAT"; "D=M");
                        push_d_onto_stack(&mut ans);

                        ans
                    }
                    _ => panic!(),
                },
                Segment::Temp { offset } => {
                    c!(&mut ans, "@{}", 5 + offset; "D=M");
                    push_d_onto_stack(&mut ans);

                    ans
                }
            },
            Node::Pop { segment } => match segment {
                Segment::Argument { offset } => {
                    load_sp_into_mem_with_offset(&mut ans, "ARG", offset);

                    ans
                }
                Segment::Local { offset } => {
                    load_sp_into_mem_with_offset(&mut ans, "LCL", offset);

                    ans
                }
                Segment::Static { offset } => {
                    sp_dec(&mut ans);
                    load_sp_into_d(&mut ans);
                    c!(&mut ans, "@{}.{}", filename.as_ref(), offset; "M=D");

                    ans
                }
                Segment::Constant { .. } => panic!("Not valid"),
                Segment::This { offset } => {
                    load_sp_into_mem_with_offset(&mut ans, "THIS", offset);

                    ans
                }
                Segment::That { offset } => {
                    load_sp_into_mem_with_offset(&mut ans, "THAT", offset);

                    ans
                }
                Segment::Pointer { offset } => match offset {
                    0 => {
                        pop_stack_into_d(&mut ans);
                        c!(&mut ans, "@THIS"; "M=D");

                        ans
                    }
                    1 => {
                        pop_stack_into_d(&mut ans);
                        c!(&mut ans, "@THAT"; "M=D");

                        ans
                    }
                    _ => panic!(),
                },
                Segment::Temp { offset } => {
                    pop_stack_into_d(&mut ans);
                    c!(&mut ans, "@{}", 5 + offset; "M=D");

                    ans
                }
            },
            Node::Label { name } => {
                c!(&mut ans, "({}.{})", filename.as_ref(), name);

                ans
            }
            Node::IfGoto { name } => {
                pop_stack_into_d(&mut ans);
                c!(&mut ans, "@{}.{}", filename.as_ref(), name; "D;JNE");

                ans
            }
            Node::Goto { name } => {
                c!(&mut ans, "@{}.{}", filename.as_ref(), name; "0;JMP");

                ans
            }
            Node::Function { name, n_locals } => {
                c!(&mut ans, "({})", name);
                c!(&mut ans, "@0"; "D=A");
                for _ in 0..n_locals {
                    push_d_onto_stack(&mut ans);
                }

                ans
            }
            Node::Return => {
                c!(&mut ans, "// endFrame - LCL");
                c!(&mut ans, "@LCL"; "D=M"; "@endFrame"; "M=D");

                c!(&mut ans, "// retAddr = *(endFrame - 5)");
                c!(&mut ans, "@5"; "D=A");
                c!(&mut ans, "@endFrame");
                c!(&mut ans, "D=M-D"; "A=D"; "D=M");
                c!(&mut ans, "@retAddr"; "M=D");

                c!(&mut ans, "// *ARG = pop()");
                pop_stack_into_d(&mut ans);
                c!(&mut ans, "@ARG"; "A=M"; "M=D");

                c!(&mut ans, "// SP = ARG + 1");
                c!(&mut ans, "@ARG"; "D=M"; "D=D+1"; "@SP"; "M=D");

                c!(&mut ans, "// THAT = *(endFrame - 1)");
                c!(&mut ans, "@1"; "D=A");
                c!(&mut ans, "@endFrame"; "D=M-D"; "A=D"; "D=M");
                c!(&mut ans, "@THAT"; "M=D");

                c!(&mut ans, "// THIS = *(endFrame - 2)");
                c!(&mut ans, "@2"; "D=A");
                c!(&mut ans, "@endFrame"; "D=M-D"; "A=D"; "D=M");
                c!(&mut ans, "@THIS"; "M=D");

                c!(&mut ans, "// ARG = *(endFrame - 3)");
                c!(&mut ans, "@3"; "D=A");
                c!(&mut ans, "@endFrame"; "D=M-D"; "A=D"; "D=M");
                c!(&mut ans, "@ARG"; "M=D");

                c!(&mut ans, "// LCL = *(endFrame - 4)");
                c!(&mut ans, "@4"; "D=A");
                c!(&mut ans, "@endFrame"; "D=M-D"; "A=D"; "D=M");
                c!(&mut ans, "@LCL"; "M=D");

                c!(&mut ans, "// goto retAddr");
                c!(&mut ans, "@retAddr"; "A=M"; "0;JMP");

                ans
            }
            Node::Call { name, n_args } => {
                c!(&mut ans, "// push returnAddress");
                c!(&mut ans, "@{}.{}.return.{}", filename.as_ref(), name, label_cnt; "D=A");
                push_d_onto_stack(&mut ans);

                c!(&mut ans, "// push LCL");
                c!(&mut ans, "@LCL"; "D=M");
                push_d_onto_stack(&mut ans);

                c!(&mut ans, "// push ARG");
                c!(&mut ans, "@ARG"; "D=M");
                push_d_onto_stack(&mut ans);

                c!(&mut ans, "// push THIS");
                c!(&mut ans, "@THIS"; "D=M");
                push_d_onto_stack(&mut ans);

                c!(&mut ans, "// push THAT");
                c!(&mut ans, "@THAT"; "D=M");
                push_d_onto_stack(&mut ans);

                c!(&mut ans, "// ARG = SP-5-nArgs");
                c!(&mut ans, "@SP"; "D=M");
                c!(&mut ans, "@5"; "D=D-A");
                c!(&mut ans, "@{}", n_args; "D=D-A");
                c!(&mut ans, "@ARG"; "M=D");

                c!(&mut ans, "// LCL = SP");
                c!(&mut ans, "@SP"; "D=M");
                c!(&mut ans, "@LCL"; "M=D");

                c!(&mut ans, "// goto functionName");
                c!(&mut ans, "@{}", name; "0;JMP");

                c!(&mut ans, "// (returnaddress)");
                c!(
                    &mut ans,
                    "({}.{}.return.{})",
                    filename.as_ref(),
                    name,
                    label_cnt
                );

                label_cnt += 1;

                ans
            }
            Node::Add => {
                pop_stack_into_d(&mut ans);
                sp_dec(&mut ans);
                c!(&mut ans, "@SP"; "A=M"; "D=D+M");
                push_d_onto_stack(&mut ans);

                ans
            }
            Node::Sub => {
                pop_stack_into_d(&mut ans);
                sp_dec(&mut ans);
                c!(&mut ans, "@SP"; "A=M"; "D=M-D");
                push_d_onto_stack(&mut ans);

                ans
            }
            Node::Or => {
                pop_stack_into_d(&mut ans);
                sp_dec(&mut ans);
                c!(&mut ans, "@SP"; "A=M"; "D=D|M");
                push_d_onto_stack(&mut ans);

                ans
            }
            Node::And => {
                pop_stack_into_d(&mut ans);
                sp_dec(&mut ans);
                c!(&mut ans, "@SP"; "A=M"; "D=D&M");
                push_d_onto_stack(&mut ans);

                ans
            }
            Node::Neg => {
                pop_stack_into_d(&mut ans);
                set_sp(&mut ans, "-D");
                sp_inc(&mut ans);

                ans
            }
            Node::Not => {
                pop_stack_into_d(&mut ans);
                set_sp(&mut ans, "!D");
                sp_inc(&mut ans);

                ans
            }
            Node::Eq => {
                build_comparison(&mut ans, "JEQ", filename.as_ref(), &mut label_cnt);

                ans
            }
            Node::Gt => {
                build_comparison(&mut ans, "JGT", filename.as_ref(), &mut label_cnt);

                ans
            }
            Node::Lt => {
                build_comparison(&mut ans, "JLT", filename.as_ref(), &mut label_cnt);

                ans
            }
        })
    }
}

fn sp_inc(v: &mut Vec<String>) {
    c!(v, "@SP"; "M=M+1");
}

fn sp_dec(v: &mut Vec<String>) {
    c!(v, "@SP"; "M=M-1");
}

fn set_sp(v: &mut Vec<String>, comp: &str) {
    c!(v, "@SP"; "A=M"; "M={}", comp);
}

fn load_sp_into_d(v: &mut Vec<String>) {
    c!(v, "@SP"; "A=M"; "D=M");
}

fn load_mem_with_offset_into_d(v: &mut Vec<String>, mem: &str, offset: u16) {
    c!(v, "@{}", mem; "D=M");
    c!(v, "@{}", offset; "A=D+A"; "D=M");
}

fn pop_stack_into_d(v: &mut Vec<String>) {
    sp_dec(v);
    load_sp_into_d(v);
}

fn push_d_onto_stack(v: &mut Vec<String>) {
    set_sp(v, "D");
    sp_inc(v);
}

fn load_sp_into_mem_with_offset(v: &mut Vec<String>, mem: &str, offset: u16) {
    c!(v, "@{}", mem; "D=M");
    c!(v, "@{}", offset; "D=D+A");
    c!(v, "@{}", "tmp"; "M=D");
    sp_dec(v);
    load_sp_into_d(v);
    c!(v, "@{}", "tmp"; "A=M"; "M=D");
}

fn build_comparison(v: &mut Vec<String>, jmp: &str, filename: &str, label_cnt: &mut u16) {
    pop_stack_into_d(v);
    sp_dec(v);
    c!(v, "@SP"; "A=M"; "D=M-D");

    // Cond
    c!(v, "@{}.label_yes.{}", filename, label_cnt; "D;{}", jmp);
    // NO
    {
        set_sp(v, "0");
        sp_inc(v);
        c!(v, "@{}.label_no.{}", filename, label_cnt; "0;JMP");
    }
    // YES
    {
        c!(v, "({}.label_yes.{})", filename, label_cnt);
        set_sp(v, "-1");
        sp_inc(v);
    }

    c!(v, "({}.label_no.{})", filename, label_cnt);

    *label_cnt += 1;
}
