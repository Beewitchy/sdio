use core::marker::PhantomData;

use embedded_hal::digital::OutputPin;
use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::spi::SpiBus;

use crate::{
    BlockCommand, BlockReadCommand, BlockWriteCommand, BusWidth, ByteReadCommand, ByteWriteCommand,
    Command, CommandIndex, ControlCommand, FromBytes, MmcBus, MmcError, Response, ResponseLenBytes,
    ResponseWords, PowerUpReady, common, sd,
};

/// Marker trait for commands in SPI mode
///
/// See also [crate::SdMode].
pub struct SpiMode;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SpiResponseLen {
    R8,
    R16,
    R40,
}

impl SpiResponseLen {
    pub const fn bytes(&self) -> usize {
        match self {
            Self::R8 => 1,
            Self::R16 => 2,
            Self::R40 => 5,
        }
    }
}

impl ResponseLenBytes for SpiResponseLen {
    fn bytes(&self) -> usize {
        self.bytes()
    }
}

/// CMD9 — SEND_CSD
pub struct Cmd9<'a> {
    pub buf: &'a mut [aligned::Aligned<aligned::A4, [u8; 16]>],
}
impl<'a> CommandIndex for Cmd9<'a> {
    const INDEX: u8 = 9;
}
impl<'a> Command<SpiMode> for Cmd9<'a> {
    type Resp<'r> = R1 where Self: 'r;
    fn arg(&self) -> u32 {
        0
    }
}
impl<'a> BlockCommand<SpiMode> for Cmd9<'a> {
    type Block = aligned::Aligned<aligned::A4, [u8; 16]>;

    fn block_count(&self) -> u32 {
        1
    }

    fn buf(&mut self) -> &mut [Self::Block] {
        &mut *self.buf
    }
}
impl<'a> BlockReadCommand<SpiMode> for Cmd9<'a> {}

/// CMD9: Send CSD
pub fn send_csd<'a>(buf: &'a mut aligned::Aligned<aligned::A4, [u8; 16]>) -> Cmd9<'a> {
    Cmd9 {
        buf: core::slice::from_mut(buf),
    }
}

/// CMD10 — SEND_CID
pub struct Cmd10<'a> {
    pub buf: &'a mut [aligned::Aligned<aligned::A4, [u8; 16]>],
}
impl<'a> CommandIndex for Cmd10<'a> {
    const INDEX: u8 = 10;
}
impl<'a> Command<SpiMode> for Cmd10<'a> {
    type Resp<'r> = R1 where Self: 'r;
    fn arg(&self) -> u32 {
        0
    }
}
impl<'a> BlockCommand<SpiMode> for Cmd10<'a> {
    type Block = aligned::Aligned<aligned::A4, [u8; 16]>;

    fn block_count(&self) -> u32 {
        1
    }

    fn buf(&mut self) -> &mut [Self::Block] {
        &mut *self.buf
    }
}
impl<'a> BlockReadCommand<SpiMode> for Cmd10<'a> {}

/// CMD10: Send CID
pub fn send_cid<'a>(buf: &'a mut aligned::Aligned<aligned::A4, [u8; 16]>) -> Cmd10<'a> {
    Cmd10 {
        buf: core::slice::from_mut(buf),
    }
}

/// CMD13: Ask card to send status
pub const fn card_status() -> common::Cmd13 {
    common::Cmd13 {
        rca: 0,
        task_status: false,
    }
}

/// ACMD41 — SD_SEND_OP_COND
///
/// In SPI mode only the HCS bit is supported.
///
/// The trait implementation for SpiMode automatically ignores the
/// unsupported arguments, so this helper function is just for
/// convenience when using a known SPI mode bus.
pub const fn sd_send_op_cond(host_high_capacity_support: bool) -> sd::Acmd41 {
    sd::Acmd41 {
        host_high_capacity_support,
        sdxc_power_control: false,
        switch_to_1_8v_request: false,
        voltage_window: 0,
    }
}

