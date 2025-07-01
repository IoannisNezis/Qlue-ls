use indoc::indoc;
use tera::{
    helpers::tests::{number_args_allowed, value_defined},
    Result, Tera, Value,
};

pub(super) fn init() -> Tera {
    let mut tera = Tera::default();
    tera.add_raw_template(
        "prefix_declarations",
        indoc! {
            "{%- for prefix in prefixes -%}
             PREFIX {{prefix.0}}: <{{prefix.1}}>
             {%- endfor -%}
            "
        },
    )
    .expect("This hardcoded template should be valid");
    tera.register_tester("variable", is_variable);
    tera
}

fn is_variable(value: Option<&Value>, params: &[Value]) -> Result<bool> {
    number_args_allowed("variable", 0, params.len())?;
    value_defined("variable", value)?;
    match value.and_then(|v| v.as_str()) {
        Some(f) => Ok(f.starts_with("?") && f.split(" ").count() == 1),
        _ => Err(tera::Error::msg(
            "Tester `variable` was called on a variable that isn't a string",
        )),
    }
}
