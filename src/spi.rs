use embedded_hal::digital::OutputPin;
use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::spi::SpiBus;

use crate::{
    BlockReadCommand, BlockWriteCommand, BusWidth, ByteReadCommand, ByteWriteCommand, Command, CommandIndex, ControlCommand, ResponseLenBytes, MmcBus, MmcError, Response, ResponseWords
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
/// 8-bit, CRC-checked, no busy
pub struct R1 {
    pub status: u8,
}

impl Response for R1 {
    type Words = [u8; 1];

    const CRC: bool = true;
    const BUSY: bool = false;

    fn from_words(buf: &Self::Words) -> Self {
        Self { status: buf[0] }
    }
}

/// R1b — R1 + busy on DAT0
///
/// 8-bit, CRC-checked, *busy*
/// Card holds DAT0 low until internal operation completes.
pub struct R1b {
    pub status: u8,
}

impl Response for R1b {
    type Words = [u8; 1];

    const CRC: bool = true;
    const BUSY: bool = false;

    fn from_words(buf: &Self::Words) -> Self {
        Self { status: buf[0] }
    }
}

/// R2 — Send Status response
///
/// 16-bit, CRC-checked
pub struct R2 {
    pub bytes: [u8; 2]
}

impl Response for R2 {
    type Words = [u8; 2];

    const CRC: bool = true;
    const BUSY: bool = false;

    fn from_words(buf: &Self::Words) -> Self {
        Self { bytes: *buf }
    }
}

/// R7 — Send Status response
///
/// 40-bit, CRC-checked
pub struct R7 {
    pub bytes: [u8; 5],
}

impl Response for R7 {
    type Words = [u8; 5];

    const CRC: bool = true;
    const BUSY: bool = false;

    fn from_words(buf: &Self::Words) -> Self {
        Self { bytes: *buf }
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
    {
        let r1 = self.read_r1().await?;

        let total_bytes = <R::Words as ResponseWords>::LEN.into();

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

        let mut words = [0u32; 4];
        for (i, chunk) in raw[..=total_bytes].chunks(4).take(words.len()).enumerate() {
            let mut w = 0u32;
            for &b in chunk {
                w = (w << 8) | b as u32;
            }
            words[i] = w;
        }

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

    async fn read_bytes<'a, C>(&mut self, mut cmd: C) -> Result<C::Resp<'a>, MmcError>
    where
        C: ByteReadCommand + 'a,
    {
        self.send_cmd_header(&cmd).await?;
        let len = cmd.byte_count();
        let slice = &mut cmd.buf()[..len];

        self.read_block(slice).await?;

        let resp = self.read_response_words::<C::Resp<'_>>().await?;
        self.deselect().await?;
        Ok(resp)
    }

    async fn write_bytes<'a, C>(&mut self, mut cmd: C) -> Result<C::Resp<'a>, MmcError>
    where
        C: ByteWriteCommand + 'a,
    {
        self.send_cmd_header(&cmd).await?;
        let len = cmd.byte_count();
        let slice = &mut cmd.buf()[..len];

        self.write_block(slice).await?;

        let resp = self.read_response_words::<C::Resp<'_>>().await?;
        self.deselect().await?;
        Ok(resp)
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
