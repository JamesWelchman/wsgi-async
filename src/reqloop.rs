use std::{
	fmt,
	result,
	collections::HashMap,
	sync::mpsc::{Sender, Receiver},
};

#[derive(Debug)]
pub enum Error {}

impl fmt::Display for Error {
    fn fmt(&self, w: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(w, "{:?}", self)
    }
}

type Result<T> = result::Result<T, Error>;

pub fn request_loop(
    inq: &Receiver<(u32, String, HashMap<String, Vec<String>>, Vec<u8>)>,
    outq: &Sender<(u32, Response)>,
) -> Result<()> {

    for (id, url, headers, payload) in inq {
        println!("{:?}", id);
        println!("{:?}", url);
        println!("{:?}", headers);
        println!("{:?}", payload);

        outq.send((id, Response {}));
    }

    Ok(())
}

pub struct Response{}