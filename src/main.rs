use std::{
    fmt::Display,
    fs::OpenOptions,
    io::{BufRead, BufReader, Write},
    iter,
    str::FromStr,
};

use clap::{Parser, Subcommand};

#[derive(Debug, Subcommand, Clone)]
enum Mode {
    /// Encode the file in the format to be read by the verilog
    Encode { dest_file: String },
    /// Decode the file to a human readable format
    Decode { dest_file: String },
    /// Hash the file, do not write to file
    Hash,
}

#[derive(Parser, Debug)]
struct Args {
    #[clap(subcommand)]
    pub mode: Mode,
    /// Source file to be read
    pub filename: String,
}

#[derive(Debug)]
struct DataLine {
    length_valid: bool,
    length: u32,
    data_valid: bool,
    data: u8,
}

impl FromStr for DataLine {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut split = value.split(&[' ', '_']);
        let length_valid = split
            .next()
            .unwrap()
            .parse::<u8>()
            .expect("length valid invalid")
            == 1;
        let length = u32::from_str_radix(split.next().unwrap(), 2).expect("Data Length invalid");
        let data_valid = split
            .next()
            .unwrap()
            .parse::<u8>()
            .expect("Data Valid value invalid")
            == 1;
        let data =
            u8::from_str_radix(split.next().unwrap(), 2).expect("Failed to read Data in line");
        Ok(DataLine {
            length_valid,
            length,
            data_valid,
            data,
        })
    }
}

impl From<u8> for DataLine {
    fn from(value: u8) -> Self {
        Self {
            length_valid: false,
            length: 0,
            data_valid: true,
            data: value,
        }
    }
}

impl Display for DataLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}_{:0>32b}_{}_{:0>8b}",
            self.length_valid as u8, self.length, self.data_valid as u8, self.data
        )
    }
}

struct DataStream<I>
where
    I: Iterator<Item = DataLine>,
{
    data: I,
    length: u32,
    content: String,
    a: u16,
    b: u16,
}

impl<I> DataStream<I>
where
    I: Iterator<Item = DataLine>,
{
    fn new(data: I) -> Self {
        Self {
            data,
            content: String::new(),
            length: 0,
            a: 1,
            b: 0,
        }
    }

    fn reset(&mut self) {
        self.a = 1;
        self.b = 0;
        self.content.clear();
        self.length = 0;
    }

    fn checksum(&self) -> u32 {
        let b = (self.b as u32).wrapping_shl(16);
        let a = self.a as u32;
        b | a
    }
}

impl<I> Iterator for DataStream<I>
where
    I: Iterator<Item = DataLine>,
{
    type Item = (u32, String);

    fn next(&mut self) -> Option<Self::Item> {
        for next in self.data.by_ref() {
            if next.length_valid {
                self.length = next.length;
            }

            if next.data_valid && self.length > 0 {
                self.content.push(next.data as char);
                self.a = (self.a + next.data as u16) % 65521;
                self.b = self.b.overflowing_add(self.a).0 % 65521;
                self.length -= 1;
                if self.length == 0 {
                    let retval = (self.checksum(), self.content.clone());
                    self.reset();
                    return Some(retval);
                }
            }
        }
        None
    }
}

fn main() {
    let args = Args::parse();

    match args.mode {
        Mode::Hash => {
            let file = OpenOptions::new()
                .read(true)
                .open(args.filename)
                .expect("Failed to open file");
            // Read the lines
            let line_iter = std::io::BufReader::new(file).lines();
            let data = line_iter
                .map(|x| x.expect("Failed to read line"))
                .filter(|x| !x.starts_with("#")) // Anything with a # is a comment
                .map(|x| x.parse::<DataLine>().expect("Failed to parse line"));

            DataStream::new(data)
                .into_iter()
                .for_each(|(checksum, content)| {
                    println!("Checksum: 32'h{:0>8x} Content: {:?}", checksum, content);
                });
        }
        Mode::Encode { dest_file } => {
            let source = OpenOptions::new()
                .read(true)
                .open(args.filename)
                .expect("Failed to open source file");
            let source = BufReader::new(source);
            let mut dest = OpenOptions::new()
                .write(true)
                .create(true)
                .append(true)
                .open(dest_file)
                .expect("Failed to open destination file");

            let source_lines: Vec<DataLine> = source
                .lines()
                .map(|l| l.expect("Failed to read line"))
                .map(|line| {
                    iter::once(DataLine {
                        length_valid: true,
                        length: line.len() as u32,
                        data_valid: false,
                        data: 0,
                    })
                    .chain(line.bytes().map(|character| DataLine::from(character)))
                    .collect::<Vec<_>>() // This could be avoided maybe. I'm .... rusty
                })
                .flatten()
                .collect();

            for line in &source_lines {
                dest.write_fmt(format_args!("{line}\n"))
                    .expect("failed to write to file");
            }
            println!("Wrote {} lines", source_lines.len());
        }
        Mode::Decode { dest_file } => {
            let file = OpenOptions::new()
                .read(true)
                .open(args.filename)
                .expect("Failed to open file");
            let mut dest = OpenOptions::new()
                .write(true)
                .create(true)
                .open(dest_file)
                .expect("Failed to open destination file");
            // Read the lines
            let line_iter = std::io::BufReader::new(file).lines();
            let data = line_iter
                .map(|x| x.expect("Failed to read line"))
                .filter(|x| !x.starts_with("#")) // Anything with a # is a comment
                .map(|x| x.parse::<DataLine>().expect("Failed to parse line"));

            DataStream::new(data)
                .into_iter()
                .for_each(|(checksum, content)| {
                    dest.write_fmt(format_args!("{}\n", content))
                        .expect("Failed to write to file");
                    println!("Checksum: 32'h{:0>8x} Content: {:?}", checksum, content);
                });
        }
    }
    // println!("Checksum: 32'h{:x}", v);
}
