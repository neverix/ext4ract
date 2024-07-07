use std::any::Any;
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <path_to_img_file> <output_directory>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    let output_dir = &args[2];

    // Create the output directory if it doesn't exist
    fs::create_dir_all(output_dir).expect("Failed to create output directory");

    let r = fs::File::open(filename).expect("openable file");
    let mut options = ext4::Options::default();
    options.checksums = ext4::Checksums::Enabled;
    let vol = ext4::SuperBlock::new_with_options(r, &options).expect("ext4 volume");
    let root = vol.root().expect("root");

    vol.walk(&root, "/", &mut |vol, path, entry, _| {
        let full_path = Path::new(output_dir).join(path.trim_start_matches('/'));

        let enhanced = vol.enhance(entry).expect("enhancement failed");

        if let ext4::Enhanced::Directory(_) = enhanced {
            fs::create_dir_all(&full_path).expect("Failed to create directory");
        }
        if let ext4::Enhanced::RegularFile = enhanced {
            let to_read = usize::try_from(entry.stat.size).unwrap();
            let mut buf = vec![0u8; to_read];

            vol.open(entry)?.read_exact(&mut buf)?;

            let mut output_file = fs::File::create(&full_path).expect("Failed to create output file");
            output_file.write_all(&buf).expect("Failed to write file contents");
        };

        Ok(true)
    })
    .expect("walk");

    println!("Extraction complete. Files saved to: {}", output_dir);
}