/// CMD55: App Command. Indicates that next command will be a app command
///
/// In SPI mode the rca (card address register) is not used.
///
/// The trait implementation for SpiMode ignores the rca argument,
/// so this helper function is just for convenience when using a
/// known SPI mode bus.
pub const fn app_cmd() -> common::Cmd55 {
    common::Cmd55 { rca: 0 }
}

/// CRC_ON_OFF for SPI mode
///
/// CRC checking is disabled by default in SPI mode.
///
/// This command should be sent BEFORE ACmd41 to change
/// the setting.
///
/// See Physical Layer Simplified Specification section 7.2.2
/// Bus Transfer Protection.
pub enum Cmd59 {
    On,
    Off,
}

impl CommandIndex for Cmd59 {
    const INDEX: u8 = 59;
}
impl Command<SpiMode> for Cmd59 {
    type Resp<'a> = R1;
    fn arg(&self) -> u32 {
        match self {
            Self::On => 0x00000001,
            Self::Off => 0x00000000,
        }
    }
}
impl ControlCommand<SpiMode> for Cmd59 {}

pub type CrcOnOff = Cmd59;

impl ResponseWords for [u8; 1] {
    type Word = u8;
    type Len = SpiResponseLen;

    const LEN: Self::Len = SpiResponseLen::R8;
}

impl ResponseWords for [u8; 2] {
    type Word = u8;
    type Len = SpiResponseLen;

    const LEN: Self::Len = SpiResponseLen::R16;
}

impl ResponseWords for [u8; 5] {
    type Word = u8;
    type Len = SpiResponseLen;

    const LEN: Self::Len = SpiResponseLen::R40;
}

/// R1 — Normal status response
///
/// 8-bit, no busy
#[derive(Clone, Copy)]
pub struct R1 {
    pub result: u8,
}

impl R1 {
    pub const PARAMETER_ERROR: u8 = 0b0100_0000;
    pub const ADDRESS_ERROR: u8 = 0b0010_0000;
    pub const ERASE_SEQ_ERROR: u8 = 0b0001_0000;
    pub const COM_CRC_ERROR: u8 = 0b0000_1000;
    pub const ILLEGAL_COMMAND: u8 = 0b0000_0100;
    pub const ERASE_RESET: u8 = 0b0000_0010;
    pub const IN_IDLE_STATE: u8 = 0b0000_0001;
}

impl Response for R1 {
    type Words = [u8; 1];

    const CRC: bool = false;
    const BUSY: bool = false;

    #[inline]
    fn from_words(buf: &Self::Words) -> Self {
        Self { result: buf[0] }
    }

    #[inline]
    fn to_result(self) -> Result<Self, MmcError> {
        match self.result.isolate_highest_one() {
            Self::PARAMETER_ERROR => Err(MmcError::Card(crate::CardError::OutOfRange)),
            Self::ADDRESS_ERROR => Err(MmcError::Card(crate::CardError::AddressError)),
            Self::ERASE_SEQ_ERROR => Err(MmcError::Card(crate::CardError::EraseSeqError)),
            Self::COM_CRC_ERROR => Err(MmcError::Card(crate::CardError::ComCrcError)),
            Self::ILLEGAL_COMMAND => Err(MmcError::Card(crate::CardError::IllegalCommand)),
            Self::ERASE_RESET => Err(MmcError::Card(crate::CardError::EraseReset)),
            _ => Ok(self),
        }
    }
}

impl common::CardInIdleState for R1 {
    fn card_in_idle_state(&self) -> bool {
        self.result & Self::IN_IDLE_STATE != 0
    }
}

impl<T: common::CardInIdleState + Response> PowerUpReady for T {
    fn ready(&self) -> bool {
        self.card_in_idle_state()
    }
}

/// R1b — R1 + busy on DAT0
///
/// 8-bit, *busy*
/// Card holds DAT0 low until internal operation completes.
pub struct R1b {
    pub response: R1,
}

