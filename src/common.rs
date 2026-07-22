use core::fmt;
use core::marker::PhantomData;

use core::fmt::Debug;

use aligned::{A4, Aligned};

use crate::{
    BlockCommand, BlockReadCommand, BlockWriteCommand, Command, CommandIndex, ControlCommand, R0,
    R1, R1b, R2, R3, R4, R6, SdMode, spi,
};

// ============================================================================
// COMMON COMMANDS
// ============================================================================

/// CMD0 — GO_IDLE_STATE
pub struct Cmd0;

impl CommandIndex for Cmd0 {
    const INDEX: u8 = 0;
}
impl Command<SdMode> for Cmd0 {
    type Resp<'a> = R0;
    fn arg(&self) -> u32 {
        0
    }
}
impl Command<spi::SpiMode> for Cmd0 {
    type Resp<'a> = spi::R1;
    fn arg(&self) -> u32 {
        0
    }
}
impl<M> ControlCommand<M> for Cmd0 where Self: Command<M> {}

/// CMD0 — GO_IDLE_STATE
pub fn idle() -> Cmd0 {
    Cmd0
}

pub type Idle = Cmd0;

/// CMD2 — ALL_SEND_CID
pub struct Cmd2;
impl CommandIndex for Cmd2 {
    const INDEX: u8 = 2;
}
impl Command<SdMode> for Cmd2 {
    type Resp<'a> = R2;
    fn arg(&self) -> u32 {
        0
    }
}
impl<M> ControlCommand<M> for Cmd2 where Self: Command<M> {}

/// CMD2: Ask any card to send their CID
pub fn all_send_cid() -> Cmd2 {
    Cmd2
}

pub type AllSendCid = Cmd2;

/// CMD7 — SELECT/DESELECT_CARD
pub struct Cmd7 {
    pub rca: u16,
}
impl CommandIndex for Cmd7 {
    const INDEX: u8 = 7;
}
impl Command<SdMode> for Cmd7 {
    type Resp<'a> = R1;
    fn arg(&self) -> u32 {
        (self.rca as u32) << 16
    }
}
impl<M> ControlCommand<M> for Cmd7 where Self: Command<M> {}

/// CMD7: Select or deselect card
pub fn select_card(rca: u16) -> Cmd7 {
    Cmd7 { rca }
}

/// CMD9 — SEND_CSD
pub struct Cmd9 {
    pub rca: u16,
}
impl CommandIndex for Cmd9 {
    const INDEX: u8 = 9;
}
impl Command<SdMode> for Cmd9 {
    type Resp<'a> = R2;
    fn arg(&self) -> u32 {
        (self.rca as u32) << 16
    }
}
impl ControlCommand<SdMode> for Cmd9 {}

/// CMD9: Send CSD
pub fn send_csd(rca: u16) -> Cmd9 {
    Cmd9 { rca }
}

/// CMD10 — SEND_CID
pub struct Cmd10 {
    pub rca: u16,
}
impl CommandIndex for Cmd10 {
    const INDEX: u8 = 10;
}
impl Command<SdMode> for Cmd10 {
    type Resp<'a> = R2;
    fn arg(&self) -> u32 {
        (self.rca as u32) << 16
    }
}
impl<M> ControlCommand<M> for Cmd10 where Self: Command<M> {}

/// CMD10: Send CID
pub fn send_cid(rca: u16) -> Cmd10 {
    Cmd10 { rca }
}

/// CMD12 — STOP_TRANSMISSION (R1b)
pub struct Cmd12;
impl CommandIndex for Cmd12 {
    const INDEX: u8 = 12;
}
impl Command<SdMode> for Cmd12 {
    type Resp<'a> = R1b;
    fn arg(&self) -> u32 {
        0
    }
}
impl Command<spi::SpiMode> for Cmd12 {
    type Resp<'a> = spi::R1b;
    fn arg(&self) -> u32 {
        0
    }
}
impl<M> ControlCommand<M> for Cmd12 where Self: Command<M> {}

/// CMD12: Stop transmission
pub fn stop_transmission() -> Cmd12 {
    Cmd12
}

pub type StopTransmission = Cmd2;

/// CMD13 — SEND_STATUS
pub struct Cmd13 {
    pub rca: u16,
    pub task_status: bool,
}
impl CommandIndex for Cmd13 {
    const INDEX: u8 = 13;
}
impl Command<SdMode> for Cmd13 {
    type Resp<'a> = R1;
    fn arg(&self) -> u32 {
        (self.rca as u32) << 16 | (self.task_status as u32) << 15
    }
}
impl Command<spi::SpiMode> for Cmd13 {
    type Resp<'a> = spi::R2;
    fn arg(&self) -> u32 {
        0
    }
}
impl<M> ControlCommand<M> for Cmd13 where Self: Command<M> {}

