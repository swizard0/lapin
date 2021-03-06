use std::io;
use futures::{Async,Poll,Stream,task};
use tokio_io::{AsyncRead,AsyncWrite};
use std::sync::{Arc,Mutex};

use message::Delivery;
use transport::*;

#[derive(Clone)]
pub struct Consumer<T> {
  pub transport:    Arc<Mutex<AMQPTransport<T>>>,
  pub channel_id:   u16,
  pub queue:        String,
  pub consumer_tag: String,
  pub registered:   bool,
}

impl<T: AsyncRead+AsyncWrite+Sync+Send+'static> Stream for Consumer<T> {
  type Item = Delivery;
  type Error = io::Error;

  fn poll(&mut self) -> Poll<Option<Delivery>, io::Error> {
    trace!("poll; consumer_tag={:?}", self.consumer_tag);
    let mut transport = lock_transport!(self.transport);
    if !self.registered {
        transport.register_consumer(&self.consumer_tag, task::current());
        self.registered = true;
    }
    transport.poll()?;
    trace!("poll transport; consumer_tag={:?} status=NotReady", self.consumer_tag);
    if let Some(message) = transport.conn.next_delivery(self.channel_id, &self.queue, &self.consumer_tag) {
      trace!("delivery; consumer_tag={:?} delivery_tag={:?}", self.consumer_tag, message.delivery_tag);
      return Ok(Async::Ready(Some(message)));
    }
    Ok(Async::NotReady)
  }
}
