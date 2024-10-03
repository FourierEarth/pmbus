// References:
//
// RPPal seems to have a pretty robust implementation to learn from for a better understanding of the buffer format.
// <https://github.com/golemparts/rppal/blob/master/src/i2c/ioctl.rs>
//
// I trust this much less, but it was a simple enough example that matches the SMBus specification at first glance.
// Something strange is happening with the maximum block size however, so I have adjusted the logic according to my own understanding.
// There is one more problem that I forsee in `block_read` regarding how many times (and in how many transactions/operations)
// the command (register) byte is sent.
// <https://github.com/CBJamo/smbus-adapter/blob/main/src/lib.rs>
// That code also has no license, which is a problem, although it is too trivial to hold copyright.

use embedded_hal_async::i2c::{AddressMode, I2c, Operation, SevenBitAddress};

// TODO: Dig deeper, the specification claims that the maximum block size is 255 bits,
// but as we all know, 32 * 8 = 256. Why do other crates assume 32 bytes here and what is the last/first bit supposed to be used for?
pub const SMBUS_MAX_BLOCK_SIZE: usize = 32;

/// Based on System Management Bus (SMBus) Specification Version 3.2.
///
/// <https://smbus.org/specs/SMBus_3_2_20220112.pdf>
///
/// The implementation is not comprehensive, and omits several legacy transactions including read/write N-bytes protocols.
///
/// Also missing is SMBus Host Notify protocol (6.5.9, Pg. 44) which will later be necessary
/// to interrupt the host when a device on the bus has set the `SMBALERT#` bit.
/// This is not critical, as there are often alternative ways to listen for that signal.
///
/// This trait currently does not handle Packet Error Correction.
/// PEC can be implemented later with either a `const` boolean generic parameter,
/// or with a Cargo feature.
pub trait SmBus<A: AddressMode = SevenBitAddress>: I2c<A> {
    /// 6.5.1, Pg. 38
    async fn quick_command(&mut self, address: A, bit: bool) -> Result<(), Self::Error> {
        if bit {
            self.read(address, &mut []).await
        } else {
            self.write(address, &[]).await
        }
    }

    /// 6.5.2, Pg. 38-39
    async fn send_byte(&mut self, address: A, byte: u8) -> Result<(), Self::Error> {
        self.write(address, &[byte]).await
    }

    /// 6.5.3, Pg. 39
    async fn receive_byte(&mut self, address: A) -> Result<u8, Self::Error> {
        let mut buf = [0x00];
        self.read(address, &mut buf).await?;
        Ok(buf[0])
    }

    /// 6.5.4, Pg. 39-40
    async fn write_byte(&mut self, address: A, command: u8, byte: u8) -> Result<(), Self::Error> {
        self.write(address, &[command, byte]).await
    }

    /// 6.5.4, Pg. 39-40
    async fn write_word(&mut self, address: A, command: u8, word: u16) -> Result<(), Self::Error> {
        let word = word.to_le_bytes();
        self.write(address, &[command, word[0], word[1]]).await
    }

    /// 6.5.5, Pg. 40-41
    async fn read_byte(&mut self, address: A, command: u8) -> Result<u8, Self::Error> {
        let mut buf = [0x00];
        self.write_read(address, &[command], &mut buf).await?;
        Ok(buf[0])
    }

    /// 6.5.5, Pg. 40-41
    async fn read_word(&mut self, address: A, command: u8) -> Result<u16, Self::Error> {
        let mut buf = [0x00, 0x00];
        self.write_read(address, &[command], &mut buf).await?;
        Ok(u16::from_le_bytes(buf))
    }

    /// 6.5.6, Pg. 41
    async fn process_call(
        &mut self,
        address: A,
        command: u8,
        word: u16,
    ) -> Result<u16, Self::Error> {
        let word = word.to_le_bytes();
        let mut buf = [0x00, 0x00];
        self.write_read(address, &[command, word[0], word[1]], &mut buf)
            .await?;
        Ok(u16::from_le_bytes(buf))
    }

    /// 6.5.7, Pg. 42
    async fn block_write(
        &mut self,
        address: A,
        command: u8,
        block: &[u8],
    ) -> Result<(), Self::Error> {
        assert!(block.len() <= SMBUS_MAX_BLOCK_SIZE);
        self.transaction(
            address,
            &mut [
                Operation::Write(&[command, block.len() as u8]),
                Operation::Write(block),
            ],
        )
        .await
    }

    /// 6.5.7, Pg. 42
    async fn block_read(&mut self, address: A, command: u8) -> Result<Vec<u8>, Self::Error> {
        // The first byte is reserved for the size of the data written back.
        // Later, when PEC is implemented, we will need an extra byte at the end also.
        let mut buf = Vec::with_capacity(SMBUS_MAX_BLOCK_SIZE + 1);
        // Figure 37 shows the address byte being sent twice, once for the command write and then again for the read operation.
        // Currently, `write_read` will perform this in a single transaction, only sending the address once. This might be broken.
        self.write_read(address, &[command], &mut buf).await?;
        let (len, block) = (buf[0] as usize, &buf[1..]);
        assert!(block.len() == std::cmp::min(len, SMBUS_MAX_BLOCK_SIZE));
        Ok(block.to_vec())
    }

    /// 6.5.8, Pg. 43-44
    async fn block_process_call(
        &mut self,
        address: A,
        command: u8,
        write_block: &[u8],
    ) -> Result<Vec<u8>, Self::Error> {
        assert!(write_block.len() <= SMBUS_MAX_BLOCK_SIZE);
        let mut buf = Vec::with_capacity(SMBUS_MAX_BLOCK_SIZE + 1);
        self.transaction(
            address,
            &mut [
                Operation::Write(&[command, write_block.len() as u8]),
                Operation::Write(write_block),
                Operation::Read(&mut buf),
            ],
        )
        .await?;
        let (len, block) = (buf[0] as usize, &buf[1..]);
        assert!(block.len() == std::cmp::min(len, SMBUS_MAX_BLOCK_SIZE));
        Ok(block.to_vec())
    }
}