/// CMD13: Ask card to send status or task status
pub fn card_status(rca: u16, task_status: bool) -> Cmd13 {
    Cmd13 { rca, task_status }
}

pub type SendStatus = Cmd13;

// /// CMD15: Sends card to inactive state
// pub fn go_inactive_state(rca: u16) -> Cmd<Rz> {
//     cmd(15, u32::from(rca) << 16)
// }
//

/// CMD16 — SET_BLOCKLEN (rarely used on SDHC/SDXC)
pub struct Cmd16 {
    pub block_len: u32,
}
impl CommandIndex for Cmd16 {
    const INDEX: u8 = 16;
}
impl Command<SdMode> for Cmd16 {
    type Resp<'a> = R1;
    fn arg(&self) -> u32 {
        self.block_len
    }
}
impl Command<spi::SpiMode> for Cmd16 {
    type Resp<'a> = spi::R1;
    fn arg(&self) -> u32 {
        self.block_len
    }
}
impl<M> ControlCommand<M> for Cmd16 where Self: Command<M> {}

/// CMD16: Set block len
pub fn set_block_length(block_len: u32) -> Cmd16 {
    Cmd16 { block_len }
}

/// CMD17 — READ_SINGLE_BLOCK
pub struct Cmd17<'a, const BLOCK_SIZE: usize> {
    pub addr: u32,
    pub buf: &'a mut [Aligned<A4, [u8; BLOCK_SIZE]>],
}
impl<'a, const BLOCK_SIZE: usize> CommandIndex for Cmd17<'a, BLOCK_SIZE> {
    const INDEX: u8 = 17;
}
impl<'a, const BLOCK_SIZE: usize> Command<SdMode> for Cmd17<'a, BLOCK_SIZE> {
    type Resp<'b>
        = R1
    where
        Self: 'b;
    fn arg(&self) -> u32 {
        self.addr
    }
}
impl<'a, const BLOCK_SIZE: usize> Command<spi::SpiMode> for Cmd17<'a, BLOCK_SIZE> {
    type Resp<'b>
        = spi::R1
    where
        Self: 'b;
    fn arg(&self) -> u32 {
        self.addr
    }
}
impl<'a, const BLOCK_SIZE: usize, M> BlockCommand<M> for Cmd17<'a, BLOCK_SIZE>
where
    Self: Command<M>,
{
    type Block = Aligned<A4, [u8; BLOCK_SIZE]>;
    fn block_count(&self) -> u32 {
        1
    }
    fn buf(&mut self) -> &mut [Self::Block] {
        &mut *self.buf
    }
}

impl<'a, const BLOCK_SIZE: usize, M> BlockReadCommand<M> for Cmd17<'a, BLOCK_SIZE> where
    Self: Command<M>
{
}

/// CMD17: Read a single block from the card
pub fn read_single_block<const BLOCK_SIZE: usize>(
    addr: u32,
    buf: &mut Aligned<A4, [u8; BLOCK_SIZE]>,
) -> Cmd17<'_, BLOCK_SIZE> {
    Cmd17 {
        addr,
        buf: core::slice::from_mut(buf),
    }
}

/// CMD18 — READ_MULTIPLE_BLOCK
pub struct Cmd18<'a, const BLOCK_SIZE: usize> {
    pub addr: u32,
    pub bufs: &'a mut [Aligned<A4, [u8; BLOCK_SIZE]>],
}
impl<'a, const BLOCK_SIZE: usize> CommandIndex for Cmd18<'a, BLOCK_SIZE> {
    const INDEX: u8 = 18;
}
impl<'a, const BLOCK_SIZE: usize> Command<SdMode> for Cmd18<'a, BLOCK_SIZE> {
    type Resp<'b>
        = R1
    where
        Self: 'b;
    fn arg(&self) -> u32 {
        self.addr
    }
}
impl<'a, const BLOCK_SIZE: usize> Command<spi::SpiMode> for Cmd18<'a, BLOCK_SIZE> {
    type Resp<'b>
        = spi::R1
    where
        Self: 'b;
    fn arg(&self) -> u32 {
        self.addr
    }
}
impl<'a, const BLOCK_SIZE: usize, M> BlockCommand<M> for Cmd18<'a, BLOCK_SIZE>
where
    Self: Command<M>,
{
    type Block = Aligned<A4, [u8; BLOCK_SIZE]>;
    fn block_count(&self) -> u32 {
        self.bufs.len() as u32
    }
    fn buf(&mut self) -> &mut [Self::Block] {
        self.bufs
    }
}
impl<'a, const BLOCK_SIZE: usize, M> BlockReadCommand<M> for Cmd18<'a, BLOCK_SIZE> where
    Self: Command<M>
{
}