impl Response for R1b {
    type Words = <R1 as Response>::Words;

    const CRC: bool = <R1 as Response>::CRC;
    const BUSY: bool = true;

    #[inline]
    fn from_words(buf: &Self::Words) -> Self {
        Self {
            response: R1::from_words(buf),
        }
    }

    #[inline]
    fn to_result(self) -> Result<Self, MmcError> {
        self.response.to_result().and(Ok(self))
    }
}

/// R2 — Send Status response
///
/// 16-bit
pub struct R2 {
    pub result: R1,
    pub status: u8,
}

impl Response for R2 {
    type Words = [u8; 2];

    const CRC: bool = false;
    const BUSY: bool = false;

    #[inline]
    fn from_words(buf: &Self::Words) -> Self {
        Self {
            result: R1::from_words(&[buf[0]]),
            status: buf[1],
        }
    }

    #[inline]
    fn to_result(self) -> Result<Self, MmcError> {
        self.result.to_result().and(match self.status.isolate_highest_one() {
            Self::CSD_OVERWRITE => Err(MmcError::Card(crate::CardError::CidCsdOverwrite)),
            Self::ERASE_PARAM => Err(MmcError::Card(crate::CardError::EraseParamError)),
            Self::WP_VIOLATION => Err(MmcError::Card(crate::CardError::WriteProtViolation)),
            Self::CARD_ECC_FAILED => Err(MmcError::Card(crate::CardError::CardEccFailed)),
            Self::CC_ERROR => Err(MmcError::Card(crate::CardError::CcError)),
            Self::ERROR => Err(MmcError::Card(crate::CardError::Error)),
            Self::LOCK_UNLOCK_FAILED => Err(MmcError::Card(crate::CardError::LockUnlockFailed)),
            _ => Ok(self),
        })
    }
}

impl R2 {
    // NOTE: Two bits, 7 and 1, are overloaded and can each indicate one of two errors
    // depending on context (which ever error is relevant to the sent command).
    pub const OUT_OF_RANGE: u8 = 0b1000_0000;
    pub const CSD_OVERWRITE: u8 = 0b1000_0000;
    pub const ERASE_PARAM: u8 = 0b0100_0000;
    pub const WP_VIOLATION: u8 = 0b0010_0000;
    pub const CARD_ECC_FAILED: u8 = 0b0001_0000;
    pub const CC_ERROR: u8 = 0b0000_1000;
    pub const ERROR: u8 = 0b0000_0100;
    pub const WP_ERASE_SKIP: u8 = 0b0000_0010;
    pub const LOCK_UNLOCK_FAILED: u8 = 0b0000_0010;
    pub const CARD_IS_LOCKED: u8 = 0b0000_0001;
}

impl common::CardStatus for R2 {
    fn out_of_range(&self) -> bool {
        self.status & Self::OUT_OF_RANGE != 0
    }

    fn address_error(&self) -> bool {
        self.result.result & R1::ADDRESS_ERROR != 0
    }

    fn block_len_error(&self) -> bool {
        false
    }

    fn erase_seq_error(&self) -> bool {
        self.result.result & R1::ERASE_SEQ_ERROR != 0
    }

    fn erase_param(&self) -> bool {
        self.status & Self::ERASE_PARAM != 0
    }

    fn wp_violation(&self) -> bool {
        self.status & Self::WP_VIOLATION != 0
    }

    fn card_is_locked(&self) -> bool {
        self.status & Self::CARD_IS_LOCKED != 0
    }

    fn lock_unlock_failed(&self) -> bool {
        self.status & Self::LOCK_UNLOCK_FAILED != 0
    }

    fn com_crc_error(&self) -> bool {
        self.result.result & R1::COM_CRC_ERROR != 0
    }

    fn illegal_command(&self) -> bool {
        self.result.result & R1::ILLEGAL_COMMAND != 0
    }

    fn card_ecc_failed(&self) -> bool {
        self.status & Self::CARD_ECC_FAILED != 0
    }

