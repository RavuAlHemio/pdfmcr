mod jpegparse;


use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;

use clap::{Parser, ValueEnum};


#[derive(Parser)]
struct Opts {
    #[arg(short, long, default_value = "dpi")]
    pub unit: Unit,

    #[arg(short, long)]
    pub width: u64,

    #[arg(short, long)]
    pub height: u64,

    #[arg(short, long)]
    pub input_file: PathBuf,

    #[arg(short, long)]
    pub output_file: PathBuf,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, ValueEnum)]
enum Unit {
    #[value(name = "cm")] Centimeters,
    #[value(name = "in")] Inches,
    #[value(name = "dpi")] DotsPerInch,
    #[value(name = "dpcm")] DotsPerCentimeter,
}


fn copy_over<R: Read, W: Write>(source: &mut R, destination: &mut W, mut byte_count: u64) {
    if byte_count == 0 {
        return;
    }

    let mut buf = vec![0u8; 4*1024*1024];
    let buf_size_u64: u64 = buf.len().try_into().unwrap();
    while byte_count > 0 {
        let read_this_many_u64 = byte_count.min(buf_size_u64);
        let read_this_many: usize = read_this_many_u64.try_into().unwrap();

        let actually_read = source.read(&mut buf[..read_this_many])
            .expect("failed to read while copying");
        if actually_read == 0 {
            panic!("file ended while copying");
        }
        let actually_read_u64: u64 = actually_read.try_into().unwrap();
        assert!(actually_read_u64 <= byte_count);
        byte_count -= actually_read_u64;

        destination.write_all(&buf[..actually_read])
            .expect("failed to write while copying");
    }
}


fn handle_app0<R: Read, W: Write>(source: &mut R, destination: &mut W) {
    let mut length_buf = [0u8; 2];
    source.read_exact(&mut length_buf)
        .expect("failed to read APP0 header length");
    let length_u16 = u16::from_be_bytes(length_buf);
    let length: usize = length_u16.into();
    if length < 2 {
        panic!("invalid APP0 header length (must be at least 2 bytes for length)");
    }
    let app0_data_length = length - 2;
    let mut app0_buf = vec![0u8; app0_data_length];

    source.read_exact(&mut app0_buf)
        .expect("failed to read APP0 header");

    // what kind of APP0 header is this?
    if app0_buf.starts_with(b"JFIF\0") {
        // JFIF, that's the one we care about

    } else {
        // no idea, just copy it over
        destination.write_all(&[0xFF, 0xE0])
            .expect("failed to write start of APP0 header");
        destination.write_all(&length_buf)
            .expect("failed to write length of APP0 header");
        destination.write_all(&app0_buf)
            .expect("failed to write APP0 header");
    }
}


fn main() {
    let opts = Opts::parse();

    // find the basic metadata of the JPEG file
    let mut input_file = File::open(&opts.input_file)
        .expect("failed to open input file");
    let mut output_file = File::create(&opts.output_file)
        .expect("failed to create output file");

    // image must start with Start of Image
    let mut buf2 = [0u8; 2];
    input_file.read_exact(&mut buf2)
        .expect("failed to read Start of Image");
    if buf2 != [0xFF, 0xD8] {
        panic!("invalid Start of Image -- expected 0xFF 0xD8, obtained 0x{:02X} 0x{:02X}", buf2[0], buf2[1]);
    }
    output_file.write_all(&buf2)
        .expect("failed to write Start of Image");

    loop {
        // what's the next block?
        input_file.read_exact(&mut buf2)
            .expect("failed to read next block");
        if buf2 == [0xFF, 0xE0] {
            // APP0, possibly JFIF?
            handle_app0(&mut input_file, &mut output_file);
        } else if buf2 == [0xFF, 0xE1] {
            // APP1, possibly Exif?
            handle_app1(&mut input_file, &mut output_file);
        } else if buf2 == [0xFF, 0xDA] {
            // Start of Scan; this one has no length following it
            break;
        } else {
            if buf2[0] != 0xFF {
                panic!("header starts with invalid byte 0x{:02X}", buf2[0]);
            }

            // some other kind of header
            output_file.write_all(&buf2)
                .expect("failed to write block header");

            input_file.read_exact(&mut buf2)
                .expect("failed to read block length");
            let block_length = u16::from_be_bytes(buf2);
            if block_length < 2 {
                panic!("invalid block length; must be at least 2 to accommodate the length bytes we just read");
            }

            // copy that
            let copy_count: u64 = (block_length - 2).into();
            copy_over(&mut input_file, &mut output_file, copy_count);

            // next header
        }
    }

    loop {
        // copy until we see 0xFF
        let mut buf1 = [0u8; 1];
        input_file.read_exact(&mut buf1)
            .expect("failed to read data byte");
        output_file.write_all(&buf1)
            .expect("failed to write data byte");

        if buf1[0] == 0xFF {
            // marker!

            // what kind?
            input_file.read_exact(&mut buf1)
                .expect("failed to read marker byte type");
            output_file.write_all(&buf1)
                .expect("failed to write marker byte type");

            if buf1[0] == 0x00 {
                // byte-stuffed non-marker; part of data
                // go again
            } else if buf1[0] == 0xD9 {
                // end of data; break out
                break;
            } else {
                panic!("unknown marker sequence 0xFF 0x{:02X} in image data", buf1[0]);
            }
        }
    }

    // ensure we wrote it all
    output_file.flush()
        .expect("failed to flush output file");

    // that's it
}
