use super::Log;

const SCREEN_WIDTH: u8 = 160;
const SCREEN_HEIGHT: u8 = 144;

#[derive(Copy, Clone, PartialEq)]
enum BGPriority {
    Color0,
    Color123,
}

pub struct PPU {
    log_mode: u8,
    counter: u16, // cpu 4194304 Hz
    frame_buffer: [u8; (SCREEN_WIDTH as u16 * SCREEN_HEIGHT as u16) as usize],
    bg_priority: [BGPriority; SCREEN_WIDTH as usize], // background priority
    /* Memory */
    vram: [u8; 0x2000],
    oam: [u8; 0xa0],
    /* Interrupts */
    pub irq_vblank: bool,
    pub irq_lcdc: bool,
    /* Regsters */
    lcdc: u8, // lcd control
    stat: u8, // lcd status
    // lcd position and scrolling
    scly: u8, // scroll y
    sclx: u8, // scroll x
    ly: u8,   // current horizontal line
    lyc: u8,  //
    wy: u8,   // window y
    wx: u8,   // window x
    // lcd monochrome palletes
    bgp: u8,  // back ground palette
    obp0: u8, // object palette data 0
    obp1: u8, // object palette data 1
}

impl PPU {
    pub fn new(log_mode: u8) -> Self {
        PPU {
            log_mode: log_mode,
            counter: 0,
            frame_buffer: [0; (SCREEN_WIDTH as u16 * SCREEN_HEIGHT as u16) as usize],
            bg_priority: [BGPriority::Color0; SCREEN_WIDTH as usize],
            vram: [0; 0x2000],
            oam: [0; 0xa0],
            irq_vblank: false,
            irq_lcdc: false,
            lcdc: 0x80,
            stat: 0x02,
            scly: 0,
            sclx: 0,
            ly: 0,
            lyc: 0,
            bgp: 0,
            obp0: 0,
            obp1: 0,
            wx: 0,
            wy: 0,
        }
    }

    fn fetch_tile(&self, tile_no: u8, offset_y: u8, tile_data_sel: bool) -> (u8, u8) {
        let tile_data_addr = if tile_data_sel {
            (tile_no as u16) << 4
        } else {
            (0x1000 as u16).wrapping_add(((tile_no as i8 as i16) << 4) as u16)
        };
        let row_addr = tile_data_addr + (offset_y << 1) as u16;

        let tile0 = self.vram[row_addr as usize];
        let tile1 = self.vram[(row_addr + 1) as usize];

        (tile0, tile1)
    }

    fn fetch_tile_via_xy(
        &self,
        tile_x: u8,
        tile_y: u8,
        offset_y: u8,
        tile_map_base: u16,
    ) -> (u8, u8) {
        let tile_map_addr = tile_map_base | ((tile_x & 0x1f) as u16 + ((tile_y as u16) << 5));
        let tile_no = self.vram[tile_map_addr as usize];

        self.fetch_tile(tile_no, offset_y, self.lcdc & 0x10 > 0)
    }

    fn get_color(&self, color_no: u8, palette: u8) -> u8 {
        match (palette >> (color_no << 1)) & 0x03 {
            0 => 0xff,
            1 => 0xaa,
            2 => 0x55,
            3 | _ => 0x00,
        }
    }

    fn get_color_no(&self, tile: (u8, u8), bitops: u8) -> u8 {
        let low = tile.0 >> bitops & 0x01;
        let high = tile.1 >> bitops & 0x01;

        high << 1 | low
    }

    fn get_bg_tile_map_base(&self) -> u16 {
        // 0x08:bg tile map display select
        match self.lcdc & 0x08 {
            0x08 => 0x1c00,
            _ => 0x1800,
        }
    }

    fn get_window_tile_map_base(&self) -> u16 {
        // 0x40:window tile map display select
        match self.lcdc & 0x40 {
            0x40 => 0x1c00,
            _ => 0x1800,
        }
    }

