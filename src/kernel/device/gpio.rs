use crate::choreography::protocol::GpioSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpioError {
    InvalidPin,
}

pub struct GpioStateTable<const N: usize> {
    levels: [bool; N],
}

impl<const N: usize> GpioStateTable<N> {
    pub const fn new() -> Self {
        Self { levels: [false; N] }
    }

    pub fn apply(&mut self, set: GpioSet) -> Result<GpioSet, GpioError> {
        let slot = self
            .levels
            .get_mut(set.pin() as usize)
            .ok_or(GpioError::InvalidPin)?;
        *slot = set.high();
        Ok(set)
    }

    pub fn level(&self, pin: u8) -> Result<bool, GpioError> {
        self.levels
            .get(pin as usize)
            .copied()
            .ok_or(GpioError::InvalidPin)
    }
}

impl<const N: usize> Default for GpioStateTable<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{GpioError, GpioStateTable};
    use crate::choreography::protocol::GpioSet;

    #[test]
    fn gpio_set_updates_bounded_pin_state() {
        let mut pins: GpioStateTable<32> = GpioStateTable::new();
        pins.apply(GpioSet::new(25, true)).expect("set pin high");
        assert_eq!(pins.level(25), Ok(true));
        pins.apply(GpioSet::new(25, false)).expect("set pin low");
        assert_eq!(pins.level(25), Ok(false));
    }

    #[test]
    fn gpio_set_rejects_invalid_pin() {
        let mut pins: GpioStateTable<2> = GpioStateTable::new();
        assert_eq!(
            pins.apply(GpioSet::new(25, true)),
            Err(GpioError::InvalidPin)
        );
    }
}