    fn cc_error(&self) -> bool {
        self.status & Self::CC_ERROR != 0
    }

    fn error(&self) -> bool {
        self.status & Self::ERROR != 0
    }

    fn csd_overwrite(&self) -> bool {
        self.status & Self::CSD_OVERWRITE != 0
    }

    fn wp_erase_skip(&self) -> bool {
        self.status & Self::WP_ERASE_SKIP != 0
    }

    fn erase_reset(&self) -> bool {
        self.result.result & R1::ERASE_RESET != 0
    }

    fn state(&self) -> sd::CurrentState {
        if self.result.result & R1::IN_IDLE_STATE != 0 {
            sd::CurrentState::Idle
        } else {
            sd::CurrentState::Ready
        }
    }

    fn ready_for_data(&self) -> bool {
        false
    }

    fn app_cmd(&self) -> bool {
        false
    }
}

impl<Ext> From<R2> for common::CardStatusRegister<Ext> {
    fn from(resp: R2) -> Self {
        let mut register_bits = 0u32;
        if resp.result.result & R1::PARAMETER_ERROR != 0 {
            register_bits |= Self::OUT_OF_RANGE;
        }
        if resp.result.result & R1::ADDRESS_ERROR != 0 {
            register_bits |= Self::ADDRESS_ERROR;
        }
        if resp.result.result & R1::ERASE_SEQ_ERROR != 0 {
            register_bits |= Self::ERASE_SEQ_ERROR;
        }
        if resp.result.result & R1::COM_CRC_ERROR != 0 {
            register_bits |= Self::COM_CRC_ERROR;
        }
        if resp.result.result & R1::ILLEGAL_COMMAND != 0 {
            register_bits |= Self::ILLEGAL_COMMAND;
        }
        if resp.result.result & R1::ERASE_RESET != 0 {
            register_bits |= Self::ERASE_RESET;
        }
        if resp.result.result & R1::IN_IDLE_STATE == 0 {
            register_bits |= ((common::CurrentState::Ready as u32) << 9) & Self::STATE_BITS_MASK;
        }
        if resp.status & R2::CSD_OVERWRITE != 0 {
            register_bits |= Self::CSD_OVERWRITE;
        }
        if resp.status & R2::ERASE_PARAM != 0 {
            register_bits |= Self::ERASE_PARAM;
        }
        if resp.status & R2::WP_VIOLATION != 0 {
            register_bits |= Self::WP_VIOLATION;
        }
        if resp.status & R2::CARD_ECC_FAILED != 0 {
            register_bits |= Self::CARD_ECC_FAILED;
        }
        if resp.status & R2::CC_ERROR != 0 {
            register_bits |= Self::CC_ERROR;
        }
        if resp.status & R2::ERROR != 0 {
            register_bits |= Self::ERROR;
        }
        if resp.status & R2::LOCK_UNLOCK_FAILED == 0 {
            register_bits |= Self::LOCK_UNLOCK_FAILED;
        }
        if resp.status & R2::CARD_IS_LOCKED == 0 {
            register_bits |= Self::CARD_IS_LOCKED;
        }
        Self(register_bits, PhantomData)
    }
}

/// R3 — Read OCR response
///
/// 40-bit
pub struct R3 {
    pub response: R1,
    pub ocr: u32,
}

impl Response for R3 {
    type Words = [u8; 5];

    const CRC: bool = false;
    const BUSY: bool = false;

    #[inline]
    fn from_words(buf: &Self::Words) -> Self {
        let response = R1::from_words(&[buf[0]]);
        let (_, ocr) = buf.split_last_chunk().unwrap();
        let ocr = u32::from_ne_bytes(*ocr);
        Self { response, ocr }
    }

    #[inline]
    fn to_result(self) -> Result<Self, MmcError> {
        self.response.to_result().and(Ok(self))
    }
}

impl<Ext> From<R3> for common::OCR<Ext> {
    fn from(value: R3) -> Self {
        Self(value.ocr, PhantomData)
    }
}

