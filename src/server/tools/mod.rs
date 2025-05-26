mod tera;

use ::tera::Tera;

pub(super) struct Tools {
    pub(super) tera: Tera,
}

impl Tools {
    pub(super) fn init() -> Self {
        Self { tera: tera::init() }
    }
}
