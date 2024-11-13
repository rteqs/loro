use base64::prelude::*;
use loro_internal::encoding::ExportMode;
use loro_internal::{LoroDoc, TextHandler};
use pyo3::{prelude::*, types::PyList};
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

    pub fn import_snapshot(&mut self, snapshot: &[u8]) -> PyResult<()> {
        self.0.import(snapshot).unwrap();
        Ok(())
    }

    pub fn import_update_batch(&mut self, data: &PyList) -> PyResult<()> {
        let decoded_updates = data
            .iter()
            .filter_map(|x| {
                let update: String = x.extract::<String>().ok()?.to_string();
                BASE64_STANDARD.decode(update).ok()
            })
            .collect::<Vec<_>>();

        if decoded_updates.is_empty() {
            return Ok(());
        }
        self.0.import_batch(&decoded_updates[..]).unwrap();
        Ok(())
    }

    pub fn export_snapshot(&self) -> PyResult<String> {
        let snapshot = self
            .0
            .export(ExportMode::ShallowSnapshot(Cow::Owned(
                self.0.oplog_frontiers().into(),
            )))
            .unwrap();
        Ok(BASE64_STANDARD.encode(snapshot))
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

    // pub fn value(&self) -> String {
    //     Arc::try_unwrap(self.0.get_value().into_string().unwrap()).unwrap_or_else(|x| (*x).clone())
    // }
}

#[pymodule]
fn pyloro(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Loro>()?;
    m.add_class::<LoroText>()?;
    Ok(())
}
