
use std::io;
use std::fs::File;
use std::env;
use std::mem;
use std::str;
use memmap::{Mmap, MmapOptions};
use zerocopy::{FromBytes, AsBytes, Unaligned, LayoutVerified};
use ::zerocopy::byteorder::{U16, U32};
use ::byteorder::BigEndian;
use std::error::Error;


#[repr(C)]
#[derive(FromBytes, AsBytes, Unaligned)]
#[derive(Debug, Copy, Clone)]
struct FontHeader {
    sfnt_version: U32<BigEndian>,
    num_tables: U16<BigEndian>,
    search_range: U16<BigEndian>,
    entry_selector: U16<BigEndian>,
    range_shift: U16<BigEndian>,
}


#[repr(C)]
#[derive(FromBytes, AsBytes, Unaligned)]
#[derive(Debug, Copy, Clone)]
struct TableHeader {
    raw_tag: U32<BigEndian>,
    checksum: U32<BigEndian>,
    offset: U32<BigEndian>,
    length: U32<BigEndian>
}


fn read<'m, T>(mmap: &'m Mmap, offset: usize, nth: usize) -> Result<LayoutVerified<&'m[u8], T>, Box<dyn Error>>
    where T: FromBytes {
    let size = mem::size_of::<T>();
    let offset = offset + nth * size;
    let raw: &[u8] = mmap.get(offset..offset + size).ok_or(io::Error::new(io::ErrorKind::Other, "Badness"))?;
    let lv = LayoutVerified::<&[u8], T>::new(raw).ok_or(io::Error::new(io::ErrorKind::Other, "Badness"))?;
    Ok(lv)
}


fn main()-> Result<(), Box<dyn Error>> {
    for arg in env::args().skip(1) {
        let mmap = unsafe {
            MmapOptions::new()
                        .map(&File::open(&arg)?)?
        };

        // ...are you copying the struct?
        let fh = *read::<FontHeader>(&mmap, 0, 0)?;

        println!("{} is {} bytes", &arg, mmap.len());
        println!("{:#?}", fh);

        for i in 0..fh.num_tables.get() {
            let th = *read::<TableHeader>(&mmap, mem::size_of::<FontHeader>(), usize::from(i))?;

            let tag_parts: &[u8; 4] = th.raw_tag.as_ref();
            let tag = str::from_utf8(tag_parts)?;
            println!("{} {} {:#?}", i, tag, th);
        }
    }
    Ok(())
}
