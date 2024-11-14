use loro_internal::encoding::ExportMode;
use loro_internal::{LoroDoc, TextHandler, ToJson};
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use std::borrow::Cow;

#[pyclass]
struct Loro(LoroDoc);

#[pyclass]
struct LoroText(TextHandler);

#[pymethods]
impl Loro {
    #[new]
    pub fn __new__() -> Self {
        Self(LoroDoc::default())
    }

    pub fn get_text(&mut self, id: &str) -> LoroText {
        let text = self.0.get_text(id);
        LoroText(text)
    }

    pub fn to_json(&mut self) -> PyResult<String> {
        Ok(self.0.get_deep_value().to_json_value().to_string())
    }

    pub fn import_snapshot(&mut self, snapshot: &[u8]) -> PyResult<()> {
        self.0.import(snapshot).unwrap();
        Ok(())
    }

    pub fn import_update_batch(&mut self, data: Vec<Vec<u8>>) -> PyResult<()> {
        if data.is_empty() {
            return Ok(());
        }
        self.0.import_batch(&data).unwrap();
        Ok(())
    }

    pub fn export_snapshot(&self, py: Python) -> PyResult<PyObject> {
        let snapshot = self
            .0
            .export(ExportMode::ShallowSnapshot(Cow::Owned(
                self.0.oplog_frontiers().into(),
            )))
            .unwrap();
        Ok(PyBytes::new(py, &snapshot).into())
    }
}

#[pymethods]
impl LoroText {
    pub fn insert(&mut self, ctx: &Loro, pos: usize, value: &str) -> PyResult<()> {
        self.0
            .insert_with_txn(&mut ctx.0.txn().unwrap(), pos, value)
            .unwrap();
        Ok(())
    }
}

#[pymodule]
fn pyloro(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Loro>()?;
    m.add_class::<LoroText>()?;
    Ok(())
}
