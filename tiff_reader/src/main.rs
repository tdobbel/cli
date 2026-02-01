use anyhow::Result;
use memmap2::Mmap;
use num_traits::{Num, NumCast};
use std::convert::{TryFrom, TryInto};
use std::env;
use std::fs::File;

pub enum TiffError {
    InvalidDataType,
}

pub enum Endianness {
    Little,
    Big,
}

trait FromBytes: Sized {
    const SIZE: usize;
    fn from_le_bytes(bytes: &[u8]) -> Option<Self>;
    fn from_be_bytes(bytes: &[u8]) -> Option<Self>;
}

macro_rules! impl_from_bytes {
    ($($t:ty),* $(,)?) => {
        $(
            impl FromBytes for $t {
                const SIZE: usize = std::mem::size_of::<$t>();

                fn from_le_bytes(bytes: &[u8]) -> Option<Self> {
                    let array: [u8; Self::SIZE] = bytes
                        .get(..Self::SIZE)?
                        .try_into()
                        .ok()?;

                    Some(<$t>::from_le_bytes(array))
                }

                fn from_be_bytes(bytes: &[u8]) -> Option<Self> {
                    let array: [u8; Self::SIZE] = bytes
                        .get(..Self::SIZE)?
                        .try_into()
                        .ok()?;

                    Some(<$t>::from_be_bytes(array))
                }
            }
        )*
    };
}

impl_from_bytes!(u16, u32, u64, f32, f64);

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
    model_tie_points: Option<Vec<f64>>,
    model_pixel_scale_tag: Option<Vec<f64>>,
    model_transformation_tag: Option<Vec<f64>>,
    geo_double_params_tag: Option<Vec<f64>>,
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

    fn read_scalar<T: FromBytes>(&mut self) -> T {
        let shift = size_of::<T>();
        let slice = &self.data[self.offset..self.offset + shift];
        let value = match self.endianness {
            Endianness::Little => T::from_le_bytes(slice).unwrap(),
            Endianness::Big => T::from_be_bytes(slice).unwrap(),
        };
        self.offset += shift;
        value
    }

    fn read_vector<T: Num + NumCast>(&mut self, entry: &IFDEntry) -> Result<Vec<T>, TiffError> {
        let mut vec = Vec::new();
        let current = self.offset;
        self.set_offset(entry.value_offset);
        for _ in 0..entry.count {
            let num = match entry.field_type.try_into() {
                Ok(TiffDataType::Float) => NumCast::from(self.read_scalar::<f32>()).unwrap(),
                Ok(TiffDataType::Double) => NumCast::from(self.read_scalar::<f64>()).unwrap(),
                Ok(TiffDataType::Short) => NumCast::from(self.read_scalar::<u16>()).unwrap(),
                Ok(TiffDataType::Long) => NumCast::from(self.read_scalar::<u32>()).unwrap(),
                _ => return Err(TiffError::InvalidDataType),
            };
            vec.push(num);
        }
        self.offset = current;
        Ok(vec)
    }

    fn read_ifd_entry(&mut self) -> IFDEntry {
        IFDEntry {
            tag: self.read_scalar(),
            field_type: self.read_scalar(),
            count: self.read_scalar(),
            value_offset: self.read_scalar(),
        }
    }

    fn set_ifd_entry(&mut self, ifd: &mut IFD) -> Result<(), TiffError> {
        let entry = self.read_ifd_entry();
        println!("Current tag: {}", entry.tag);
        match entry.tag {
            256 => ifd.image_width = entry.value_offset,
            257 => ifd.image_length = entry.value_offset,
            258 => ifd.bits_per_sample = entry.value_offset as u16,
            259 => ifd.compression = entry.value_offset as u16,
            262 => ifd.photometric_interpretation = entry.value_offset as u16,
            273 => ifd.strip_offsets = self.read_vector(&entry)?,
            277 => ifd.samples_per_pixel = entry.value_offset as u16,
            278 => ifd.rows_per_strip = entry.value_offset,
            279 => ifd.strip_byte_counts = self.read_vector(&entry)?,
            284 => ifd.planar_configuration = entry.value_offset as u16,
            339 => ifd.sample_format = entry.value_offset as u16,
            33922 => ifd.model_tie_points = Some(self.read_vector(&entry)?),
            33550 => ifd.model_pixel_scale_tag = Some(self.read_vector(&entry)?),
            34264 => ifd.model_transformation_tag = Some(self.read_vector(&entry)?),
            34735 => {}
            34736 => ifd.geo_double_params_tag = Some(self.read_vector(&entry)?),
            34737 => {
                let start = entry.value_offset as usize;
                let stop = start + entry.count as usize;
                ifd.projection = String::from_utf8_lossy(&self.data[start..stop]).to_string();
            }
            _ => println!("Unknown IFD entry {:?}", entry),
        };
        Ok(())
    }

    fn read_tiff(&mut self) {
        self.offset = 2;
        assert!(self.read_scalar::<u16>() == 42);
        self.offset = self.read_scalar::<u32>() as usize;
        let n_entry: u16 = self.read_scalar();
        let mut ifd = IFD::default();
        for _ in 0..n_entry as usize {
            let _ = self.set_ifd_entry(&mut ifd);
        }
        println!("{} {}", ifd.image_width, ifd.image_length);
        println!("{:?}", ifd.model_tie_points);
        println!("{:?}", ifd.geo_double_params_tag);
        if let Some(trans) = ifd.model_transformation_tag {
            println!("{:?}", trans)
        }
        println!("Projection: {}", ifd.projection);
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
