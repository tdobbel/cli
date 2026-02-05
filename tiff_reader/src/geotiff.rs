use anyhow::{Result, anyhow};
use memmap2::Mmap;
use num_traits::{Num, NumCast};
use std::convert::{TryFrom, TryInto};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum TiffError {
    InvalidDataType,
    InvalidTransformation,
    NoDataLoaded,
    BadMagicNumber,
    UndefinedSampleFormat,
}

impl fmt::Display for TiffError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTransformation => write!(f, "Tiff error: Invalid transformation data"),
            Self::InvalidDataType => write!(f, "Tiff error: Invalid data type"),
            Self::NoDataLoaded => write!(f, "Tiff error: Tif data must be loaded"),
            Self::BadMagicNumber => write!(f, "Bad magic number (expected 42 or 43 for Big Tiff)"),
            Self::UndefinedSampleFormat => write!(f, "Undefined sample format"),
        }
    }
}

impl Error for TiffError {}

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

impl_from_bytes!(u16, u32, u64, i16, f32, f64);

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

pub enum TiffDataArray {
    UnsignedInt(Vec<u16>),
    SignedInt(Vec<i16>),
    Float(Vec<f32>),
}

impl TiffDataArray {
    pub fn push_reader(&mut self, reader: &mut TiffReader) {
        match self {
            Self::UnsignedInt(v) => v.push(reader.read_scalar()),
            Self::SignedInt(v) => v.push(reader.read_scalar()),
            Self::Float(v) => v.push(reader.read_scalar()),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::UnsignedInt(v) => v.len(),
            Self::SignedInt(v) => v.len(),
            Self::Float(v) => v.len(),
        }
    }

    pub fn get(&self, index: usize) -> TiffSample {
        match self {
            Self::UnsignedInt(v) => TiffSample::U16(v[index]),
            Self::SignedInt(v) => TiffSample::I16(v[index]),
            Self::Float(v) => TiffSample::F32(v[index]),
        }
    }
}

pub enum TiffSample {
    U16(u16),
    I16(i16),
    F32(f32),
}

impl fmt::Display for TiffSample {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::U16(x) => write!(f, "{x}"),
            Self::I16(x) => write!(f, "{x}"),
            Self::F32(x) => write!(f, "{x}"),
        }
    }
}

#[derive(Default)]
struct TiffIfd {
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
    model_transformation_tag: Option<[f64; 16]>,
    geo_double_params_tag: Option<Vec<f64>>,
}

impl TiffIfd {
    fn generate_coordinates(&self) -> Result<(Vec<f64>, Vec<f64>), TiffError> {
        let nx = self.image_width as usize;
        let ny = self.image_length as usize;
        let mut x = vec![0.0; nx];
        let mut y = vec![0.0; ny];
        if let Some(trans) = self.model_transformation_tag {
            if trans[1].abs() > f64::EPSILON || trans[4].abs() > f64::EPSILON {
                return Err(TiffError::InvalidTransformation);
            }
            for i in 0..nx {
                x[i] = trans[3] + trans[0] * (i as f64);
            }
            for i in 0..ny {
                y[i] = trans[7] + trans[5] * (i as f64);
            }
            return Ok((x, y));
        }
        if self.model_tie_points.is_some() && self.model_pixel_scale_tag.is_some() {
            // Assume upper left corner is provided
            let tie_points = self.model_tie_points.as_ref().unwrap();
            let pixel_scale = self.model_pixel_scale_tag.as_ref().unwrap();
            if tie_points.len() != 6 {
                eprintln!("model_tie_points has unexpected size");
                return Err(TiffError::InvalidTransformation);
            }
            let i = tie_points[0] as usize;
            let j = tie_points[1] as usize;
            if i != 0 || j != 0 {
                eprintln!("Provided tie point was not (0, 0)");
                return Err(TiffError::InvalidTransformation);
            }
            x[0] = tie_points[3];
            for i in 1..nx {
                x[i] = x[i - 1] + pixel_scale[0];
            }
            y[0] = tie_points[4];
            for i in 1..ny {
                y[i] = y[i - 1] - pixel_scale[1];
            }
            return Ok((x, y));
        }
        Err(TiffError::InvalidTransformation)
    }
}

#[derive(Debug)]
struct IfdEntry {
    tag: u16,
    field_type: u16,
    count: u32,
    value_offset: u32,
}

