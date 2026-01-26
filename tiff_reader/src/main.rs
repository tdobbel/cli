use anyhow::Result;
use memmap2::Mmap;
use std::env;
use std::fs::File;

pub enum Endianness {
    Little,
    Big,
}

pub trait ByteOrder {
    fn read_u16(buf: &[u8]) -> u16;
    fn read_u32(buf: &[u8]) -> u32;
}

pub enum LittleEndian {}
impl ByteOrder for LittleEndian {
    fn read_u16(buf: &[u8]) -> u16 {
        u16::from_le_bytes(buf[..2].try_into().unwrap())
    }

    fn read_u32(buf: &[u8]) -> u32 {
        u32::from_le_bytes(buf[..4].try_into().unwrap())
    }
}

pub enum BigEndian {}
impl ByteOrder for BigEndian {
    fn read_u16(buf: &[u8]) -> u16 {
        u16::from_be_bytes(buf[..2].try_into().unwrap())
    }

    fn read_u32(buf: &[u8]) -> u32 {
        u32::from_be_bytes(buf[..4].try_into().unwrap())
    }
}

#[derive(Default)]
struct IFD {
    image_width: u32,
    image_length: u32,
    bits_per_sample: u16,
    compression: u16,
    photometric_interpretation: u16,
    samples_per_pixel: u16,
    strip_offsets: Vec<u32>,
    rows_per_strip: u32,
    planar_configuration: u16,
    sample_format: u16,
    strip_byte_counts: Vec<u32>,
    projection: Option<String>,
    model_tie_points: Option<Vec<f32>>,
    model_pixel_scale_tag: Option<Vec<f32>>,
}

struct IFDEntry {
    tag: u16,
    field_type: u16,
    count: u32,
    value_offset: u32,
}

struct TiffData {
    ifd: IFD,
    x: Vec<f64>,
    y: Vec<f64>,
    data: Option<Vec<f32>>,
}

struct TiffReader {
    offset: usize,
    data: Mmap,
    endianness: Endianness,
}

impl TiffReader {
    fn new(map: Mmap, endianness: Endianness) -> Self {
        Self {
            offset: 0,
            data: map,
            endianness,
        }
    }

    fn set_offset(&mut self, offset: usize) {
        self.offset = offset;
    }

    fn read_u16(&mut self) -> u16 {
        let value = match self.endianness {
            Endianness::Little => LittleEndian::read_u16(&self.data[self.offset..]),
            Endianness::Big => BigEndian::read_u16(&self.data[self.offset..]),
        };
        self.offset += 2;
        value
    }

    fn read_u32(&mut self) -> u32 {
        let value = match self.endianness {
            Endianness::Little => LittleEndian::read_u32(&self.data[self.offset..]),
            Endianness::Big => BigEndian::read_u32(&self.data[self.offset..]),
        };
        self.offset += 4;
        value
    }

    fn read_ifd_entry(&mut self) -> IFDEntry {
        IFDEntry {
            tag: self.read_u16(),
            field_type: self.read_u16(),
            count: self.read_u32(),
            value_offset: self.read_u32(),
        }
    }
}

fn main() -> Result<()> {
    let filename = env::args().nth(1).expect("Missing input file");
    let file = File::open(filename)?;
    let map = unsafe { Mmap::map(&file)? };
    let first_two_bytes = &map[..2];
    let mut tiff_reader = match first_two_bytes {
        b"II" => TiffReader::new(map, Endianness::Little),
        b"MM" => TiffReader::new(map, Endianness::Big),
        _ => panic!("First 2 bytes not recognized"),
    };
    tiff_reader.set_offset(4);
    assert!(tiff_reader.read_u16() == 42);
    Ok(())
}
