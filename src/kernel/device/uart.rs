use crate::choreography::protocol::{UartWrite, UartWriteDone};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UartError {
    Capacity,
}

pub struct UartTxLog<const N: usize> {
    bytes: [u8; N],
    len: usize,
}

impl<const N: usize> UartTxLog<N> {
    pub const fn new() -> Self {
        Self {
            bytes: [0; N],
            len: 0,
        }
    }

    pub fn write(&mut self, write: UartWrite) -> Result<UartWriteDone, UartError> {
        let bytes = write.as_bytes();
        if self.len + bytes.len() > N {
            return Err(UartError::Capacity);
        }
        self.bytes[self.len..self.len + bytes.len()].copy_from_slice(bytes);
        self.len += bytes.len();
        Ok(UartWriteDone::new(bytes.len() as u8))
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
}

impl<const N: usize> Default for UartTxLog<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{UartError, UartTxLog};
    use crate::choreography::protocol::UartWrite;

    #[test]
    fn uart_tx_log_records_bounded_device_writes() {
        let mut log: UartTxLog<16> = UartTxLog::new();
        let done = log
            .write(UartWrite::new(b"uart").expect("write"))
            .expect("room");
        assert_eq!(done.written(), 4);
        assert_eq!(log.as_bytes(), b"uart");
    }

    #[test]
    fn uart_tx_log_rejects_overflow() {
        let mut log: UartTxLog<3> = UartTxLog::new();
        assert_eq!(
            log.write(UartWrite::new(b"uart").expect("write")),
            Err(UartError::Capacity)
        );
    }
}
