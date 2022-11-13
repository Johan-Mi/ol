mod expression;
mod method;
mod object;
mod parse;
mod program;
mod resolve;
mod typ;
mod value;
mod vm;

use expression::Expression;
use method::Method;
use std::rc::Rc;
use typ::Type;
use value::Value;
use vm::VM;

fn main() {
    let mut vm = VM::new();

    if let Some(file) = std::env::args().nth(1) {
        let file = std::fs::read_to_string(file).unwrap();
        let program = parse::program(&file);
        eprintln!("{program:#?}");
        if let Ok((_, program)) = program {
            let class_ids = vm.load_program(program);
            vm.run(*class_ids.get("MyMainType").unwrap());
        }
    } else {
        // class MyMainType {
        //   def main =
        //     let message = concat "Hello, " "world!"
        //      in println message;
        // }
        let main_type = vm.new_class_id();
        vm.add_method(
            Type::Object(main_type),
            "main".to_owned(),
            Rc::new(Method::Custom {
                body: Expression::LetIn {
                    name: (),
                    bound: Box::new(Expression::MethodCall {
                        name: "concat".to_owned(),
                        this: Box::new(Expression::Literal(Value::String(
                            "Hello, ".to_owned(),
                        ))),
                        arguments: vec![Expression::Literal(Value::String(
                            "world!".to_owned(),
                        ))],
                    }),
                    body: Box::new(Expression::MethodCall {
                        name: "println".to_owned(),
                        this: Box::new(Expression::LocalVariable {
                            name_or_de_brujin_index: 0,
                        }),
                        arguments: vec![],
                    }),
                },
            }),
        );

        vm.run(main_type);
    }
}
