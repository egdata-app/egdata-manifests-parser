//! Tiny helpers for LE primitives and UE-style FStrings.

use byteorder::{ByteOrder, LittleEndian};
use std::io::{self, Read, Seek, SeekFrom};
use uuid::Uuid;

/// Extension methods, implemented for every `Read`.
pub trait ReadExt: Read + Seek {
  fn i32(&mut self) -> io::Result<i32> {
    let bytes = self.read_bytes_tolerant(4)?;
    if bytes.len() < 4 {
      return Err(io::Error::new(
        io::ErrorKind::UnexpectedEof,
        format!("Expected 4 bytes for i32 but got {} bytes", bytes.len()),
      ));
    }
    Ok(LittleEndian::read_i32(&bytes))
  }
  fn u8(&mut self) -> io::Result<u8> {
    let bytes = self.read_bytes_tolerant(1)?;
    if bytes.is_empty() {
      return Err(io::Error::new(
        io::ErrorKind::UnexpectedEof,
        "Expected 1 byte for u8 but got 0 bytes",
      ));
    }
    Ok(bytes[0])
  }
  fn u32(&mut self) -> io::Result<u32> {
    let bytes = self.read_bytes_tolerant(4)?;
    if bytes.len() < 4 {
      return Err(io::Error::new(
        io::ErrorKind::UnexpectedEof,
        format!("Expected 4 bytes for u32 but got {} bytes", bytes.len()),
      ));
    }
    Ok(LittleEndian::read_u32(&bytes))
  }
  fn i64(&mut self) -> io::Result<i64> {
    let bytes = self.read_bytes_tolerant(8)?;
    if bytes.len() < 8 {
      return Err(io::Error::new(
        io::ErrorKind::UnexpectedEof,
        format!("Expected 8 bytes for i64 but got {} bytes", bytes.len()),
      ));
    }
    Ok(LittleEndian::read_i64(&bytes))
  }
  fn u64(&mut self) -> io::Result<u64> {
    let bytes = self.read_bytes_tolerant(8)?;
    if bytes.len() < 8 {
      return Err(io::Error::new(
        io::ErrorKind::UnexpectedEof,
        format!("Expected 8 bytes for u64 but got {} bytes", bytes.len()),
      ));
    }
    Ok(LittleEndian::read_u64(&bytes))
  }

  // Additional primitive type readers
  fn i8(&mut self) -> io::Result<i8> {
    let bytes = self.read_bytes_tolerant(1)?;
    if bytes.is_empty() {
      return Err(io::Error::new(
        io::ErrorKind::UnexpectedEof,
        "Expected 1 byte for i8 but got 0 bytes",
      ));
    }
    Ok(bytes[0] as i8)
  }
  fn i16(&mut self) -> io::Result<i16> {
    let bytes = self.read_bytes_tolerant(2)?;
    if bytes.len() < 2 {
      return Err(io::Error::new(
        io::ErrorKind::UnexpectedEof,
        format!("Expected 2 bytes for i16 but got {} bytes", bytes.len()),
      ));
    }
    Ok(LittleEndian::read_i16(&bytes))
  }
  fn u16(&mut self) -> io::Result<u16> {
    let bytes = self.read_bytes_tolerant(2)?;
    if bytes.len() < 2 {
      return Err(io::Error::new(
        io::ErrorKind::UnexpectedEof,
        format!("Expected 2 bytes for u16 but got {} bytes", bytes.len()),
      ));
    }
    Ok(LittleEndian::read_u16(&bytes))
  }
  fn bool(&mut self) -> io::Result<bool> {
    self.u8().map(|b| b != 0)
  }
  fn f32(&mut self) -> io::Result<f32> {
    let bytes = self.read_bytes_tolerant(4)?;
    if bytes.len() < 4 {
      return Err(io::Error::new(
        io::ErrorKind::UnexpectedEof,
        format!("Expected 4 bytes for f32 but got {} bytes", bytes.len()),
      ));
    }
    Ok(LittleEndian::read_f32(&bytes))
  }
  fn f64(&mut self) -> io::Result<f64> {
    let bytes = self.read_bytes_tolerant(8)?;
    if bytes.len() < 8 {
      return Err(io::Error::new(
        io::ErrorKind::UnexpectedEof,
        format!("Expected 8 bytes for f64 but got {} bytes", bytes.len()),
      ));
    }
    Ok(LittleEndian::read_f64(&bytes))
  }

