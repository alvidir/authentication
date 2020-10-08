use crate::transaction::traits::Body;
use crate::transaction::traits::Tx;
use crate::transaction::factory as TxFactory;
use std::error::Error;
use std::any::Any;

pub struct TxLogin {
    cookie: String,
    name: String,
    addr: String,
    pwd: String,
}

impl TxLogin {
    pub fn new(cookie: String, name: String, addr: String, pwd: String) -> Self {
        TxLogin{
            cookie: cookie,
            name: name,
            addr: addr,
            pwd: pwd,
        }
    }
}

impl Body for TxLogin {
    fn precondition(&self) -> Result<(), String> {
        Ok(())
    }

	fn postcondition(&self) -> Result<Box<dyn Any>, String> {
        Ok(Box::new(String::new()))
    }

	fn commit(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

	fn rollback(&self) {

    }

}