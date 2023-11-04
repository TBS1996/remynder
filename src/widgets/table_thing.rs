use crate::create_field;

fn barfoo() {
    let x = Field {
        name: "for int".into(),
        eval: Box::new(|x: &str| x.parse::<i32>().is_ok()),
        value: "heythere".into(),
    };
    let x = create_field!("heythere", i32);
}

struct Field {
    name: String,
    eval: Box<dyn FnMut(&str) -> bool>,
    value: String,
}

struct InputTable {
    fields: Vec<Field>,
    idx: usize,
}

mod Macro {
    use super::*;

    #[macro_export]
    macro_rules! create_field {
        ($name:expr, $type:ty) => {
            Field {
                name: $name.to_string(),
                eval: Box::new(|input: &str| input.parse::<$type>().is_ok()),
                value: String::new(),
            }
        };
    }
}
