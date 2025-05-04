use std::io::{self, Read};

/// An unsigned 8-bit integer that cannot assume the two extreme values 0x00 and 0xFF.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct NonExtremeU8(u8);
impl NonExtremeU8 {
    pub const fn try_from_u8(value: u8) -> Result<Self, u8> {
        if value == 0x00 || value == 0xFF {
            Err(value)
        } else {
            Ok(Self(value))
        }
    }

    pub const fn as_u8(&self) -> u8 {
        self.0
    }
}
impl TryFrom<u8> for NonExtremeU8 {
    type Error = u8;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::try_from_u8(value)
    }
}
impl From<NonExtremeU8> for u8 {
    fn from(value: NonExtremeU8) -> Self {
        value.as_u8()
    }
}


/// A piece of data read from a JPEG file.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum JpegDataPiece {
    /// A marker that holds a value and encodes its length.
    MarkerWithLength {
        /// Number of additional 0xFF bytes.
        ///
        /// Markers start with an 0xFF byte followed by a non-0xFF, non-0x00 byte encoding the
        /// marker type. It is however allowed to encode a sequence of multiple 0xFF bytes instead.
        additional_ff_count: usize,

        /// The type of the marker itself.
        marker_type: NonExtremeU8,

        /// The value of the marker.
        value: Vec<u8>,
    },

    /// A marker that does not hold a value.
    EmptyMarker {
        /// Number of additional 0xFF bytes.
        ///
        /// Markers start with an 0xFF byte followed by a non-0xFF, non-0x00 byte encoding the
        /// marker type. It is however allowed to encode a sequence of multiple 0xFF bytes instead.
        additional_ff_count: usize,

        /// The type of the marker itself.
        marker_type: NonExtremeU8,
    },

    /// An 0xFF value that has been byte-stuffed into the entropy-coded data.
    ///
    /// This is encoded as a sequence of at least one 0xFF value followed by a 0x00 value.
    ByteStuffedFF {
        /// Number of additional 0xFF bytes.
        ///
        /// Markers start with an 0xFF byte followed by a non-0xFF, non-0x00 byte encoding the
        /// marker type. It is however allowed to encode a sequence of multiple 0xFF bytes instead.
        additional_ff_count: usize,
    },

    /// Data that is not a marker.
    EntropyCodedData {
        data: Vec<u8>,
    },
}

/// Wrapper that makes readers peekable.
struct PeekWrapper<'r, R: Read> {
    reader: &'r mut R,
    holding_cell: Option<u8>,
}
impl<'r, R: Read> PeekWrapper<'r, R> {
    pub fn new(reader: &'r mut R) -> Self {
        Self {
            reader,
            holding_cell: None,
        }
    }

    pub fn peek(&mut self) -> Result<Option<u8>, io::Error> {
        if let Some(b) = self.holding_cell {
            return Ok(Some(b));
        }

        let mut buf = [0u8];
        let bytes_read = self.reader.read(&mut buf)?;
        if bytes_read == 0 {
            Ok(None)
        } else {
            self.holding_cell = Some(buf[0]);
            Ok(Some(buf[0]))
        }
    }

    pub fn read_byte(&mut self) -> Result<Option<u8>, io::Error> {
        match self.peek() {
            Ok(Some(b)) => {
                // forget the held value again
                self.holding_cell = None;
                Ok(Some(b))
            },
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
impl<'r, R: Read> Read for PeekWrapper<'r, R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.len() == 0 {
            return Ok(0);
        }

        if let Some(held) = self.holding_cell {
            self.holding_cell = None;
            buf[0] = held;
            return Ok(1);
        }

        self.reader.read(buf)
    }
}

pub fn read_next<R: Read>(reader: &mut R) -> Result<JpegDataPiece, io::Error> {
    let mut peek_reader = PeekWrapper::new(reader);

    // read one byte
    let byte = peek_reader.read_byte()?
        .ok_or_else(|| io::ErrorKind::UnexpectedEof)?;
    if byte == 0xFF {
        // marker
        let mut additional_ff_count = 0;
        let marker_byte = loop {
            let next_byte = peek_reader.read_byte()?
                .ok_or_else(|| io::ErrorKind::UnexpectedEof)?;
            if next_byte == 0xFF {
                additional_ff_count += 1;
            } else {
                break next_byte;
            }
        };
        match marker_byte {
            0x00 => {
                // stuffed byte
                return Ok(JpegDataPiece::ByteStuffedFF {
                    additional_ff_count,
                });
            },
            0x01|0xD0..=0xD7|0xD8|0xD9 => {
                // data-less marker
                let marker_type = NonExtremeU8::try_from_u8(marker_byte).unwrap();
                return Ok(JpegDataPiece::EmptyMarker {
                    additional_ff_count,
                    marker_type,
                });
            },
            0xFF => unreachable!(),
            other => {
                // marker with length in the next two bytes
                let marker_type = NonExtremeU8::try_from_u8(other).unwrap();

                let mut length_buf = [0u8; 2];
                reader.read_exact(&mut length_buf)?;
                let mut length: usize = u16::from_be_bytes(length_buf).into();

                // the length must include the length value itself
                if length < 2 {
                    return Err(io::ErrorKind::InvalidData.into());
                }
                length -= 2;

                let mut data_buf = vec![0u8; length];
                reader.read_exact(&mut data_buf)?;

                return Ok(JpegDataPiece::MarkerWithLength {
                    additional_ff_count,
                    marker_type,
                    value: data_buf,
                });
            },
        }
    } else {
        // entropy-coded bytes
        let mut buf = vec![byte];
        loop {
            match peek_reader.peek() {
                Ok(Some(0xFF)) => {
                    // marker starts; leave it for the next go-around
                    return Ok(JpegDataPiece::EntropyCodedData { data: buf });
                },
                Ok(Some(b)) => {
                    // another entropy-coded byte
                    buf.push(b);

                    // consume it
                    let _ = peek_reader.read_byte();
                },
                Ok(None) => {
                    // EOF
                    return Ok(JpegDataPiece::EntropyCodedData { data: buf });
                },
                Err(e) => return Err(e),
            }
        }
    }
}