/// R7 — Send Status response
///
/// 40-bit
pub struct R7 {
    pub response: R1,
    pub voltage: u8,
    pub check_pattern: u8,
}

impl Response for R7 {
    type Words = [u8; 5];

    const CRC: bool = false;
    const BUSY: bool = false;

    #[inline]
    fn from_words(buf: &Self::Words) -> Self {
        let response = R1::from_words(&[buf[0]]);
        let voltage = 0b0000_1111 & buf[3];
        let check_pattern = buf[4];
        Self {
            response,
            voltage,
            check_pattern,
        }
    }

    fn to_result(self) -> Result<Self, MmcError> {
        self.response.to_result().and(Ok(self))
    }
}

/// Modified R1 — Normal status response for SDIO
///
/// 8-bit, no busy
#[derive(Clone, Copy)]
pub struct R1M {
    pub response: u8,
}

impl Response for R1M {
    type Words = [u8; 1];

    const CRC: bool = false;
    const BUSY: bool = false;

    #[inline]
    fn from_words(buf: &Self::Words) -> Self {
        Self { response: buf[0] }
    }

    #[inline]
    fn to_result(self) -> Result<Self, MmcError> {
        match self.response {
            0b0100_0000 => Err(MmcError::Card(crate::CardError::OutOfRange)),
            // TODO: Sort out R1 error status handling: this should be function number
            0b0001_0000 => Err(MmcError::Card(crate::CardError::AddressError)),
            0b0000_1000 => Err(MmcError::Card(crate::CardError::ComCrcError)),
            0b0000_0100 => Err(MmcError::Card(crate::CardError::IllegalCommand)),
            _ => Ok(self),
        }
    }
}

impl common::CardInIdleState for R1M {
    fn card_in_idle_state(&self) -> bool {
        self.response & 0b0000_0001 == 0b0000_0001
    }
}

/// R4 — SDIO OCR + capability
///
/// 40-bit, no busy
/// Returned by CMD5 (IO_SEND_OP_COND)
pub struct R4 {
    pub response: R1M,
    pub ocr: u32,
}

impl Response for R4 {
    type Words = [u8; 5];
    const CRC: bool = false;
    const BUSY: bool = false;

    #[inline]
    fn from_words(buf: &Self::Words) -> Self {
        let response = R1M::from_words(&[buf[0]]);
        let (_, ocr) = buf.split_last_chunk().unwrap();
        let ocr = u32::from_ne_bytes(*ocr);
        Self { response, ocr }
    }

    #[inline]
    fn to_result(self) -> Result<Self, MmcError> {
        self.response.to_result().and(Ok(self))
    }
}

impl common::CardInIdleState for R4 {
    fn card_in_idle_state(&self) -> bool {
        self.response.card_in_idle_state()
    }
}

pub trait SetHz {
    fn set_hz(&mut self, hz: u32);
}

/// CRC7 over the 5 command bytes (start+index+arg), poly x^7+x^3+1 (0x89).
fn crc7(data: &[u8]) -> u8 {
    let mut crc: u8 = 0;
    for &byte in data {
        crc ^= byte;
        for _ in 0..8 {
            if crc & 0x80 != 0 {
                crc ^= 0x89;
            }
            crc <<= 1;
        }
    }
    crc >> 1
}

pub struct SpiMmcBus<SPI, CS, DLY> {
    spi: SPI,
    cs: CS,
    delay: DLY,
}

impl<SPI, CS, DLY> SpiMmcBus<SPI, CS, DLY> {
    pub fn new(spi: SPI, cs: CS, delay: DLY) -> Self {
        Self { spi, cs, delay }
    }

    async fn select(&mut self) -> Result<(), MmcError>
    where
        CS: OutputPin,
    {
        self.cs.set_low().map_err(|_| MmcError::Io)
    }