/// CMD18: Read multiple block from the card
pub fn read_multiple_blocks<const BLOCK_SIZE: usize>(
    addr: u32,
    bufs: &mut [Aligned<A4, [u8; BLOCK_SIZE]>],
) -> Cmd18<'_, BLOCK_SIZE> {
    Cmd18 { addr, bufs }
}

/// CMD24 — WRITE_BLOCK
pub struct Cmd24<'a, const BLOCK_SIZE: usize> {
    pub addr: u32,
    pub buf: &'a mut [Aligned<A4, [u8; BLOCK_SIZE]>],
}
impl<'a, const BLOCK_SIZE: usize> CommandIndex for Cmd24<'a, BLOCK_SIZE> {
    const INDEX: u8 = 24;
}
impl<'a, const BLOCK_SIZE: usize> Command<SdMode> for Cmd24<'a, BLOCK_SIZE> {
    type Resp<'b>
        = R1b
    where
        Self: 'b;
    fn arg(&self) -> u32 {
        self.addr
    }
}
impl<'a, const BLOCK_SIZE: usize> Command<spi::SpiMode> for Cmd24<'a, BLOCK_SIZE> {
    type Resp<'b>
        = spi::R1b
    where
        Self: 'b;
    fn arg(&self) -> u32 {
        self.addr
    }
}
impl<'a, const BLOCK_SIZE: usize, M> BlockCommand<M> for Cmd24<'a, BLOCK_SIZE>
where
    Self: Command<M>,
{
    type Block = Aligned<A4, [u8; BLOCK_SIZE]>;
    fn block_count(&self) -> u32 {
        1
    }
    fn buf(&mut self) -> &mut [Self::Block] {
        self.buf
    }
}

impl<'a, const BLOCK_SIZE: usize, M> BlockWriteCommand<M> for Cmd24<'a, BLOCK_SIZE> where
    Self: Command<M>
{
}

/// CMD24: Write block
pub fn write_single_block<const BLOCK_SIZE: usize>(
    addr: u32,
    buf: &mut Aligned<A4, [u8; BLOCK_SIZE]>,
) -> Cmd24<'_, BLOCK_SIZE> {
    Cmd24 {
        addr,
        buf: core::slice::from_mut(buf),
    }
}

/// CMD25 — WRITE_MULTIPLE_BLOCK
pub struct Cmd25<'a, const BLOCK_SIZE: usize> {
    pub addr: u32,
    pub bufs: &'a mut [Aligned<A4, [u8; BLOCK_SIZE]>],
}
impl<'a, const BLOCK_SIZE: usize> CommandIndex for Cmd25<'a, BLOCK_SIZE> {
    const INDEX: u8 = 25;
}
impl<'a, const BLOCK_SIZE: usize> Command<SdMode> for Cmd25<'a, BLOCK_SIZE> {
    type Resp<'b>
        = R1b
    where
        Self: 'b;
    fn arg(&self) -> u32 {
        self.addr
    }
}
impl<'a, const BLOCK_SIZE: usize> Command<spi::SpiMode> for Cmd25<'a, BLOCK_SIZE> {
    type Resp<'b>
        = spi::R1b
    where
        Self: 'b;
    fn arg(&self) -> u32 {
        self.addr
    }
}
impl<'a, const BLOCK_SIZE: usize, M> BlockCommand<M> for Cmd25<'a, BLOCK_SIZE>
where
    Self: Command<M>,
{
    type Block = Aligned<A4, [u8; BLOCK_SIZE]>;
    fn block_count(&self) -> u32 {
        self.bufs.len() as u32
    }

    fn buf(&mut self) -> &mut [Self::Block] {
        self.bufs
    }
}
impl<'a, const BLOCK_SIZE: usize, M> BlockWriteCommand<M> for Cmd25<'a, BLOCK_SIZE> where
    Self: Command<M>
{
}

/// CMD25: Write multiple blocks
pub fn write_multiple_blocks<const BLOCK_SIZE: usize>(
    addr: u32,
    bufs: &mut [Aligned<A4, [u8; BLOCK_SIZE]>],
) -> Cmd25<'_, BLOCK_SIZE> {
    Cmd25 { addr, bufs }
}

// /// CMD27: Program CSD
// pub fn program_csd() -> Cmd<R1> {
//     cmd(27, 0)
// }

/// CMD38 — ERASE (R1b)
pub struct Cmd38;
impl CommandIndex for Cmd38 {
    const INDEX: u8 = 38;
}
impl Command<SdMode> for Cmd38 {
    type Resp<'a> = R1b;
}
impl Command<spi::SpiMode> for Cmd38 {
    type Resp<'a> = spi::R1b;
}
impl<M> ControlCommand<M> for Cmd38 where Self: Command<M> {}

