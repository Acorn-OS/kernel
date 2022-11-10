use crate::arch::io::{in8_delay, out8_delay};

const PIC1_BASE: u16 = 0x20;
const PIC2_BASE: u16 = 0xA0;

const COMMAND_OFFSET: u16 = 0;
const DATA_OFFSET: u16 = 1;

/// Different commands to send to the PIC.
pub mod cmd {
    /// End of interrupt command.
    pub const EOI: u8 = 0x20;
    /// TODO:
    pub const ICW1_INIT: u8 = 0x10;
    /// TODO:
    pub const ICW4_8086: u8 = 0x01;
}

struct Pic {
    base: u16,
    cascade_ident: u8,
}

impl Pic {
    fn remap(&self, start_vec: u8) {
        let mask = self.data_in();
        self.command(cmd::ICW1_INIT);
        self.data_out(start_vec);
        self.data_out(self.cascade_ident);
        self.data_out(cmd::ICW4_8086);
        self.data_out(mask);
    }

    #[inline]
    fn command(&self, cmd: u8) {
        out8_delay(self.base + COMMAND_OFFSET, cmd);
    }

    #[inline]
    fn data_out(&self, out: u8) {
        out8_delay(self.base + DATA_OFFSET, out);
    }

    #[inline]
    fn data_in(&self) -> u8 {
        in8_delay(self.base + DATA_OFFSET)
    }
}

static PIC1: Pic = Pic {
    base: PIC1_BASE,
    cascade_ident: 4,
};
static PIC2: Pic = Pic {
    base: PIC2_BASE,
    cascade_ident: 2,
};

/// Remaps both the PIC1 and PIC2.
pub fn remap(pic1_vec: u8, pic2_vec: u8) {
    pic1::remap(pic1_vec);
    pic2::remap(pic2_vec);
}

/// Sends a command to both the PIC1 and PIC2.
pub fn cmd(cmd: u8) {
    pic1::cmd(cmd);
    pic2::cmd(cmd);
}

/// Signals the end of an interrupt for both PICs.
/// This is preferable over signaling to indiviual
/// PICs.
pub fn end_of_interrupt() {
    pic1::end_of_interrupt();
    pic2::end_of_interrupt();
}

/// Disables interrupts for both PIC1 and PIC2.
pub fn disable_all() {
    pic1::disable_all();
    pic2::disable_all();
}

/// Enables interrupts for both PIC1 and PIC2.
pub fn enable_all() {
    pic1::enable_all();
    pic2::enable_all();
}

macro_rules! impl_pic {
    ($ident:ident) => {
        use super::*;

        /// Remaps the PIC from the starting vector.
        pub fn remap(start_vec: u8) {
            $ident.remap(start_vec);
        }

        /// Masks the PIC.
        pub fn mask(mask: u8) {
            $ident.data_out(mask);
        }

        /// Sends a command to the PIC.
        pub fn cmd(cmd: u8) {
            $ident.command(cmd);
        }

        /// Signals the end of an interrupt for the PIC.
        pub fn end_of_interrupt() {
            cmd(cmd::EOI)
        }

        /// Enable all interrupts.
        pub fn enable_all() {
            mask(0);
        }

        /// Disable all interrupts.
        pub fn disable_all() {
            mask(0xFF);
        }
    };
}

/// Operate on PIC1.
pub mod pic1 {
    impl_pic!(PIC1);
}

/// Operator on PIC2.
pub mod pic2 {
    impl_pic!(PIC2);
}
