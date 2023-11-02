use chrono::{NaiveDateTime, Local, TimeZone, FixedOffset};
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::PathBuf,
};

const BEGIN_LOG: &str = "BEGIN_LOG";
const END_LOG: &str = "END_LOG";
const NEW_SPLIT: &str = "NEW";

fn main() {
    let path = PathBuf::from(std::env::args().nth(1).unwrap());
    let logfile = File::open(&path).unwrap();
    let output_directory = path.parent().unwrap();
    
    split_log(&logfile, output_directory.to_path_buf());
    print!("\nJobs done ...");
    std::io::stdout().flush().unwrap();

    let mut buf = "".to_string();
    std::io::stdin().read_line(&mut buf).unwrap();
}

fn get_date(unixtime: &str) -> String {
    let ndt = NaiveDateTime::from_timestamp_millis(unixtime.parse::<i64>().unwrap()).unwrap();
    let tz_offset = FixedOffset::offset_from_local_datetime(Local::now().offset(), &ndt).unwrap();
    let dt = NaiveDateTime::and_local_timezone(&ndt, tz_offset).unwrap();

    format!("{}", dt.format("%Y-%m-%d_%H-%M"))
}

struct Split {
    date: String,
    instance: String,
    writer: Option<Box<dyn Write>>,
}

fn split_log(logfile: &File, output_directory: PathBuf) {
    let mut s = Split {date: String::from(""), instance: String::from(""), writer: None};

    let logfile = BufReader::new(logfile);
    logfile.lines()
        .for_each(|line| {
            if line.as_ref().unwrap().contains(BEGIN_LOG) {
                s.date = line.unwrap();
                s.instance = NEW_SPLIT.to_string();
            }
            else if s.instance == NEW_SPLIT {
                s.instance = line.unwrap();

                let file = format!("{}_{}.log",
                   get_date(s.date.split(',').nth(2).unwrap()),
                   s.instance.split(',').nth(3).unwrap().trim_matches('\"'),
                );
                println!("{}", file);

                s.writer = Some(Box::new(BufWriter::new(File::create(&output_directory.join(&file)).unwrap())));
                writeln!(&mut s.writer.as_mut().unwrap(), "{}\n{}", s.instance, s.date).unwrap()
            }
            else if line.as_ref().unwrap().contains(END_LOG) {
                writeln!(&mut s.writer.as_mut().unwrap(), "{}\n", line.unwrap()).unwrap();
                s.writer.as_mut().unwrap().flush().unwrap();
                
                s.date = String::from("");
                s.instance = String::from("");
            }
            else {
                writeln!(&mut s.writer.as_mut().unwrap(), "{}\n", line.unwrap()).unwrap();
            }
        });
}