/// CMD38: Erase all previously selected write blocks
pub fn erase() -> Cmd38 {
    Cmd38
}

/// CMD55 — APP_CMD prefix
pub struct Cmd55 {
    pub rca: u16,
}
impl CommandIndex for Cmd55 {
    const INDEX: u8 = 55;
}
impl Command<SdMode> for Cmd55 {
    type Resp<'a> = R1;
    fn arg(&self) -> u32 {
        (self.rca as u32) << 16
    }
}
impl Command<spi::SpiMode> for Cmd55 {
    type Resp<'a> = spi::R1;
    fn arg(&self) -> u32 {
        0
    }
}
impl<M> ControlCommand<M> for Cmd55 where Self: Command<M> {}

/// CMD55: App Command. Indicates that next command will be a app command
pub fn app_cmd(rca: u16) -> Cmd55 {
    Cmd55 { rca }
}

/// Types of SD Card
#[derive(Debug, Copy, Clone)]
#[non_exhaustive]
#[derive(Default)]
pub enum CardCapacity {
    /// SDSC / Standard Capacity (<= 2GB)
    #[default]
    StandardCapacity,
    /// SDHC / High capacity (<= 32GB for SD cards, <= 256GB for eMMC)
    HighCapacity,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BlockSize {
    B1,
    B2,
    B4,
    B8,
    B16,
    B32,
    B64,
    B128,
    B256,
    B512,
    B1024,
    B2048,
    B4096,
    B8192,
    B16kB,
    Unknown,
}

impl BlockSize {
    /// Length of the block size. Will return 0 if unknown.
    #[allow(clippy::len_without_is_empty)]
    pub const fn len(&self) -> usize {
        match self {
            BlockSize::B1 => 1,
            BlockSize::B2 => 2,
            BlockSize::B4 => 4,
            BlockSize::B8 => 8,
            BlockSize::B16 => 16,
            BlockSize::B32 => 32,
            BlockSize::B64 => 64,
            BlockSize::B128 => 128,
            BlockSize::B256 => 256,
            BlockSize::B512 => 512,
            BlockSize::B1024 => 1024,
            BlockSize::B2048 => 2048,
            BlockSize::B4096 => 4096,
            BlockSize::B8192 => 8192,
            BlockSize::B16kB => 16384,
            _ => 0,
        }
    }
}

pub(crate) const fn block_size(len: usize) -> BlockSize {
    match len {
        1 => BlockSize::B1,
        2 => BlockSize::B2,
        4 => BlockSize::B4,
        8 => BlockSize::B8,
        16 => BlockSize::B16,
        32 => BlockSize::B32,
        64 => BlockSize::B64,
        128 => BlockSize::B128,
        256 => BlockSize::B256,
        512 => BlockSize::B512,
        1024 => BlockSize::B1024,
        2048 => BlockSize::B2048,
        4096 => BlockSize::B4096,
        8192 => BlockSize::B8192,
        16384 => BlockSize::B16kB,
        _ => BlockSize::Unknown,
    }
}

/// CURRENT_STATE enum. Used for R1 response in command queue mode in SD spec, or all R1 responses
/// in eMMC spec.
///
/// Ref PLSS_v7_10 Table 4-75
/// Ref JESD84-B51 Table 68
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
#[repr(u8)]
#[allow(dead_code)]
pub enum CurrentState {
    /// Card is in idle state
    Idle = 0,
    /// Card state is ready
    Ready = 1,
    /// Card is in identification state
    Identification = 2,
    /// Card is in standby state
    Standby = 3,
    /// Card is in transfer state
    Transfer = 4,
    /// Card is sending an operation
    Sending = 5,
    /// Card is receiving operation information
    Receiving = 6,
    /// Card is in programming state
    Programming = 7,
    /// Card is disconnected
    Disconnected = 8,
    /// Card is in bus testing mode. Only valid for eMMC (reserved by SD spec).
    BusTest = 9,
    /// Card is in sleep mode. Only valid for eMMC (reserved by SD spec).
    Sleep = 10,
    // 11 - 15: Reserved
    /// Error
    Error = 128,
}

impl From<u8> for CurrentState {
    fn from(n: u8) -> Self {
        match n {
            0 => Self::Idle,
            1 => Self::Ready,
            2 => Self::Identification,
            3 => Self::Standby,
            4 => Self::Transfer,
            5 => Self::Sending,
            6 => Self::Receiving,
            7 => Self::Programming,
            8 => Self::Disconnected,
            9 => Self::BusTest,
            10 => Self::Sleep,
            _ => Self::Error,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
#[allow(non_camel_case_types)]
pub enum CurrentConsumption {
    I_0mA,
    I_1mA,
    I_5mA,
    I_10mA,
    I_25mA,
    I_35mA,
    I_45mA,
    I_60mA,
    I_80mA,
    I_100mA,
    I_200mA,
}
impl From<&CurrentConsumption> for u32 {
    fn from(i: &CurrentConsumption) -> u32 {
        match i {
            CurrentConsumption::I_0mA => 0,
            CurrentConsumption::I_1mA => 1,
            CurrentConsumption::I_5mA => 5,
            CurrentConsumption::I_10mA => 10,
            CurrentConsumption::I_25mA => 25,
            CurrentConsumption::I_35mA => 35,
            CurrentConsumption::I_45mA => 45,
            CurrentConsumption::I_60mA => 60,
            CurrentConsumption::I_80mA => 80,
            CurrentConsumption::I_100mA => 100,
            CurrentConsumption::I_200mA => 200,
        }
    }
}
impl CurrentConsumption {
    fn from_minimum_reg(reg: u128) -> CurrentConsumption {
        match reg & 0x7 {
            0 => CurrentConsumption::I_0mA,
            1 => CurrentConsumption::I_1mA,
            2 => CurrentConsumption::I_5mA,
            3 => CurrentConsumption::I_10mA,
            4 => CurrentConsumption::I_25mA,
            5 => CurrentConsumption::I_35mA,
            6 => CurrentConsumption::I_60mA,
            _ => CurrentConsumption::I_100mA,
        }
    }
    fn from_maximum_reg(reg: u128) -> CurrentConsumption {
        match reg & 0x7 {
            0 => CurrentConsumption::I_1mA,
            1 => CurrentConsumption::I_5mA,
            2 => CurrentConsumption::I_10mA,
            3 => CurrentConsumption::I_25mA,
            4 => CurrentConsumption::I_35mA,
            5 => CurrentConsumption::I_45mA,
            6 => CurrentConsumption::I_80mA,
            _ => CurrentConsumption::I_200mA,
        }
    }
}
impl fmt::Debug for CurrentConsumption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ma: u32 = self.into();
        write!(f, "{} mA", ma)
    }
}

/// Operation Conditions Register (OCR)
///
/// R3
#[derive(Clone, Copy, Default)]
pub struct OCR<Ext>(pub(crate) u32, pub(crate) PhantomData<Ext>);
impl<Ext> From<R3> for OCR<Ext> {
    fn from(resp: R3) -> Self {
        Self(resp.ocr, PhantomData)
    }
}
impl<Ext> From<R4> for OCR<Ext> {
    fn from(resp: R4) -> Self {
        Self(resp.ocr, PhantomData)
    }
}
impl<Ext> OCR<Ext> {
    /// Card power up status bit (busy)
    pub fn is_busy(&self) -> bool {
        self.0 & 0x8000_0000 == 0 // Set active LOW
    }
}

/// Card Identification Register (CID)
///
/// R2
#[derive(Clone, Copy, Default)]
pub struct CID<Ext> {
    pub(crate) bytes: [u8; 16],
    ext: PhantomData<Ext>,
}
impl<Ext> From<[u8; 16]> for CID<Ext> {
    fn from(bytes: [u8; 16]) -> Self {
        Self {
            bytes: bytes,
            ext: PhantomData,
        }
    }
}
/// From little endian words
impl<Ext> From<R2> for CID<Ext> {
    fn from(resp: R2) -> Self {
        let words = resp.words;
        let inner = ((words[3] as u128) << 96)
            | ((words[2] as u128) << 64)
            | ((words[1] as u128) << 32)
            | words[0] as u128;

        Self {
            bytes: inner.to_be_bytes(),
            ext: PhantomData,
        }
    }
}
impl<Ext> CID<Ext> {
    pub(crate) const fn inner(&self) -> u128 {
        u128::from_be_bytes(self.bytes)
    }

