//! Tiny helpers for LE primitives and UE-style FStrings.

use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use std::io::{self, Read, Seek, SeekFrom};
use uuid::Uuid;

/// Extension methods, implemented for every `Read`.
pub trait ReadExt: Read + Seek {
  fn i32(&mut self) -> io::Result<i32> {
    self.read_i32::<LittleEndian>()
  }
  fn u8(&mut self) -> io::Result<u8> {
    self.read_u8()
  }
  fn u32(&mut self) -> io::Result<u32> {
    self.read_u32::<LittleEndian>()
  }
  fn i64(&mut self) -> io::Result<i64> {
    self.read_i64::<LittleEndian>()
  }
  fn u64(&mut self) -> io::Result<u64> {
    self.read_u64::<LittleEndian>()
  }

  // Additional primitive type readers
  fn i8(&mut self) -> io::Result<i8> {
    self.read_i8()
  }
  fn i16(&mut self) -> io::Result<i16> {
    self.read_i16::<LittleEndian>()
  }
  fn u16(&mut self) -> io::Result<u16> {
    self.read_u16::<LittleEndian>()
  }
  fn bool(&mut self) -> io::Result<bool> {
    self.read_u8().map(|b| b != 0)
  }
  fn f32(&mut self) -> io::Result<f32> {
    self.read_f32::<LittleEndian>()
  }
  fn f64(&mut self) -> io::Result<f64> {
    self.read_f64::<LittleEndian>()
  }

  /// Read exactly n bytes
  fn read_bytes(&mut self, count: usize) -> io::Result<Vec<u8>> {
    if count == 0 {
      return Ok(Vec::new());
    }
    let mut buf = vec![0u8; count];
    self.read_exact(&mut buf)?;
    Ok(buf)
  }

  /// Read a GUID (UUID) stored as 4 uint32 segments in Big Endian
  fn guid(&mut self) -> io::Result<Uuid> {
    let mut data = [0u32; 4];
    for i in 0..4 {
      data[i] = self.read_u32::<byteorder::BigEndian>()?;
    }
    let mut guid_bytes = [0u8; 16];
    for i in 0..4 {
      LittleEndian::write_u32(&mut guid_bytes[i * 4..(i + 1) * 4], data[i]);
    }
    Ok(Uuid::from_bytes(guid_bytes))
  }

  /// Peek n bytes without advancing the reader
  fn peek(&mut self, n: usize) -> io::Result<Vec<u8>> {
    let pos = self.stream_position()?;
    let bytes = self.read_bytes(n)?;
    self.seek(SeekFrom::Start(pos))?;
    Ok(bytes)
  }

  /// Unreal's FString (32-bit length, optionally null-terminated)
  fn fstring(&mut self) -> io::Result<String> {
    let len = self.u32()?;
    if len == 0 {
      return Ok(String::new());
    }

    // Add reasonable size limit
    const MAX_REASONABLE_STRING_LENGTH: u32 = 1024 * 1024 * 1024; // 1GB max string length
    if len > MAX_REASONABLE_STRING_LENGTH {
      return Err(io::Error::new(
        io::ErrorKind::InvalidData,
        format!(
          "String length ({}) exceeds maximum allowed size of {} bytes",
          len, MAX_REASONABLE_STRING_LENGTH
        ),
      ));
    }

    let mut buf = vec![0u8; len as usize];
    self.read_exact(&mut buf)?;

    // Use the length field directly to determine string length
    // This handles both null-terminated and non-null-terminated strings
    Ok(String::from_utf8_lossy(&buf).to_string())
  }

  fn fstring_array(&mut self) -> io::Result<Vec<String>> {
    let len = self.u32()? as usize;
    let mut strings = Vec::with_capacity(len);
    for _ in 0..len {
      strings.push(self.fstring()?);
    }
    Ok(strings)
  }

  fn skip(&mut self, bytes: u64) -> io::Result<()> {
    self.seek(SeekFrom::Current(bytes as i64))?;
    Ok(())
  }
}

impl<T: Read + Seek + ?Sized> ReadExt for T {}
