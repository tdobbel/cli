mod geotiff;

use anyhow::Result;
use geotiff::TiffReader;
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
    println!("{}", tif.get(0, 0)?);
    Ok(())
}