pub struct TiffDataset {
    ifd: TiffIfd,
    x: Vec<f64>,
    y: Vec<f64>,
    data: Option<TiffDataArray>,
}

impl TiffDataset {
    fn from_ifd(ifd: TiffIfd) -> Result<Self, TiffError> {
        let (x, y) = ifd.generate_coordinates()?;
        Ok(Self {
            ifd,
            x,
            y,
            data: None,
        })
    }

    pub fn get_extent(&self) -> (f64, f64, f64, f64) {
        let x0 = *self.x.first().unwrap();
        let x1 = *self.x.last().unwrap();
        let y0 = *self.y.first().unwrap();
        let y1 = *self.y.last().unwrap();
        (x0.min(x1), x0.max(x1), y0.min(y1), y0.max(y1))
    }

    pub fn load_data(&mut self, reader: &mut TiffReader) -> Result<()> {
        if self.data.is_some() {
            return Ok(());
        }
        let nx = self.ifd.image_width as usize;
        let ny = self.ifd.image_length as usize;
        let mut data = match self.ifd.sample_format {
            1 => TiffDataArray::UnsignedInt(Vec::with_capacity(nx * ny)),
            2 => TiffDataArray::SignedInt(Vec::with_capacity(nx * ny)),
            3 => TiffDataArray::Float(Vec::with_capacity(nx * ny)),
            _ => return Err(TiffError::UndefinedSampleFormat.into()),
        };
        let bytesize = (self.ifd.bits_per_sample / 8) as usize;
        for (i, offset) in self.ifd.strip_offsets.iter().enumerate() {
            let n_entry = self.ifd.strip_byte_counts[i] as usize / bytesize;
            reader.set_offset(*offset);
            for _ in 0..n_entry {
                data.push_reader(reader);
            }
        }
        assert_eq!(data.len(), nx * ny);
        self.data = Some(data);
        Ok(())
    }

    pub fn get(&self, i: usize, j: usize) -> Result<TiffSample> {
        match self.data.as_ref() {
            Some(data) => Ok(data.get(i * (self.ifd.image_width as usize) + j)),
            None => Err(TiffError::NoDataLoaded.into()),
        }
    }
}

pub struct TiffReader {
    offset: usize,
    data: Mmap,
    endianness: Endianness,
}

impl TiffReader {
    pub fn new(map: Mmap) -> Result<Self> {
        let first_two_bytes = &map[..2];
        let endianness = match first_two_bytes {
            b"II" => Endianness::Little,
            b"MM" => Endianness::Big,
            _ => return Err(anyhow!("First 2 bytes not recognized")),
        };
        Ok(Self {
            offset: 0,
            data: map,
            endianness,
        })
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

    fn read_vector<T: Num + NumCast>(&mut self, entry: &IfdEntry) -> Result<Vec<T>, TiffError> {
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

    fn read_ifd_entry(&mut self) -> IfdEntry {
        IfdEntry {
            tag: self.read_scalar(),
            field_type: self.read_scalar(),
            count: self.read_scalar(),
            value_offset: self.read_scalar(),
        }
    }

    fn set_ifd_entry(&mut self, ifd: &mut TiffIfd) -> Result<()> {
        let entry = self.read_ifd_entry();
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
            34264 => {
                let vec = self.read_vector::<f64>(&entry)?;
                ifd.model_transformation_tag = Some(vec[..16].try_into()?);
            }
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

    pub fn read_tiff(&mut self) -> Result<TiffDataset> {
        self.offset = 2;
        let magic: u16 = self.read_scalar();
        if magic == 43 {
            return Err(anyhow!("Big Tiff reader not implemented yet..."));
        } else if magic != 42 {
            return Err(TiffError::BadMagicNumber.into());
        }
        self.offset = self.read_scalar::<u32>() as usize;
        let n_entry: u16 = self.read_scalar();
        let mut ifd = TiffIfd::default();
        for _ in 0..n_entry as usize {
            let _ = self.set_ifd_entry(&mut ifd);
        }
        if ifd.sample_format == 4 {
            return Err(TiffError::UndefinedSampleFormat.into());
        }
        if self.read_scalar::<u32>() != 0 {
            return Err(anyhow!("More than 1 IFD found in file!"));
        }
        Ok(TiffDataset::from_ifd(ifd)?)
    }
}
