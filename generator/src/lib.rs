//! `ono-course`: the deterministic generator, validator and packager of the Ono-Sendai Rust
//! Reading Course. See `docs/architecture.md`.

pub mod diag;
pub mod highlight;
pub mod html;
pub mod load;
pub mod markdown;
pub mod model;
pub mod package;
pub mod paths;
pub mod render;
pub mod sitecheck;
pub mod snippet;
pub mod upstream;
pub mod validate;

use std::path::Path;

/// Load and validate the course at `root`.
pub fn load_and_validate(root: &Path) -> (Option<load::Course>, diag::Diagnostics) {
    let mut diags = diag::Diagnostics::default();
    let course = load::load(root, &mut diags);
    if let Some(c) = &course {
        validate::validate(c, &mut diags);
    }
    (course, diags)
}
