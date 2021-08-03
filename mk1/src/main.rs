
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

// Do NOT #derive Copy; far too easy to accidentally copy things

#[repr(C)]
#[derive(FromBytes, AsBytes, Unaligned)]
#[derive(Debug, Clone)]
struct FontHeader {
    sfnt_version: U32<BigEndian>,
    num_tables: U16<BigEndian>,
    search_range: U16<BigEndian>,
    entry_selector: U16<BigEndian>,
    range_shift: U16<BigEndian>,
}

#[repr(C)]
#[derive(FromBytes, AsBytes, Unaligned)]
#[derive(Debug, Clone)]
struct TableHeader {
    raw_tag: U32<BigEndian>,
    checksum: U32<BigEndian>,
    offset: U32<BigEndian>,
    length: U32<BigEndian>,
}

#[repr(C)]
#[derive(FromBytes, AsBytes, Unaligned)]
#[derive(Debug, Clone)]
struct cmapHeader {
    version: U16<BigEndian>,
    num_tables: U16<BigEndian>,
}

#[repr(C)]
#[derive(FromBytes, AsBytes, Unaligned)]
#[derive(Debug, Clone)]
struct cmapEncodingRecord {
    platform_id: U16<BigEndian>,
    encoding_id: U16<BigEndian>,
    subtable_offset: U32<BigEndian>,
}

// You horrible horrible table. I mean ... wonderful test case.
// Ref https://github.com/fonttools/fonttools/blob/5cb288f3453bb7a2adea0437f0b7252efc12e321/Lib/fontTools/ttLib/tables/_c_m_a_p.py#L680
#[repr(C)]
#[derive(FromBytes, AsBytes, Unaligned)]
#[derive(Debug, Clone)]
struct cmapFormat4 {
    // The fixed size parts of format 4
    format: U16<BigEndian>,
    length: U16<BigEndian>,
    language: U16<BigEndian>,
    seg_countx2: U16<BigEndian>,
    search_range: U16<BigEndian>,
    entry_selector: U16<BigEndian>,
    range_shift: U16<BigEndian>,

}

// Ref https://docs.rs/zerocopy/0.5.0/zerocopy/struct.LayoutVerified.html
// Ref https://www.youtube.com/watch?v=VjAqt0gt920 asserts Typic might be an evolution of zerocopy
// Ref https://gankra.github.io/blah/rust-layouts-and-abis/
fn read<'m, T>(mmap: &'m Mmap, offset: usize, nth: usize) -> Result<LayoutVerified<&'m [u8], T>, Box<dyn Error>>
    where T: FromBytes {
    let size = mem::size_of::<T>();
    let offset = offset + nth * size;
    let raw: &[u8] = mmap.get(offset..offset + size).ok_or(io::Error::new(io::ErrorKind::Other, "Badness"))?;
    let lv = LayoutVerified::<&[u8], T>::new(raw).ok_or(io::Error::new(io::ErrorKind::Other, "Badness"))?;
    Ok(lv)
}

fn main()-> Result<(), Box<dyn Error>> {
    for arg in env::args().skip(1) {
        // Will structs happily outlive the mmap? - that would be bad
        let mmap = unsafe {
            MmapOptions::new()
                        .map(&File::open(&arg)?)?
        };
        
        let fh = read::<FontHeader>(&mmap, 0, 0)?;

        println!("{} is {} bytes", &arg, mmap.len());
        println!("{:#?}", *fh);

        for i in 0..fh.num_tables.get() {
            let th = read::<TableHeader>(&mmap, mem::size_of::<FontHeader>(), usize::from(i))?;

            let tag_parts: &[u8; 4] = th.raw_tag.as_ref();
            let tag = str::from_utf8(tag_parts)?;
            println!("{} {}", i, tag);

            if tag == "cmap" {
                let cmap_start = th.offset.get() as usize;
                let cmapHeader = read::<cmapHeader>(&mmap, cmap_start, 0)?;
                let er_start = cmap_start + mem::size_of::<cmapHeader>();
                for j in 0..cmapHeader.num_tables.get() {
                    let er = read::<cmapEncodingRecord>(&mmap, er_start, j as usize)?;
                    println!("  {:#?}", *er);
                    let subtable_start = cmap_start + er.subtable_offset.get() as usize;
                    let fmt = read::<U16<BigEndian>>(&mmap, subtable_start, 0)?.get();
                    println!("  {}", fmt);
                    if fmt == 4 {
                        let fmt4 = read::<cmapFormat4>(&mmap, subtable_start, 0)?;
                        println!("  {:#?}", *fmt4);
                    }
                }
            }
        }
    }
    Ok(())
}
