mod geotiff;

use anyhow::Result;
use geotiff::{SampleFormat, TiffReader};
use memmap2::Mmap;
use std::env;
use std::fs::File;

fn main() -> Result<()> {
    let filename = env::args().nth(1).expect("Missing input file");
    let file = File::open(filename)?;
    let map = unsafe { Mmap::map(&file)? };
    let mut tiff_reader = TiffReader::new(map)?;
    let mut tif = tiff_reader.read_tiff()?;
    println!("{:?}", tif.get_extent());
    tif.load_data(&mut tiff_reader)?;
    let (ny, nx) = tif.shape();
    println!("{}", tif.get(ny - 1, nx - 1)?);
    if matches!(tif.get_sample_format(), SampleFormat::Float) {
        println!("{}", tif.get_f32(ny - 1, nx - 1)?);
    } else {
        println!("{}", tif.get_i32(ny - 1, nx - 1)?);
    }
    Ok(())
}