    fn render_bg(&mut self, buffer: &mut [u8; SCREEN_WIDTH as usize]) {
        let mut tile_x = self.sclx >> 3;
        let mut tile_y = self.scly.wrapping_add(self.ly) >> 3;

        let mut offset_x = self.sclx & 0x07;
        let mut offset_y = self.scly.wrapping_add(self.ly) & 0x07;

        let mut tile: (u8, u8) =
            self.fetch_tile_via_xy(tile_x, tile_y, offset_y, self.get_bg_tile_map_base());
        let mut window = false;

        for x in 0..SCREEN_WIDTH {
            if self.lcdc & 0x20 == 0x20 {
                // 0x20:window display enable
                if self.wy <= self.ly && self.wx == x + 7 {
                    tile_x = 0;
                    tile_y = (self.ly - self.wy) >> 3;
                    offset_x = 0;
                    offset_y = (self.ly - self.wy) & 0x07;
                    tile = self.fetch_tile_via_xy(
                        tile_x,
                        tile_y,
                        offset_y,
                        self.get_window_tile_map_base(),
                    );
                    window = true;
                }
            }

            let color_no = self.get_color_no(tile, 7 - offset_x);
            let color = self.get_color(color_no, self.bgp);

            self.bg_priority[x as usize] = match color_no {
                0x00 => BGPriority::Color0,
                _ => BGPriority::Color123,
            };

            buffer[x as usize] = color;

            offset_x += 1;

            if offset_x >= 8 {
                offset_x = 0;
                tile_x += 1;

                if window {
                    tile = self.fetch_tile_via_xy(
                        tile_x,
                        tile_y,
                        offset_y,
                        self.get_window_tile_map_base(),
                    );
                } else {
                    tile = self.fetch_tile_via_xy(
                        tile_x,
                        tile_y,
                        offset_y,
                        self.get_bg_tile_map_base(),
                    );
                }
            }
        }
    }

    fn render_sprites(&mut self, buffer: &mut [u8; SCREEN_WIDTH as usize]) {
        let mut n_sprites = 0;
        // 0x04:obj(sprite) size
        let height = match self.lcdc & 0x04 {
            0x04 => 16,
            _ => 8,
        };

        const MAX_SPRITES: usize = 40;
        for i in 0..MAX_SPRITES {
            let entry_addr = i << 2;
            let sprite_y = self.oam[entry_addr];
            let sprite_x = self.oam[entry_addr + 1];
            let flags = self.oam[entry_addr + 3];

            let obj_prio = flags & 0x80 == 0x80; // 0x80:obj-to-bg priority
            let flip_y = flags & 0x40 == 0x40; // y-flip
            let flip_x = flags & 0x20 == 0x20; // x-flip

            // 0x10:pallete number
            let palette = match flags & 0x10 {
                0x10 => self.obp1,
                _ => self.obp0,
            };

            if sprite_y <= self.ly + 16 - height || sprite_y > self.ly + 16 {
                // out of range
                continue;
            }

            n_sprites += 1;
            if n_sprites > 10 {
                // max 10 sprites
                break;
            }

            if sprite_x == 0 || sprite_x > (SCREEN_WIDTH as u8) + 8 - 1 {
                // out of screen
                continue;
            }

            // 0x04 sprite size
            let tile_no = match self.lcdc & 0x04 {
                0x04 => {
                    // 8x16
                    if (self.ly + 8 < sprite_y) ^ flip_y {
                        self.oam[entry_addr + 2] & 0xfe
                    } else {
                        self.oam[entry_addr + 2] | 0x01
                    }
                }
                _ => self.oam[entry_addr + 2], // 8x8
            };

            // flip
            let offset_y = match flip_y {
                true => 7 - ((self.ly + 16 - sprite_y) & 0x07),
                _ => (self.ly + 16 - sprite_y) & 0x07,
            };

            //fetch
            let tile = self.fetch_tile(tile_no, offset_y, true);

            for offset_x in 0..8 {
                if offset_x + sprite_x < 8 {
                    // out of screen
                    continue;
                }

                let x = offset_x + sprite_x - 8;

                if x >= SCREEN_WIDTH as u8 {
                    // out of screen
                    break;
                }

                let bitpos = match flip_x {
                    true => offset_x,
                    _ => 7 - offset_x,
                };
                let color_no = self.get_color_no(tile, bitpos);
                if color_no == 0 {
                    // 0:trasparent
                    continue;
                }
                if self.bg_priority[x as usize] == BGPriority::Color123 && obj_prio {
                    // behind bg
                    continue;
                }
                let color = self.get_color(color_no, palette);

                buffer[x as usize] = color;
            }
        }
    }

