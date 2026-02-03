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
    let mut tiff_data = tiff_reader.read_tiff()?;
    println!("{:?}", tiff_data.get_extent());
    tiff_data.load_data(&mut tiff_reader);
    println!("{}", tiff_data.get(0, 0)?);
    Ok(())
}
