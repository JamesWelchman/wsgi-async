use std::{
    collections::HashMap,
    sync::mpsc::{channel, Receiver, Sender, TryRecvError},
    sync::{Arc, Mutex},
    thread::Builder as ThreadBuilder,
};

use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;
use rand::Rng;

mod reqloop;

#[pyclass]
pub struct RequestThread {
    inq: Sender<(u32, String, HashMap<String, Vec<String>>, Vec<u8>)>,
    outq: Receiver<(u32, reqloop::Response)>,
    errq: Receiver<reqloop::Error>,

    // Completed Requests
    completed_requests: Arc<Mutex<HashMap<u32, Response>>>,
}

#[pymethods]
impl RequestThread {
    #[new]
    pub fn new() -> PyResult<Self> {
        let (inq_s, inq_r) = channel();
        let (outq_s, outq_r) = channel();
        let (errq_s, errq_r) = channel();

        ThreadBuilder::new()
            .name("request-thread".to_string())
            .spawn(move || run_request_thread(inq_r, outq_s, errq_s))
            .expect("couldn't start request thread");

        Ok(Self {
            inq: inq_s,
            outq: outq_r,
            errq: errq_r,

            completed_requests: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn send(
        &self,
        url: String,
        headers: HashMap<String, Vec<String>>,
        payload: Vec<u8>,
    ) -> PyResult<Ref> {
        let id: u32 = rand::thread_rng().gen();

        self.inq.send((id, url, headers, payload));

        Ok(Ref { id })
    }

    pub fn try_take(&self, r: &Ref) -> PyResult<Option<Response>> {
        // Drain the outq
        loop {
            match self.outq.try_recv() {
                Ok((id, resp)) => {
                    self.completed_requests
                        .lock()
                        .map_err(|_| PyRuntimeError::new_err("couldn't lock mutex"))?
                        .insert(id, Response::from_resp(resp));
                }
                Err(e) => match e {
                    TryRecvError::Empty => break,
                    TryRecvError::Disconnected => {
                        // We should raise an exception
                    }
                },
            }
        }

        Ok({
            self.completed_requests
                .lock()
                .map_err(|_| PyRuntimeError::new_err("couldn't lock mutex"))?
                .remove(&r.id)
        })
    }

    pub fn wait(&self, r: &Ref) -> PyResult<Option<Response>> {
        if let Some(r) = self.try_take(r)? {
            return Ok(Some(r));
        }

        for (id, response) in self.outq.iter() {
            self.completed_requests
                .lock()
                .map_err(|_| PyRuntimeError::new_err("couldn't lock mutex"))?
                .insert(id, Response::from_resp(response));

            if id == r.id {
                break;
            }
        }

        self.try_take(r)
    }
}

#[pyclass]
pub struct Ref {
    id: u32,
}

#[pyclass]
pub struct Response {}

impl Response {
    fn from_resp(resp: reqloop::Response) -> Self {
        Self{}
    }
}

#[pymodule]
fn wsgi_async_core(_: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<RequestThread>()?;
    Ok(())
}

fn run_request_thread(
    inq: Receiver<(u32, String, HashMap<String, Vec<String>>, Vec<u8>)>,
    outq: Sender<(u32, reqloop::Response)>,
    errq: Sender<reqloop::Error>,
) {
    loop {
        if let Err(e) = reqloop::request_loop(&inq, &outq) {
            // Write to the errq
            errq.send(e).expect("couldn't send error");
        }
    }
}

