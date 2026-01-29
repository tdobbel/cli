use anyhow::{Result, anyhow};
use memmap2::Mmap;
use std::convert::TryFrom;
use std::env;
use std::fs::File;

pub enum TiffError {
    InvalidDataType,
}

pub enum Endianness {
    Little,
    Big,
}

pub enum TiffDataType {
    Short = 3,
    Long = 4,
    Float = 11,
    Double = 12,
}

impl TryFrom<u16> for TiffDataType {
    type Error = ();

    fn try_from(num: u16) -> Result<Self, Self::Error> {
        match num {
            x if x == TiffDataType::Long as u16 => Ok(TiffDataType::Long),
            x if x == TiffDataType::Short as u16 => Ok(TiffDataType::Short),
            x if x == TiffDataType::Float as u16 => Ok(TiffDataType::Float),
            x if x == TiffDataType::Double as u16 => Ok(TiffDataType::Double),
            _ => Err(()),
        }
    }
}

pub trait ByteOrder {
    fn read_u16(buf: &[u8]) -> u16;
    fn read_u32(buf: &[u8]) -> u32;
    fn read_u64(buf: &[u8]) -> u64;
    fn read_f32(buf: &[u8]) -> f32;
    fn read_f64(buf: &[u8]) -> f64;
}

pub enum LittleEndian {}
impl ByteOrder for LittleEndian {
    fn read_u16(buf: &[u8]) -> u16 {
        u16::from_le_bytes(buf[..2].try_into().unwrap())
    }

    fn read_u32(buf: &[u8]) -> u32 {
        u32::from_le_bytes(buf[..4].try_into().unwrap())
    }

    fn read_u64(buf: &[u8]) -> u64 {
        u64::from_le_bytes(buf[..8].try_into().unwrap())
    }

    fn read_f32(buf: &[u8]) -> f32 {
        f32::from_le_bytes(buf[..4].try_into().unwrap())
    }

    fn read_f64(buf: &[u8]) -> f64 {
        f64::from_le_bytes(buf[..8].try_into().unwrap())
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

    fn read_u64(buf: &[u8]) -> u64 {
        u64::from_be_bytes(buf[..8].try_into().unwrap())
    }

    fn read_f32(buf: &[u8]) -> f32 {
        f32::from_be_bytes(buf[..4].try_into().unwrap())
    }

    fn read_f64(buf: &[u8]) -> f64 {
        f64::from_be_bytes(buf[..8].try_into().unwrap())
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
    projection: String,
    model_tie_points: Vec<f64>,
    model_pixel_scale_tag: Vec<f64>,
}

#[derive(Debug)]
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

    fn set_offset(&mut self, offset: u32) {
        self.offset = offset as usize;
    }

    fn read_u32_vector(&mut self, entry: &IFDEntry) -> Result<Vec<u32>, TiffError> {
        let mut vec = Vec::new();
        let current = self.offset;
        self.set_offset(entry.value_offset);
        for _ in 0..entry.count {
            let num = match entry.field_type.try_into() {
                Ok(TiffDataType::Short) => self.read_u16() as u32,
                Ok(TiffDataType::Long) => self.read_u32(),
                _ => return Err(TiffError::InvalidDataType),
            };
            vec.push(num);
        }
        self.offset = current;
        Ok(vec)
    }

    fn read_f64_vector(&mut self, entry: &IFDEntry) -> Result<Vec<f64>, TiffError> {
        let mut vec = Vec::new();
        let current = self.offset;
        self.set_offset(entry.value_offset);
        for _ in 0..entry.count {
            let num = match entry.field_type.try_into() {
                Ok(TiffDataType::Float) => self.read_f32() as f64,
                Ok(TiffDataType::Double) => self.read_f64(),
                _ => return Err(TiffError::InvalidDataType),
            };
            vec.push(num);
        }
        self.offset = current;
        Ok(vec)
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

    fn read_f32(&mut self) -> f32 {
        let value = match self.endianness {
            Endianness::Little => LittleEndian::read_f32(&self.data[self.offset..]),
            Endianness::Big => BigEndian::read_f32(&self.data[self.offset..]),
        };
        self.offset += 4;
        value
    }

    fn read_f64(&mut self) -> f64 {
        let value = match self.endianness {
            Endianness::Little => LittleEndian::read_f64(&self.data[self.offset..]),
            Endianness::Big => BigEndian::read_f64(&self.data[self.offset..]),
        };
        self.offset += 8;
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

    fn set_ifd_entry(&mut self, ifd: &mut IFD) -> Result<(), TiffError> {
        let entry = self.read_ifd_entry();
        match entry.tag {
            256 => ifd.image_width = entry.value_offset,
            257 => ifd.image_length = entry.value_offset,
            258 => ifd.bits_per_sample = entry.value_offset as u16,
            259 => ifd.compression = entry.value_offset as u16,
            262 => ifd.photometric_interpretation = entry.value_offset as u16,
            273 => ifd.strip_offsets = self.read_u32_vector(&entry)?,
            277 => ifd.samples_per_pixel = entry.value_offset as u16,
            278 => ifd.rows_per_strip = entry.value_offset,
            279 => ifd.strip_byte_counts = self.read_u32_vector(&entry)?,
            284 => ifd.planar_configuration = entry.value_offset as u16,
            339 => ifd.sample_format = entry.value_offset as u16,
            33922 => ifd.model_tie_points = self.read_f64_vector(&entry)?,
            33550 => ifd.model_pixel_scale_tag = self.read_f64_vector(&entry)?,
            34735 => {}
            34737 => {
                let start = entry.value_offset as usize;
                let stop = start + entry.count as usize;
                ifd.projection = String::from_utf8_lossy(&self.data[start..stop]).to_string();
            }
            _ => print!("Unknown IFD entry {:?}", entry),
        };
        Ok(())
    }

    fn read_tiff(&mut self) {
        self.offset = 2;
        assert!(self.read_u16() == 42);
        self.offset = self.read_u32() as usize;
        let n_entry = self.read_u16();
        let mut ifd = IFD::default();
        for _ in 0..n_entry as usize {
            let _ = self.set_ifd_entry(&mut ifd);
        }
        println!("{:?}", ifd.model_tie_points);
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
    tiff_reader.read_tiff();
    Ok(())
}
