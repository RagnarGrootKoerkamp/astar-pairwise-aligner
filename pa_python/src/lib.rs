use pa_types::Cost;
use pyo3::{Bound, PyResult, prelude::PyModule, types::PyModuleMethods, wrap_pyfunction};

#[pyo3::pyfunction]
pub fn search<'s>(pattern: &'s [u8], text: &'s [u8], unmatched_cost: f32) -> Vec<i32> {
    pa_bitpacking::search(pattern, text, unmatched_cost).out
}

#[pyo3::pymodule]
fn pa_python(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(search, m)?)?;
    Ok(())
}
