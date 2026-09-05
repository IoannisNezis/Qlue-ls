use indoc::indoc;
use tera::{Kwargs, State, Tera};

pub(super) fn init() -> Tera {
    let mut tera = Tera::default();
    // NOTE: tests/filters/functions must be registered *before* any template is added:
    // tera v2 resolves them at template compile time.
    tera.register_test("variable", is_variable);
    tera.add_raw_template(
        "prefix_declarations",
        indoc! {
            "{%- for prefix in prefixes -%}
             PREFIX {{prefix[0]}}: <{{prefix[1]}}>
             {%- endfor -%}
            "
        },
    )
    .expect("This hardcoded template should be valid");
    tera
}

/// INFO: declaring the argument as `&str` makes tera reject non-string values
/// with a proper error message, no manual type check needed.
fn is_variable(value: &str, _kwargs: Kwargs, _state: &State) -> bool {
    value.starts_with("?") && value.split(" ").count() == 1
}
