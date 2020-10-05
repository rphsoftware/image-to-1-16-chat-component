use std::{env, fs};
use std::io::{Error, ErrorKind, Write};
use std::fs::File;
use std::path::Path;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: ./program <source directory> <target directory>");
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Invalid user input"
        ));
    }

    // Read input directory
    for entry in fs::read_dir(&args[1])? {
        let entry = entry?;
        let path = entry.path();

        // Open file descriptor
        let mut fd = File::open(path.clone())?;

        // Strip extension from file name
        let name = path.as_path().file_stem().unwrap_or_default();

        // Open writing file descriptor
        let mut write_str = String::new();
        write_str.push_str(&*args[2]);
        write_str.push_str("/");
        write_str.push_str(name.to_str().unwrap_or_default());
        write_str.push_str(".json");

        let mut write_fd = File::create(write_str)?;

        // Begin reading the png itself
        let decoder = png::Decoder::new(fd);
        let (info, mut reader) = decoder.read_info()?;

        let mut buf = vec![0u8; info.buffer_size()];
        let multiplier = match info.color_type {
            png::ColorType::RGB => Ok(3),
            png::ColorType::RGBA => Ok(4),
            _ => Err(Error::new(
                ErrorKind::InvalidInput,
                "Unsupported color space"
            ))
        }?;
        reader.next_frame(&mut buf)?;

        // Begin writing the json payload for the chat component
        let mut bb = String::with_capacity(0xFFFFFF);
        bb.push_str("{\"text\":\"\",\"extra\":[".as_ref());
        for y in 0..info.height as usize {
            for x in 0..info.width as usize {
                // Dibsed from https://github.com/rphsoftware/video-to-subtitle/blob/master/src/main.rs#L28
                let index = ((y * (info.width as usize)) * multiplier) + (x * multiplier);

                // Write first part of our component
                bb.push_str("{\"text\":\"■\",\"color\":\"#".as_ref());

                let r = buf[index];
                let g = buf[index + 1];
                let b = buf[index + 2];

                bb.push_str(format!("{:01$x}", r, 2).as_ref());
                bb.push_str(format!("{:01$x}", g, 2).as_ref());
                bb.push_str(format!("{:01$x}", b, 2).as_ref());
                bb.push_str("\"},".as_ref());
            }
            if (y + 1) < info.height as usize {
                bb.push_str("{\"text\":\"\\n\"},".as_ref());
            } else {
                bb.push_str("{\"text\":\"\\n\"}]}".as_ref());
            }
        }
        write_fd.write(bb.as_str().as_ref());
    }

    Ok(())
}