  /// Read exactly n bytes
  fn read_bytes(&mut self, count: usize) -> io::Result<Vec<u8>> {
    if count == 0 {
      return Ok(Vec::new());
    }
    // Use tolerant reading to handle EOF gracefully
    self.read_bytes_tolerant(count)
  }

  /// Read up to n bytes, returning whatever is available (like .NET BinaryReader.ReadBytes)
  /// This method handles partial reads gracefully and doesn't fail on EOF
  fn read_bytes_available(&mut self, count: usize) -> io::Result<Vec<u8>> {
    if count == 0 {
      return Ok(Vec::new());
    }
    let mut buf = vec![0u8; count];
    let bytes_read = self.read(&mut buf)?;
    buf.truncate(bytes_read);
    Ok(buf)
  }

  /// Read exactly n bytes, but handle EOF gracefully by returning available bytes
  fn read_bytes_tolerant(&mut self, count: usize) -> io::Result<Vec<u8>> {
    if count == 0 {
      return Ok(Vec::new());
    }
    let mut buf = vec![0u8; count];
    let mut total_read = 0;
    
    while total_read < count {
      match self.read(&mut buf[total_read..]) {
        Ok(0) => break, // EOF reached
        Ok(n) => total_read += n,
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
        Err(e) => return Err(e),
      }
    }
    
    buf.truncate(total_read);
    Ok(buf)
  }

  /// Read a GUID (UUID) stored as 4 uint32 segments in Big Endian
  fn guid(&mut self) -> io::Result<Uuid> {
    let mut data = [0u32; 4];
    for i in 0..4 {
      let bytes = self.read_bytes_tolerant(4)?;
      if bytes.len() < 4 {
        return Err(io::Error::new(
          io::ErrorKind::UnexpectedEof,
          format!("Expected 4 bytes for GUID segment {} but got {} bytes", i, bytes.len()),
        ));
      }
      data[i] = byteorder::BigEndian::read_u32(&bytes);
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

  /// Unreal's FString (signed 32-bit length; negative = UTF-16 LE)
  fn fstring(&mut self) -> io::Result<String> {
    let len = self.i32()?;
    if len == 0 {
      return Ok(String::new());
    }

    if len < 0 {
      let chars_with_null = (-len) as usize;
      let bytes_to_read = chars_with_null.checked_mul(2).ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "wide string length overflow")
      })?;

      const MAX_REASONABLE_STRING_LENGTH_BYTES: usize = 1024 * 1024 * 1024;
      if bytes_to_read > MAX_REASONABLE_STRING_LENGTH_BYTES {
        return Err(io::Error::new(
          io::ErrorKind::InvalidData,
          format!(
            "String length ({}) exceeds maximum allowed size of {} bytes",
            bytes_to_read, MAX_REASONABLE_STRING_LENGTH_BYTES
          ),
        ));
      }

      let buf = self.read_bytes_tolerant(bytes_to_read)?;
      if buf.len() < bytes_to_read {
        return Err(io::Error::new(
          io::ErrorKind::UnexpectedEof,
          format!("Expected {} bytes for string but got {} bytes", bytes_to_read, buf.len()),
        ));
      }

      let mut utf16: Vec<u16> = Vec::with_capacity(chars_with_null);
      let mut i = 0usize;
      while i + 1 < buf.len() {
        let v = LittleEndian::read_u16(&buf[i..i + 2]);
        utf16.push(v);
        i += 2;
      }

      let slice_len = utf16.len().saturating_sub(1);
      let s = String::from_utf16_lossy(&utf16[..slice_len]);
      return Ok(s);
    } else {
      let len_u = len as usize;
      const MAX_REASONABLE_STRING_LENGTH_BYTES: usize = 1024 * 1024 * 1024;
      if len_u > MAX_REASONABLE_STRING_LENGTH_BYTES {
        return Err(io::Error::new(
          io::ErrorKind::InvalidData,
          format!(
            "String length ({}) exceeds maximum allowed size of {} bytes",
            len_u, MAX_REASONABLE_STRING_LENGTH_BYTES
          ),
        ));
      }

      let buf = self.read_bytes_tolerant(len_u)?;
      if buf.len() < len_u {
        return Err(io::Error::new(
          io::ErrorKind::UnexpectedEof,
          format!("Expected {} bytes for string but got {} bytes", len_u, buf.len()),
        ));
      }

      let slice_len = if !buf.is_empty() && *buf.last().unwrap() == 0 { buf.len() - 1 } else { buf.len() };
      let s = String::from_utf8_lossy(&buf[..slice_len]).to_string();
      return Ok(s);
    }
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
