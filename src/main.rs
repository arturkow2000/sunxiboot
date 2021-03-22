use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

use anyhow::Context;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use clap::Clap;

#[derive(Clap)]
struct Options {
    #[clap(subcommand)]
    pub subcommand: SubCommand,
}
#[derive(Clap)]
enum SubCommand {
    #[clap(about = "Compute header checksum and write it to file.")]
    Checksum { file: PathBuf },
}

fn open_file(subcommand: &SubCommand) -> Result<File, io::Error> {
    let (write, path) = match subcommand {
        SubCommand::Checksum { file } => (true, file.as_path()),
    };
    OpenOptions::new().read(true).write(write).open(path)
}

fn main() -> anyhow::Result<()> {
    let options: Options = Options::parse();
    let file = open_file(&options.subcommand).context("Failed to open file")?;
    execute_command(&options, file)
}

fn execute_command(options: &Options, file: File) -> anyhow::Result<()> {
    match &options.subcommand {
        SubCommand::Checksum { .. } => execute_checksum_command(file),
    }
}

fn execute_checksum_command(mut file: File) -> anyhow::Result<()> {
    let _ = file.read_u32::<LittleEndian>();
    let mut signature = [0u8; 8];
    file.read(&mut signature[..])?;
    if &signature != b"eGON.BT0" {
        return Err(anyhow::Error::msg("Invalid signature"));
    }
    let _ = file.read_u32::<LittleEndian>();
    let mut length = file.read_u32::<LittleEndian>()?;

    if length % 4 != 0 {
        return Err(anyhow::Error::msg("Length is not multiple of 4"));
    }
    length /= 4;

    file.seek(SeekFrom::Start(0))?;

    let mut checksum = 0u32;
    for i in 0..length as usize {
        let t = file.read_u32::<LittleEndian>()?;
        checksum = checksum.wrapping_add(if i == 3 { 0x5f0a6c39 } else { t });
    }

    file.seek(SeekFrom::Start(12))?;
    file.write_u32::<LittleEndian>(checksum)?;

    file.flush()?;
    Ok(())
}
