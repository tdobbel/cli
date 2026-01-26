use anyhow::Result;
use memmap2::Mmap;
use std::env;
use std::fs::File;

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

pub struct TiffReader<'a, T: ByteOrder> {
    data: &'a [u8],
}

impl<'a, T: ByteOrder> TiffReader<'a, T> {
    fn read_u16(&self, index: usize) -> u16 {
        T::read_u16(&self.data[index..])
    }
}

fn main() -> Result<()> {
    let filename = env::args().nth(1).expect("Missing input file");
    let file = File::open(filename)?;
    let map = unsafe { Mmap::map(&file)? };
    let data = &map[..];
    match &data[..2] {
        b"II" => println!("little endian"),
        b"MM" => println!("big endian"),
        _ => panic!("First 2 bytes not recognized"),
    };
    Ok(())
}
