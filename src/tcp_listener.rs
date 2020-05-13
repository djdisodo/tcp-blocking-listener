use std::net::{TcpStream, Shutdown, ToSocketAddrs};
use std::io::{Result, Read, Write};
use std::thread::{Builder, JoinHandle};
use std::mem::replace;
use crate::ReceiveHandler;

pub struct NetworkListener<T: ReceiveHandler + Send> {
	tcp_stream : TcpStream,
	join_handle: Option<JoinHandle<()>>,
	receive_handler: T
}

impl <T: ReceiveHandler + Send>NetworkListener<T> {
	pub fn new<A: ToSocketAddrs>(address: &A, receive_handler: T) -> Result<Self> {
		let tcp_stream_result = TcpStream::connect(address);
		match tcp_stream_result {
			Ok(tcp_stream) => {
				match tcp_stream.set_nonblocking(false) {
					Err(e) => return Err(e),
					_ => {}
				}
				Ok(Self {
					tcp_stream,
					join_handle: None,
					receive_handler
				})
			},
			Err(e) => Err(e)
		}
	}
	pub fn interrupt(&self) -> Result<()> {
		match &self.join_handle {
			Some(_) => self.tcp_stream.shutdown(Shutdown::Both),
			None => Ok(())
		}
	}

	fn run(&mut self) -> JoinHandle<()> {
		unsafe {
			Builder::new()
				.spawn_unchecked( move || {
					loop {
						let mut buffer: [u8; 1500] = [0; 1500];
						match self.tcp_stream.read(&mut buffer) {
							Ok(length) => {
								if length == 0 {
									break
								}
							},
							Err(_) => break
						}
						self.receive_handler.handle(&buffer);
					}
				}).unwrap()
		}
	}

	pub fn start(&mut self) {
		let join_handle = self.run();
		self.join_handle.replace(join_handle);
	}
}

impl<T: ReceiveHandler + Send> Drop for NetworkListener<T> {
	fn drop(&mut self) {
		self.interrupt().unwrap_or_default();
		if self.join_handle.is_some() {
			let join_handle = replace(&mut self.join_handle, None).unwrap();
			join_handle.join().unwrap_or_default();
		}
	}
}

impl<T: ReceiveHandler + Send> Write for NetworkListener<T> {
	fn write(&mut self, buf: &[u8]) -> Result<usize> {
		self.tcp_stream.write(buf)
	}
	fn flush(&mut self) -> Result<()> {
		self.tcp_stream.flush()
	}
}