use clap::{App, Arg};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    read: ReadType,
}

#[derive(Debug)]
enum ReadType {
    Lines(usize),
    Bytes(usize),
}

impl ReadType {
    pub fn new_lines(lines: usize) -> Self {
        Self::Lines(lines)
    }
    pub fn new_bytes(bytes: usize) -> Self {
        Self::Bytes(bytes)
    }
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("catr")
        .version("0.1.0")
        .author("Me!")
        .about("rusty helmet")
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .help("Input file(s)")
                .multiple(true)
                .default_value("-"),
        )
        .arg(
            Arg::with_name("lines")
                .short("n")
                .long("lines")
                .multiple(false)
                .value_name("LINES")
                .help("Number of lines to print")
                .default_value("10")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("bytes")
                .short("c")
                .value_name("BYTES")
                .multiple(false)
                .long("bytes")
                .help("Amount of bytes to print")
                .conflicts_with("lines")
                .takes_value(true),
        )
        .get_matches();

    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        read: if !matches.is_present("lines") && !matches.is_present("bytes") {
            ReadType::new_lines(10)
        } else if matches.is_present("bytes") {
            let val = parse_positive_int(matches.value_of("bytes").unwrap());
            ReadType::new_bytes(val?)
        } else {
            let val = parse_positive_int(matches.value_of("lines").unwrap());
            ReadType::new_lines(val?)
        },
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let num_files = config.files.len();
    for (file_num, file) in config.files.iter().enumerate() {
        match open(&file) {
            Err(err) => eprintln!("{}: {}", file, err),
            Ok(buffer) => {
                if num_files > 1 {
                    println!("{}==> {} <==", if file_num > 0 { "\n" } else { "" }, file)
                }
                print_file(buffer, &config.read)?;
            }
        };
    }
    Ok(())
}

fn print_file(mut file: Box<dyn BufRead>, read: &ReadType) -> MyResult<()> {
    match read {
        ReadType::Lines(num_lines) => {
            let mut line = String::new();
            for _ in 0..*num_lines {
                let bytes = file.read_line(&mut line)?;
                if bytes == 0 {
                    break;
                }
                print!("{}", line);
                line.clear();
            }
            Ok(())
        }
        ReadType::Bytes(bytes) => {
            let mut handle = file.take(*bytes as u64);
            let mut buffer = vec![0; *bytes];
            let bytes_read = handle.read(&mut buffer)?;
            print!("{}", String::from_utf8_lossy(&buffer[..bytes_read]));

            Ok(())
        }
    }
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

fn parse_positive_int(val: &str) -> MyResult<usize> {
    match val.parse() {
        Ok(n) if n > 0 => Ok(n),
        _ => Err(From::from(val)),
    }
}

#[test]
fn test_parse_positive_int() {
    // 3 is an okay int
    let res = parse_positive_int("3");
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 3);

    // any string is err
    let res = parse_positive_int("foo");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "foo".to_string());

    // zero is err
    let res = parse_positive_int("0");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "0".to_string());
}
