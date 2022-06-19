pub struct Timer {
    cycle: u32,    // cpu 4194304 Hz
    pub irq: bool, // timer interrupt
    /* Registers */
    div: u32, // 16384 Hz
    tima: u8, // timer counter
    tma: u8,  // timer modulo
    tac: u8,  // timer control
}
impl Timer {
    pub fn new() -> Self {
        Timer {
            cycle: 0,
            tima: 0,
            tma: 0,
            tac: 0,
            div: 0,
            irq: false,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0xff04 => self.div as u8,
            0xff05 => self.tima,
            0xff06 => self.tma,
            0xff07 => self.tac,
            _ => {
                panic!("unexpected address {:#08x}", address)
            }
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0xff04 => {
                self.cycle %= 16384; // system clock % 16384 Hz
                self.div = 0;
            }
            0xff05 => self.tima = value,
            0xff06 => self.tma = value,
            0xff07 => self.tac = value & 0x07,
            _ => {
                panic!("unexpected address {:#08x}", address)
            }
        }
    }

    pub fn update(&mut self, cycle_elapsed: u8) {
        let cycle_pre: u32 = self.cycle;
        self.cycle = self.cycle.wrapping_add(cycle_elapsed as u32);

        self.div = self.cycle >> 8; // n/16384

        if self.tac & 0x04 == 0x04 {
            // 0x04:timer enable
            let division: u32 = match self.tac & 0x03 {
                // bit 0-1 Input Clock Select
                0x00 => 1024,
                0x01 => 16,
                0x02 => 64,
                0x03 => 256,
                _ => panic!("unexpected"),
            };

            let current = self.cycle / division;
            let previous = cycle_pre / division;
            let diff;
            if current < previous {
                diff = (u32::MAX / division) - previous + 1 + current;
            } else {
                diff = current - previous;
            }

            if diff > 0 {
                let (result, overflow) = self.tima.overflowing_add(diff as u8);

                if overflow {
                    self.tima = self.tma + diff as u8;
                    self.irq = true;
                } else {
                    self.tima = result;
                }
            }
        }
    }
}
