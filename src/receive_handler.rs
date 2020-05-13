pub trait ReceiveHandler {
	fn handle(&mut self, buffer: &[u8; 1500]);
}