    async fn deselect(&mut self) -> Result<(), MmcError>
    where
        CS: OutputPin,
        SPI: SpiBus<u8>,
    {
        self.cs.set_high().map_err(|_| MmcError::Io)?;
        let _ = self.spi.write(&[0xFF]).await;
        Ok(())
    }

    async fn send_cmd_header<C: Command<SpiMode>>(&mut self, cmd: &C) -> Result<(), MmcError>
    where
        SPI: SpiBus<u8>,
        CS: OutputPin,
    {
        self.select().await?;

        let idx = cmd.index() & 0x3F;
        let arg = cmd.arg();

        let mut buf = [
            0x40 | idx,
            (arg >> 24) as u8,
            (arg >> 16) as u8,
            (arg >> 8) as u8,
            arg as u8,
            0,
        ];
        buf[5] = (crc7(&buf[..5]) << 1) | 1;

        self.spi.write(&buf).await.map_err(|_| MmcError::Io)
    }

    async fn read_r1(&mut self) -> Result<u8, MmcError>
    where
        SPI: SpiBus<u8>,
    {
        let mut b = [0xFF];
        for _ in 0..8 {
            self.spi.read(&mut b).await.map_err(|_| MmcError::Io)?;
            if b[0] != 0xFF {
                return Ok(b[0]);
            }
        }
        Err(MmcError::Timeout)
    }

    async fn wait_not_busy(&mut self) -> Result<(), MmcError>
    where
        SPI: SpiBus<u8>,
    {
        let mut b = [0xFF];
        for _ in 0..65_536 {
            self.spi.read(&mut b).await.map_err(|_| MmcError::Io)?;
            if b[0] == 0xFF {
                return Ok(());
            }
        }
        Err(MmcError::Busy)
    }

    async fn read_response_words<R: Response>(&mut self) -> Result<R, MmcError>
    where
        SPI: SpiBus<u8>,
        R::Words: FromBytes,
    {
        let r1 = self.read_r1().await?;

        let total_bytes = R::Words::LEN.bytes();

        let mut raw = [0u8; 1 + 16];
        raw[0] = r1;

        if total_bytes > 0 {
            let mut tmp = [0xFFu8; 16];
            self.spi
                .read(&mut tmp[..total_bytes])
                .await
                .map_err(|_| MmcError::Io)?;
            raw[1..=total_bytes].copy_from_slice(&tmp[..total_bytes]);
        }

        let words = FromBytes::from_bytes(raw);

        if R::BUSY {
            self.wait_not_busy().await?;
        }

        Ok(R::from_words(&words))
    }

    async fn read_block(&mut self, buf: &mut [u8]) -> Result<(), MmcError>
    where
        SPI: SpiBus<u8>,
    {
        let mut b = [0xFF];
        for _ in 0..65_536 {
            self.spi.read(&mut b).await.map_err(|_| MmcError::Io)?;
            if b[0] == 0xFE {
                break;
            }
        }
        if b[0] != 0xFE {
            return Err(MmcError::Timeout);
        }

        let mut tmp = [0xFFu8; 512];
        let len = buf.len().min(512);
        self.spi
            .read(&mut tmp[..len])
            .await
            .map_err(|_| MmcError::Io)?;
        buf.copy_from_slice(&tmp[..len]);

        let mut crc = [0xFFu8; 2];
        self.spi.read(&mut crc).await.map_err(|_| MmcError::Io)?;

        Ok(())
    }

    async fn write_block(&mut self, buf: &[u8]) -> Result<(), MmcError>
    where
        SPI: SpiBus<u8>,
    {
        self.spi.write(&[0xFE]).await.map_err(|_| MmcError::Io)?;
        self.spi.write(buf).await.map_err(|_| MmcError::Io)?;
        self.spi
            .write(&[0xFF, 0xFF])
            .await
            .map_err(|_| MmcError::Io)?;

        let mut resp = [0xFF];
        self.spi.read(&mut resp).await.map_err(|_| MmcError::Io)?;
        if (resp[0] & 0x1F) != 0x05 {
            return Err(MmcError::Crc);
        }

        self.wait_not_busy().await
    }
}

