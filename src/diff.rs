use std::path::Path;

use crate::contract::{DiffFormat, RenderedContractDiff, render_contract_diff};

pub(crate) fn run(before: &Path, after: &Path, format: DiffFormat) -> RenderedContractDiff {
    render_contract_diff(before, after, format)
}
