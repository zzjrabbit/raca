use core::time::Duration;

use x86_64::instructions::port::Port;

use super::hpet::HPET;

/// Channel 2 data port (read/write)
const CHANNEL_TWO_DATA_PORT: u16 = 0x42;

/// Mode/Command register (write only, a read is ignored)
const COMMAND_REGISTER: u16 = 0x43;

/// PC Speaker positions
const SPEAKER_POSITIONS: u16 = 0x61;

/// Struct for storage ports
pub struct Speaker {
    channel_2_data_port: Port<u8>,
    command_register: Port<u8>,
    speaker_positions: Port<u8>,
}

impl Speaker {
    /// Creates a new `Speaker`.
    pub const fn new() -> Speaker {
        Speaker {
            channel_2_data_port: Port::new(CHANNEL_TWO_DATA_PORT),
            command_register: Port::new(COMMAND_REGISTER),
            speaker_positions: Port::new(SPEAKER_POSITIONS),
        }
    }

    /// Play sound using built in speaker
    fn play_sound(&mut self, n_frequency: u32) {
        let div = 1193180 / n_frequency;

        unsafe {
            self.command_register.write(0xb6);
            self.channel_2_data_port.write(div as u8);
            self.channel_2_data_port.write((div >> 8) as u8);
        }

        let tmp = unsafe { self.speaker_positions.read() };

        if tmp != (tmp | 3) {
            unsafe { self.speaker_positions.write(tmp | 3) };
        }
    }

    /// Make it shutup
    fn nosound(&mut self) {
        let tmp = unsafe { self.speaker_positions.read() };

        unsafe { self.speaker_positions.write(tmp & 0xFC) };
    }

    fn timer_wait(&mut self, n: u128) {
        //for _i in 0..10_000 * n {}
        let ns = HPET.elapsed().as_nanos();
        while HPET.elapsed().as_nanos() - ns < n {}
    }

    /// Make a beep
    pub fn beep(&mut self, frequency: u32, duration: Duration) {
        self.play_sound(frequency);
        self.timer_wait(duration.as_nanos());
        self.nosound();
    }
}