impl<SPI, CS, DLY, E> MmcBus for SpiMmcBus<SPI, CS, DLY>
where
    SPI: SpiBus<u8, Error = E> + SetHz,
    CS: OutputPin,
    DLY: DelayNs,
{
    type Mode = SpiMode;

    async fn send_command<'a, C>(&mut self, cmd: C) -> Result<C::Resp<'a>, MmcError>
    where
        C: ControlCommand<Self::Mode> + 'a,
    {
        self.send_cmd_header(&cmd).await?;
        let resp = self.read_response_words::<C::Resp<'_>>().await?;
        self.deselect().await?;
        Ok(resp)
    }

    async fn read_blocks<'a, C>(
        &mut self,
        mut cmd: C,
        auto_stop: bool,
    ) -> Result<C::Resp<'a>, MmcError>
    where
        C: BlockReadCommand<Self::Mode> + 'a,
    {
        if auto_stop {
            return Err(MmcError::Unsupported);
        }

        self.send_cmd_header(&cmd).await?;
        let total = cmd.block_count() as usize;
        let slice = &mut cmd.buf()[..total];

        use as_slice::AsMutSlice as _;
        for block in slice {
            self.read_block(block.as_mut_slice()).await?;
        }

        let resp = self.read_response_words::<C::Resp<'_>>().await?;
        self.deselect().await?;
        Ok(resp)
    }

    async fn write_blocks<'a, C>(
        &mut self,
        mut cmd: C,
        auto_stop: bool,
    ) -> Result<C::Resp<'a>, MmcError>
    where
        C: BlockWriteCommand<Self::Mode> + 'a,
    {
        if auto_stop {
            return Err(MmcError::Unsupported);
        }

        self.send_cmd_header(&cmd).await?;
        let total = cmd.block_count() as usize;
        let slice = &mut cmd.buf()[..total];

        use as_slice::AsSlice as _;
        for block in slice {
            self.write_block(block.as_slice()).await?;
        }

        let resp = self.read_response_words::<C::Resp<'_>>().await?;
        self.deselect().await?;
        Ok(resp)
    }

    async fn read_bytes<'a, C>(&mut self, _cmd: C) -> Result<C::Resp<'a>, MmcError>
    where
        C: ByteReadCommand + 'a,
    {
        Err(MmcError::Unsupported)
    }

    async fn write_bytes<'a, C>(&mut self, _cmd: C) -> Result<C::Resp<'a>, MmcError>
    where
        C: ByteWriteCommand + 'a,
    {
        Err(MmcError::Unsupported)
    }

    async fn init_idle(&mut self, hz: u32) -> Result<(), MmcError> {
        self.spi.set_hz(hz);

        self.cs.set_high().map_err(|_| MmcError::Io)?;
        let dummy = [0xFFu8; 10];
        self.spi.write(&dummy).await.map_err(|_| MmcError::Io)?;
        self.delay.delay_us(1000).await;
        Ok(())
    }

    fn set_bus(&mut self, _width: BusWidth, hz: u32) -> Result<(), MmcError> {
        self.spi.set_hz(hz);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::crc7;

    // Framed command CRC byte sent on the wire = (crc7 << 1) | 1.
    fn framed(bytes: &[u8]) -> u8 {
        (crc7(bytes) << 1) | 1
    }

    #[test]
    fn crc7_matches_known_command_vectors() {
        // SD spec well-known CRCs for these command frames.
        assert_eq!(framed(&[0x40, 0x00, 0x00, 0x00, 0x00]), 0x95); // CMD0
        assert_eq!(framed(&[0x51, 0x00, 0x00, 0x00, 0x00]), 0x55); // CMD17, arg 0
        assert_eq!(framed(&[0x48, 0x00, 0x00, 0x01, 0xAA]), 0x87); // CMD8, arg 0x1AA
    }
}
