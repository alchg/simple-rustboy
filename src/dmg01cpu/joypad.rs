use sdl2::keyboard::Keycode;

pub struct Joypad {
    p1joyp: u8, // ff00 p1/joyp
    state: u8,
    pub irq: bool,
}

impl Joypad {
    pub fn new() -> Self {
        Joypad {
            p1joyp: 0xff,
            state: 0xff,
            irq: false,
        }
    }

    pub fn keydown(&mut self, button: Keycode) {
        match button {
            Keycode::S => self.state &= !0x80,
            Keycode::W => self.state &= !0x40,
            Keycode::A => self.state &= !0x20,
            Keycode::D => self.state &= !0x10,
            Keycode::Return => self.state &= !0x08, // start
            Keycode::Space => self.state &= !0x04,  // select
            Keycode::K => self.state &= !0x02,      // b
            Keycode::L => self.state &= !0x01,      // a
            _ => return,
        }

        self.irq = true;
    }

    pub fn keyup(&mut self, button: Keycode) {
        match button {
            Keycode::S => self.state |= 0x80,
            Keycode::W => self.state |= 0x40,
            Keycode::A => self.state |= 0x20,
            Keycode::D => self.state |= 0x10,
            Keycode::Return => self.state |= 0x08, // start
            Keycode::Space => self.state |= 0x04,  // select
            Keycode::K => self.state |= 0x02,      // b
            Keycode::L => self.state |= 0x01,      // a
            _ => return,
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0xff00 => self.p1joyp = (self.p1joyp & 0xcf) | (value & 0x30), // 0x30:select button type
            _ => panic!("unexepted address {:#08x}", address),
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0xff00 => {
                if self.p1joyp & 0x10 == 0 {
                    // 0x10:direction
                    (self.p1joyp & 0xf0) | (self.state >> 4) & 0x0f
                } else if self.p1joyp & 0x20 == 0 {
                    // 0x20:button
                    (self.p1joyp & 0xf0) | self.state & 0x0f
                } else {
                    self.p1joyp
                }
            }
            _ => panic!("unexepted address {:#08x}", address),
        }
    }
}