    /// Manufacturer ID
    pub fn manufacturer_id(&self) -> u8 {
        self.bytes[0]
    }
    #[allow(unused)]
    fn crc7(&self) -> u8 {
        (self.bytes[15] >> 1) & 0x7F
    }
}

/// Card Specific Data (CSD)
#[derive(Clone, Copy, Default)]
pub struct CSD<Ext>(pub(crate) u128, PhantomData<Ext>);
impl<Ext> From<u128> for CSD<Ext> {
    fn from(inner: u128) -> Self {
        Self(inner, PhantomData)
    }
}
impl<Ext> From<[u8; 16]> for CSD<Ext> {
    fn from(bytes: [u8; 16]) -> Self {
        let inner = u128::from_ne_bytes(bytes);
        Self(inner, PhantomData)
    }
}
/// From little endian words
impl<Ext> From<R2> for CSD<Ext> {
    fn from(resp: R2) -> Self {
        let words = resp.words;

        let inner = ((words[3] as u128) << 96)
            | ((words[2] as u128) << 64)
            | ((words[1] as u128) << 32)
            | words[0] as u128;
        inner.into()
    }
}

impl<Ext> CSD<Ext> {
    /// CSD structure version
    pub fn version(&self) -> u8 {
        (self.0 >> 126) as u8 & 3
    }
    /// Maximum data transfer rate per one data line
    pub fn transfer_rate(&self) -> u8 {
        (self.0 >> 96) as u8
    }
    /// Maximum block length. In an SD Memory Card the WRITE_BL_LEN is
    /// always equal to READ_BL_LEN
    pub fn block_length(&self) -> BlockSize {
        // Read block length
        match (self.0 >> 80) & 0xF {
            0 => BlockSize::B1,
            1 => BlockSize::B2,
            2 => BlockSize::B4,
            3 => BlockSize::B8,
            4 => BlockSize::B16,
            5 => BlockSize::B32,
            6 => BlockSize::B64,
            7 => BlockSize::B128,
            8 => BlockSize::B256,
            9 => BlockSize::B512,
            10 => BlockSize::B1024,
            11 => BlockSize::B2048,
            12 => BlockSize::B4096,
            13 => BlockSize::B8192,
            14 => BlockSize::B16kB,
            _ => BlockSize::Unknown,
        }
    }
    /// Maximum read current at the minimum VDD
    pub fn read_current_minimum_vdd(&self) -> CurrentConsumption {
        CurrentConsumption::from_minimum_reg((self.0 >> 59) & 0x7)
    }
    /// Maximum write current at the minimum VDD
    pub fn write_current_minimum_vdd(&self) -> CurrentConsumption {
        CurrentConsumption::from_minimum_reg((self.0 >> 56) & 0x7)
    }
    /// Maximum read current at the maximum VDD
    pub fn read_current_maximum_vdd(&self) -> CurrentConsumption {
        CurrentConsumption::from_maximum_reg((self.0 >> 53) & 0x7)
    }
    /// Maximum write current at the maximum VDD
    pub fn write_current_maximum_vdd(&self) -> CurrentConsumption {
        CurrentConsumption::from_maximum_reg((self.0 >> 50) & 0x7)
    }
}

pub trait CardInIdleState {
    fn card_in_idle_state(&self) -> bool;
}

impl<T: CardStatus> CardInIdleState for T {
    fn card_in_idle_state(&self) -> bool {
        matches!(self.state(), CurrentState::Idle)
    }
}

pub trait CardStatus {
    /// Command's argument was out of range
    fn out_of_range(&self) -> bool;
    /// Misaligned address
    fn address_error(&self) -> bool;
    /// Block len error
    fn block_len_error(&self) -> bool;
    /// Error in the erase commands sequence
    fn erase_seq_error(&self) -> bool;
    /// Invalid selection of blocks for erase
    fn erase_param(&self) -> bool;
    /// Host attempted to write to protected area
    fn wp_violation(&self) -> bool;
    /// Card is locked by the host
    fn card_is_locked(&self) -> bool;
    /// Password error
    fn lock_unlock_failed(&self) -> bool;
    /// Crc check of previous command failed
    fn com_crc_error(&self) -> bool;
    /// Command is not legal for the card state
    fn illegal_command(&self) -> bool;
    /// Card internal ECC failed
    fn card_ecc_failed(&self) -> bool;
    /// Internal controller error
    fn cc_error(&self) -> bool;
    /// A General error occurred
    fn error(&self) -> bool;
    /// CSD error
    fn csd_overwrite(&self) -> bool;
    /// Some blocks where skipped while erasing
    fn wp_erase_skip(&self) -> bool;
    /// Erase sequence was aborted
    fn erase_reset(&self) -> bool;
    /// Current card state
    fn state(&self) -> CurrentState;
    /// Corresponds to buffer empty signaling on the bus
    fn ready_for_data(&self) -> bool;
    /// The card will accept a ACMD
    fn app_cmd(&self) -> bool;
}

/// Card Status register
///
/// Error and state information of an executed command
///
/// Ref PLSS_v7_10 Section 4.10.1
#[derive(Clone, Copy)]
pub struct CardStatusRegister<Ext>(pub(crate) u32, pub(crate) PhantomData<Ext>);

impl<Ext> From<R1> for CardStatusRegister<Ext> {
    fn from(resp: R1) -> Self {
        Self(resp.status, PhantomData)
    }
}

impl<Ext> CardStatusRegister<Ext> {
    pub const OUT_OF_RANGE: u32 = 0b1000_0000_0000_0000_0000_0000_0000_0000;
    pub const ADDRESS_ERROR: u32 = 0b0100_0000_0000_0000_0000_0000_0000_0000;
    pub const BLOCK_LEN_ERROR: u32 = 0b0010_0000_0000_0000_0000_0000_0000_0000;
    pub const ERASE_SEQ_ERROR: u32 = 0b0001_0000_0000_0000_0000_0000_0000_0000;
    pub const ERASE_PARAM: u32 = 0b0000_1000_0000_0000_0000_0000_0000_0000;
    pub const WP_VIOLATION: u32 = 0b0000_0100_0000_0000_0000_0000_0000_0000;
    pub const CARD_IS_LOCKED: u32 = 0b0000_0010_0000_0000_0000_0000_0000_0000;
    pub const LOCK_UNLOCK_FAILED: u32 = 0b0000_0001_0000_0000_0000_0000_0000_0000;
    pub const COM_CRC_ERROR: u32 = 0b0000_0000_1000_0000_0000_0000_0000_0000;
    pub const ILLEGAL_COMMAND: u32 = 0b0000_0000_0100_0000_0000_0000_0000_0000;
    pub const CARD_ECC_FAILED: u32 = 0b0000_0000_0010_0000_0000_0000_0000_0000;
    pub const CC_ERROR: u32 = 0b0000_0000_0001_0000_0000_0000_0000_0000;
    pub const ERROR: u32 = 0b0000_0000_0000_1000_0000_0000_0000_0000;
    pub const CSD_OVERWRITE: u32 = 0b0000_0000_0000_0001_0000_0000_0000_0000;
    pub const WP_ERASE_SKIP: u32 = 0b0000_0000_0000_0000_1000_0000_0000_0000;
    pub const CARD_ECC_DISABLED: u32 = 0b0000_0000_0000_0000_0100_0000_0000_0000;
    pub const ERASE_RESET: u32 = 0b0000_0000_0000_0000_0010_0000_0000_0000;
    pub const READY_FOR_DATA: u32 = 0b0000_0000_0000_0000_0000_0001_0000_0000;
    pub const FX_EVENT: u32 = 0b0000_0000_0000_0000_0000_0000_0100_0000;
    pub const APP_CMD: u32 = 0b0000_0000_0000_0000_0000_0000_0010_0000;
    pub const AKE_SEQ_ERROR: u32 = 0b0000_0000_0000_0000_0000_0000_0000_1000;
    pub const STATE_BITS_MASK: u32 = 0b0000_0000_0000_0000_0001_1110_0000_0000;
}

impl<Ext> core::ops::BitOr for CardStatusRegister<Ext> {
    type Output = CardStatusRegister<Ext>;
    fn bitor(self, rhs: Self) -> Self::Output {
        CardStatusRegister(self.0 | rhs.0, PhantomData)
    }
}

impl<Ext> core::ops::BitOrAssign for CardStatusRegister<Ext> {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = CardStatusRegister(self.0 | rhs.0, PhantomData)
    }
}

impl CardStatus for R1 {
    /// Command's argument was out of range
    fn out_of_range(&self) -> bool {
        self.status & CardStatusRegister::<()>::OUT_OF_RANGE != 0
    }
    /// Misaligned address
    fn address_error(&self) -> bool {
        self.status & CardStatusRegister::<()>::ADDRESS_ERROR != 0
    }
    /// Block len error
    fn block_len_error(&self) -> bool {
        self.status & CardStatusRegister::<()>::BLOCK_LEN_ERROR != 0
    }
    /// Error in the erase commands sequence
    fn erase_seq_error(&self) -> bool {
        self.status & CardStatusRegister::<()>::ERASE_SEQ_ERROR != 0
    }
    /// Invalid selection of blocks for erase
    fn erase_param(&self) -> bool {
        self.status & CardStatusRegister::<()>::ERASE_PARAM != 0
    }
    /// Host attempted to write to protected area
    fn wp_violation(&self) -> bool {
        self.status & CardStatusRegister::<()>::WP_VIOLATION != 0
    }
    /// Card is locked by the host
    fn card_is_locked(&self) -> bool {
        self.status & CardStatusRegister::<()>::CARD_IS_LOCKED != 0
    }
    /// Password error
    fn lock_unlock_failed(&self) -> bool {
        self.status & CardStatusRegister::<()>::LOCK_UNLOCK_FAILED != 0
    }
    /// Crc check of previous command failed
    fn com_crc_error(&self) -> bool {
        self.status & CardStatusRegister::<()>::COM_CRC_ERROR != 0
    }
    /// Command is not legal for the card state
    fn illegal_command(&self) -> bool {
        self.status & CardStatusRegister::<()>::ILLEGAL_COMMAND != 0
    }
    /// Card internal ECC failed
    fn card_ecc_failed(&self) -> bool {
        self.status & CardStatusRegister::<()>::CARD_ECC_FAILED != 0
    }
    /// Internal controller error
    fn cc_error(&self) -> bool {
        self.status & CardStatusRegister::<()>::CC_ERROR != 0
    }
    /// A General error occurred
    fn error(&self) -> bool {
        self.status & CardStatusRegister::<()>::ERROR != 0
    }
    /// CSD error
    fn csd_overwrite(&self) -> bool {
        self.status & CardStatusRegister::<()>::CSD_OVERWRITE != 0
    }
    /// Some blocks where skipped while erasing
    fn wp_erase_skip(&self) -> bool {
        self.status & CardStatusRegister::<()>::WP_ERASE_SKIP != 0
    }
    /// Erase sequence was aborted
    fn erase_reset(&self) -> bool {
        self.status & CardStatusRegister::<()>::ERASE_RESET != 0
    }
    /// Current card state
    fn state(&self) -> CurrentState {
        CurrentState::from(((self.status & CardStatusRegister::<()>::STATE_BITS_MASK) >> 9) as u8)
    }
    /// Corresponds to buffer empty signaling on the bus
    fn ready_for_data(&self) -> bool {
        self.status & CardStatusRegister::<()>::READY_FOR_DATA != 0
    }
    /// The card will accept a ACMD
    fn app_cmd(&self) -> bool {
        self.status & CardStatusRegister::<()>::APP_CMD != 0
    }
}

impl<Ext> CardStatus for CardStatusRegister<Ext> {
    /// Command's argument was out of range
    fn out_of_range(&self) -> bool {
        self.0 & Self::OUT_OF_RANGE != 0
    }
    /// Misaligned address
    fn address_error(&self) -> bool {
        self.0 & Self::ADDRESS_ERROR != 0
    }
    /// Block len error
    fn block_len_error(&self) -> bool {
        self.0 & Self::BLOCK_LEN_ERROR != 0
    }
    /// Error in the erase commands sequence
    fn erase_seq_error(&self) -> bool {
        self.0 & Self::ERASE_SEQ_ERROR != 0
    }
    /// Invalid selection of blocks for erase
    fn erase_param(&self) -> bool {
        self.0 & Self::ERASE_PARAM != 0
    }
    /// Host attempted to write to protected area
    fn wp_violation(&self) -> bool {
        self.0 & Self::WP_VIOLATION != 0
    }
    /// Card is locked by the host
    fn card_is_locked(&self) -> bool {
        self.0 & Self::CARD_IS_LOCKED != 0
    }
    /// Password error
    fn lock_unlock_failed(&self) -> bool {
        self.0 & Self::LOCK_UNLOCK_FAILED != 0
    }
    /// Crc check of previous command failed
    fn com_crc_error(&self) -> bool {
        self.0 & Self::COM_CRC_ERROR != 0
    }
    /// Command is not legal for the card state
    fn illegal_command(&self) -> bool {
        self.0 & Self::ILLEGAL_COMMAND != 0
    }
    /// Card internal ECC failed
    fn card_ecc_failed(&self) -> bool {
        self.0 & Self::CARD_ECC_FAILED != 0
    }
    /// Internal controller error
    fn cc_error(&self) -> bool {
        self.0 & Self::CC_ERROR != 0
    }
    /// A General error occurred
    fn error(&self) -> bool {
        self.0 & Self::ERROR != 0
    }
    /// CSD error
    fn csd_overwrite(&self) -> bool {
        self.0 & Self::CSD_OVERWRITE != 0
    }
    /// Some blocks where skipped while erasing
    fn wp_erase_skip(&self) -> bool {
        self.0 & Self::WP_ERASE_SKIP != 0
    }
    /// Erase sequence was aborted
    fn erase_reset(&self) -> bool {
        self.0 & Self::ERASE_RESET != 0
    }
    /// Current card state
    fn state(&self) -> CurrentState {
        CurrentState::from(((self.0 & Self::STATE_BITS_MASK) >> 9) as u8)
    }
    /// Corresponds to buffer empty signaling on the bus
    fn ready_for_data(&self) -> bool {
        self.0 & Self::READY_FOR_DATA != 0
    }
    /// The card will accept a ACMD
    fn app_cmd(&self) -> bool {
        self.0 & Self::APP_CMD != 0
    }
}

/// Relative Card Address (RCA)
///
/// R6
#[derive(Debug, Copy, Clone, Default)]
pub struct RCA<Ext>(pub(crate) u32, pub(crate) PhantomData<Ext>);
impl<Ext> From<R6> for RCA<Ext> {
    fn from(resp: R6) -> Self {
        Self(
            ((resp.rca as u32) << 16) | (resp.status as u32),
            PhantomData,
        )
    }
}
impl<Ext> RCA<Ext> {
    /// Address of card
    pub fn address(&self) -> u16 {
        (self.0 >> 16) as u16
    }
}
