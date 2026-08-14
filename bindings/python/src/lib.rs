use mneme_core::{
    MemoryStore, SqliteBackend, HashEmbedder, Embedder,
    MemoryType, AccessScope, RecallOptions,
};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::sync::Arc;
use tokio::runtime::Runtime;

#[pyclass]
struct Store {
    inner: Arc<MemoryStore>,
    rt: Arc<Runtime>,
}

#[pymethods]
impl Store {
    #[new]
    fn new(agent_id: String, backend: String) -> PyResult<Self> {
        let rt = Arc::new(Runtime::new().unwrap());
        let rt_clone = rt.clone();
        let backend = rt_clone.block_on(async {
            SqliteBackend::new(&backend).await.unwrap()
        });
        let embedder = Arc::new(HashEmbedder::new(384));
        let store = rt_clone.block_on(async {
            MemoryStore::new(&agent_id, Arc::new(backend), embedder).await
        });
        Ok(Store { inner: Arc::new(store), rt })
    }

    #[pyo3(signature = (content, memory_type, embedding=None))]
    fn remember(&self, content: &str, memory_type: &str, embedding: Option<Vec<f32>>, py: Python) -> PyResult<PyObject> {
        let mt = match memory_type {
            "episodic" => MemoryType::Episodic,
            "semantic" => MemoryType::Semantic,
            "procedural" => MemoryType::Procedural,
            _ => return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("memory_type must be 'episodic', 'semantic', or 'procedural'")),
        };
        let scope = AccessScope::default();
        let record = self.rt.block_on(async {
            self.inner.remember(
                content,
                mt,
                "",
                "",
                "python",
                1.0,
                None,
                vec![],
                scope,
                1.0,
                embedding,
            ).await.unwrap()
        });
        let dict = PyDict::new(py);
        dict.set_item("id", record.id)?;
        dict.set_item("content", record.content)?;
        dict.set_item("memory_type", record.memory_type.to_string())?;
        Ok(dict.into())
    }

    #[pyo3(signature = (query, limit=None, token_budget=None, query_embedding=None))]
    fn recall(&self, query: &str, limit: Option<usize>, token_budget: Option<usize>, query_embedding: Option<Vec<f32>>, py: Python) -> PyResult<Vec<PyObject>> {
        let options = RecallOptions {
            limit: limit.unwrap_or(5),
            token_budget: token_budget.unwrap_or(500),
            ..Default::default()
        };
        let results = self.rt.block_on(async {
            self.inner.recall(query, options, query_embedding).await.unwrap()
        });
        let mut py_results = Vec::new();
        for rm in results {
            let dict = PyDict::new(py);
            dict.set_item("content", rm.record.content)?;
            dict.set_item("score", rm.score)?;
            dict.set_item("explanation", rm.explanation)?;
            dict.set_item("memory_id", rm.record.id)?;
            py_results.push(dict.into());
        }
        Ok(py_results)
    }

    fn forget(&self, memory_id: &str) -> PyResult<()> {
        self.rt.block_on(async { self.inner.forget(memory_id).await.unwrap() });
        Ok(())
    }

    #[getter]
    fn advanced(&self) -> Advanced {
        Advanced { inner: self.inner.clone(), rt: self.rt.clone() }
    }
}

#[pyclass]
struct Advanced {
    inner: Arc<MemoryStore>,
    rt: Arc<Runtime>,
}

#[pymethods]
impl Advanced {
    fn forget_all(&self, user_id: &str) -> PyResult<usize> {
        let count = self.rt.block_on(async { self.inner.forget_all(user_id).await.unwrap() });
        Ok(count)
    }

    fn export(&self, path: &str) -> PyResult<()> {
        self.rt.block_on(async { self.inner.export(path).await.unwrap() });
        Ok(())
    }

    fn import_from(&self, path: &str) -> PyResult<usize> {
        let count = self.rt.block_on(async { self.inner.import_from(path).await.unwrap() });
        Ok(count)
    }

    #[pyo3(signature = (since=None))]
    fn audit_log(&self, since: Option<String>, py: Python) -> PyResult<Vec<PyObject>> {
        let mut log = self.rt.block_on(async { self.inner.get_audit_log().await.unwrap() });
        if let Some(since_str) = since {
            let since_dt = chrono::DateTime::parse_from_rfc3339(&since_str).unwrap().with_timezone(&chrono::Utc);
            log.retain(|e| e.timestamp >= since_dt);
        }
        let mut py_log = Vec::new();
        for event in log {
            let dict = PyDict::new(py);
            dict.set_item("timestamp", event.timestamp.to_rfc3339())?;
            dict.set_item("action", event.action.to_string())?;
            dict.set_item("agent_id", event.agent_id)?;
            dict.set_item("user_id", event.user_id)?;
            dict.set_item("memory_id", event.memory_id)?;
            py_log.push(dict.into());
        }
        Ok(py_log)
    }

    fn deduplicate(&self, threshold: Option<f32>) -> PyResult<usize> {
        let threshold = threshold.unwrap_or(0.9);
        let count = self.rt.block_on(async { self.inner.deduplicate(threshold).await.unwrap() });
        Ok(count)
    }

    #[pyo3(signature = (granted_agent, tags, permission))]
    fn grant_access(&self, granted_agent: &str, tags: Vec<String>, permission: &str) -> PyResult<String> {
        let grant = self.rt.block_on(async {
            self.inner.grant_access(granted_agent, tags, permission).await.unwrap()
        });
        Ok(grant.id)
    }

    fn revoke_access(&self, grant_id: &str) -> PyResult<()> {
        self.rt.block_on(async { self.inner.revoke_access(grant_id).await.unwrap() });
        Ok(())
    }

    #[pyo3(signature = (user_id))]
    fn consolidate(&self, user_id: &str) -> PyResult<usize> {
        let count = self.rt.block_on(async { self.inner.consolidate(user_id).await.unwrap() });
        Ok(count)
    }
}

#[pymodule]
fn mneme(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Store>()?;
    m.add_class::<Advanced>()?;
    Ok(())
}