    fn render_line(&mut self) {
        let mut line_buffer: [u8; SCREEN_WIDTH as usize] = [0; SCREEN_WIDTH as usize];

        if self.lcdc & 0x01 == 0x01 {
            // 0x01:bg display
            self.render_bg(&mut line_buffer);
        }
        if self.lcdc & 0x02 > 0 {
            self.render_sprites(&mut line_buffer);
        }

        for x in 0..SCREEN_WIDTH {
            let ix = (x as usize) + (self.ly as usize) * (SCREEN_WIDTH as usize);
            self.frame_buffer[ix] = line_buffer[x as usize];
        }
    }

    pub fn get_frame_buffer(&self) -> &[u8] {
        &self.frame_buffer
    }

    pub fn update_lyc_interrupt(&mut self) {
        // 0x04:coincidence flag

        if self.ly == self.lyc {
            self.stat |= 0x04;

            // 0x40:lyc=ly coincidence interrupt
            if self.stat & 0x40 == 0x40 {
                self.irq_lcdc = true;
            }
        } else {
            self.stat &= !0x04;
        }
    }

    fn is_vram_accessible(&self) -> bool {
        if self.stat & 0x03 == 0x03 {
            // 0x03:during transfering data to lcd driver
            false
        } else {
            true
        }
    }
    fn is_oam_accessible(&self) -> bool {
        if self.stat & 0x03 == 0 || self.stat & 0x03 == 1 {
            // 0:during h-blank
            // 1:during v-blank
            true
        } else {
            false
        }
    }
    fn is_lcd_enable_change(&self, value: u8) -> bool {
        if self.lcdc & 0x80 != value & 0x80 {
            // 0x80:lcd display enable
            true
        } else {
            false
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        Log::ppu(
            format!("{: <15}:{:#04x}", "write address", address),
            self.log_mode,
        );
        Log::ppu(format!("{: <15}:{:#04x}", "value", value), self.log_mode);

        match address {
            0x8000..=0x9fff => {
                if self.is_vram_accessible() {
                    self.vram[(address & 0x1fff) as usize] = value;
                }
            }
            0xfe00..=0xfe9f => {
                if self.is_oam_accessible() {
                    self.oam[(address & 0x00ff) as usize] = value;
                }
            }
            0xff40 => {
                if self.is_lcd_enable_change(value) {
                    self.ly = 0;
                    self.counter = 0;

                    // 0x80:lcd enable
                    let mode: u8 = match value & 0x80 {
                        0x80 => 2, // 2:during searching oam-ram
                        _ => 0,    // 0:during h-blank
                    };
                    self.stat = (self.stat & 0xf8) | mode;
                    self.update_mode_interrupt();
                }
                self.lcdc = value;
            }
            0xff41 => self.stat = (value & 0xf8) | (self.stat & 0x03), // 0x04:coincidence flag
            0xff42 => self.scly = value,
            0xff43 => self.sclx = value,
            0xff44 => (), // ly read only
            0xff45 => {
                if self.lyc != value {
                    self.update_lyc_interrupt();
                }
                self.lyc = value;
            }
            // 0xff46:dma trasfer and start address
            0xff47 => self.bgp = value,
            0xff48 => self.obp0 = value,
            0xff49 => self.obp1 = value,
            0xff4a => self.wy = value,
            0xff4b => self.wx = value,
            _ => {
                panic!("unexpected address {:#08x}", address)
            }
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        Log::ppu(
            format!("{: <15}:{:#04x}", "read address", address),
            self.log_mode,
        );

        let result: u8 = match address {
            0x8000..=0x9fff => {
                // vram
                if self.is_vram_accessible() {
                    self.vram[(address & 0x1fff) as usize]
                } else {
                    0xff
                }
            }
            0xfe00..=0xfe9f => {
                // oam
                if self.is_oam_accessible() {
                    self.oam[(address & 0x00ff) as usize]
                } else {
                    0xff
                }
            }
            0xff40 => self.lcdc,
            0xff41 => self.stat,
            0xff42 => self.scly,
            0xff43 => self.sclx,
            0xff44 => self.ly,
            0xff45 => self.lyc,
            0xff46 => 0, // write only
            0xff47 => self.bgp,
            0xff48 => self.obp0,
            0xff49 => self.obp1,
            0xff4a => self.wy,
            0xff4b => self.wx,
            _ => {
                panic!("unexpected address {:#08x}", address)
            }
        };

        Log::ppu(format!("{: <15}:{:#04x}", "result", result), self.log_mode);
        result
    }

    fn update_mode_interrupt(&mut self) {
        // 0x03:mode flag
        match self.stat & 0x03 {
            0 if self.stat & 0x08 > 0 => self.irq_lcdc = true, // 0x08:mode h-blank interrupt
            1 if self.stat & 0x10 > 0 => self.irq_lcdc = true, // 0x10:mode v-blank interrupt
            2 if self.stat & 0x20 > 0 => self.irq_lcdc = true, // 0x20:mode oam interrupt
            _ => (),                                           // unnecessary
        }
    }
    fn get_masked_status(&self) -> u8 {
        const MASK: u8 = 0xf8; // clear coincidence(lyc=ly) flag and mode flag
        self.stat & MASK
    }
    pub fn update(&mut self, cycles: u8) {
        if self.lcdc & 0x80 == 0 {
            // lcd display enable off
            return;
        }

        self.counter += cycles as u16;

        // 0x03:mode flag
        match self.stat & 0x03 {
            0x02 => {
                // during searching oam-ram
                if self.counter >= 77 {
                    self.counter -= 77;

                    self.stat = self.get_masked_status() | 0x03; // 0x03:during trasfer
                    self.render_line();
                }
            }
            0x03 => {
                // during trasfering data to lcd driver
                if self.counter >= 169 {
                    self.counter -= 169;

                    self.stat = self.get_masked_status() | 0x00; // 0x00:during h-blank
                    self.update_mode_interrupt();
                }
            }
            0x00 => {
                // during h-blank
                if self.counter >= 201 {
                    self.counter -= 201;
                    self.ly += 1;

                    if self.ly >= SCREEN_HEIGHT {
                        self.stat = self.get_masked_status() | 0x01; // 0x01:during v-blank
                        self.irq_vblank = true;
                    } else {
                        self.stat = self.get_masked_status() | 0x02; // 0x02:during searching oam-ram
                    }

                    self.update_lyc_interrupt();
                    self.update_mode_interrupt();
                }
            }
            0x01 => {
                // during v-blank
                if self.counter >= 456 {
                    self.counter -= 456;
                    self.ly += 1;

                    if self.ly >= 154 {
                        self.stat = self.get_masked_status() | 0x02; // 0x02:during searching oam-ram
                        self.ly = 0;

                        self.update_mode_interrupt();
                    }

                    self.update_lyc_interrupt();
                }
            }
            _ => (), // unnecessary
        }
    }
}
