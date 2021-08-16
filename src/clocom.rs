use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::iter::Map;
use std::process::Command;
use termion::color;

struct StockVisualizer {
    console_width: usize,
    console_height: usize,
    data: StockData,
    max: f32,
    normalization_value: f32,
    lrc: LinearRegressionCalculator,
}

impl StockVisualizer {
    fn default() -> Self {
        let mut sv = StockVisualizer {
            console_width: 0,
            console_height: 0,
            data: StockData {
                records: Default::default(),
                name: String::new(),
            },
            max: 0.0,
            normalization_value: 0.0,
            lrc: LinearRegressionCalculator::default(),
        };
        sv.get_console_width();
        sv.get_console_height();
        sv
    }

    // visualize prints stock data.
    pub fn visualize(mut self) {
        self.console_width -= 6; // for scale
                                 // step in which we should avg stock data;
        let step = (self.data.records.len() / self.console_width);
        // get new data with calculated step;
        // TODO: make F with processing function.
        let mut new_data = self.avg_by_step(step);
        new_data.truncate(self.console_width);
        self.calc_max_value(&new_data);

        // normalize data in height dimension;
        self.normalize_and_round_height(&mut new_data);

        self.lrc.calculate(self.data_to_points(&new_data));

        self.print(new_data);
    }

    fn data_to_points(&self, data: &Vec<Record>) -> Vec<Point> {
        let mut points: Vec<Point> = Vec::new();

        data.iter()
            .enumerate()
            .for_each(|(i, r)| points.push(Point::new(i as f32, r.open)));
        points
    }

    fn calc_max_value(&mut self, data: &Vec<Record>) {
        let mut max: f32 = 0.0;
        data.iter().for_each(|r| {
            if r.open > max {
                max = r.open
            }
        });
        self.max = max;
    }

    fn adjust_width(&mut self) {
        self.console_width -= (self.max as usize % 10) + 3; // +3 for 2 decimal numbers and '|'
    }

    fn print(self, data: Vec<Record>) {
        let mut regression_points: Vec<f32> = Vec::new();
        for i in 0..self.console_width {
            regression_points.push(self.lrc.a * i as f32 + self.lrc.b);
        }

        let mut scale_points: HashMap<usize, f32> = HashMap::new();
        for i in 0..self.console_height {
            scale_points.insert(i, self.max - (self.normalization_value * i as f32));
        }

        let mut regression_scale_points: HashMap<usize, usize> = HashMap::new();

        // match regression_points with closest scale_point.
        for (i, rp) in regression_points.iter().enumerate() {
            let mut min: f32 = f32::MAX;
            let mut minx_inx: usize = 0;

            for (k, v) in &scale_points {
                if (rp - v).abs() < min {
                    min = (rp - v).abs();
                    minx_inx = *k;
                }
            }
            regression_scale_points.insert(i, minx_inx);
        }

        for i in 0..self.console_height {
            let mut row = String::new();

            row.push_str(
                format!(
                    "{:<5.2}{}|{}",
                    scale_points.get(&i).unwrap(),
                    color::Fg(color::Green),
                    color::Fg(color::Reset)
                )
                .as_str(),
            );

            for (j, r) in data.iter().enumerate() {
                let rsc = regression_scale_points.get(&j).unwrap();
                if rsc == &i {
                    row.push_str(
                        format!("{}#{}", color::Bg(color::Red), color::Bg(color::Reset)).as_str(),
                    );
                } else if self.console_height - i <= r.open as usize {
                    row.push('#');
                } else {
                    row.push(' ');
                }
            }
            println!("{}", row);
        }
    }

    fn normalize_and_round_height(&mut self, data: &mut Vec<Record>) {
        println!(
            "height: {}, width: {}",
            self.console_height, self.console_width
        );

        let norm_val = self.max / self.console_height as f32;

        for item in data {
            item.open = (item.open / norm_val).round();
        }
        self.normalization_value = norm_val;
    }

    fn avg_by_step(&self, step: usize) -> Vec<Record> {
        let mut new_data: Vec<Record> = Vec::new();
        let mut open: f32 = 0.0;
        for (i, item) in self.data.records.iter().enumerate() {
            if i != 0 && i % step == 0 {
                new_data.push(Record {
                    date: item.date.clone(),
                    open: (open / step as f32) as f32,
                    high: 0.0,
                    low: 0.0,
                    close: 0.0,
                });
                open = 0.0;
            } else {
                open += item.open;
            }
        }
        return new_data;
    }

    fn get_console_width(&mut self) {
        let mut output = Command::new("sh")
            .arg("-c")
            .arg("tput cols")
            .output()
            .expect("failed to execute process");
        output.stdout.remove(output.stdout.len() - 1);

        self.console_width = self.parse_tput_output(output.stdout);
    }

    fn get_console_height(&mut self) {
        let mut output = Command::new("sh")
            .arg("-c")
            .arg("tput lines")
            .output()
            .expect("failed to execute process");
        output.stdout.remove(output.stdout.len() - 1);

        self.console_height = self.parse_tput_output(output.stdout);
    }

    fn parse_tput_output(&self, output: Vec<u8>) -> usize {
        const ASCII_0: usize = 48;
        let mut parsed: usize = 0;
        let len = output.len();
        output.iter().enumerate().for_each(|(i, item)| {
            let it = *item as usize - ASCII_0;
            let mult = (10 as usize).pow((len - i - 1) as u32);
            parsed += it * mult;
        });
        parsed
    }
}

#[derive(Debug)]
struct StockData {
    name: String,
    records: Vec<Record>,
}

#[derive(Debug, Deserialize)] // Derive is cool, I have no idea how it works!
struct Record {
    date: String,
    open: f32,
    high: f32,
    low: f32,
    close: f32,
}

// impl Record {
//     fn default() -> Self {
//         Record{
//             date: "".to_string(),
//             open: 0.0,
//             high: 0.0,
//             low: 0.0,
//             close: 0.0
//         }
//     }
// }

pub fn read_stock_data(path: &str) -> Result<(), Box<dyn Error>> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut records: Vec<Record> = Vec::new();
    let mut name = String::new();

    if name.is_empty() {
        name = String::from("ALL")
    }

    for result in reader.deserialize().into_iter() {
        let record = result?;
        // for (i, field) in record.iter().enumerate() {
        //     match i {
        //         0 => {r.date = field.parse().unwrap() }, // date
        //         1 => {r.open = string_to_f32(field).expect("failed to parse open")}, // open
        //         2 => {r.high = string_to_f32(field).expect("failed to parse high")}, // high
        //         3 => {r.low = string_to_f32(field).expect("failed to parse low")}, // low
        //         4 => {r.close = string_to_f32(field).expect("failed to parse close")}, // close
        //         _ => {continue}
        //     }
        // }
        records.push(record);
    }

    let mut vis = StockVisualizer::default();
    vis.data = StockData { name, records };

    vis.visualize();

    Ok(())
}

pub struct LinearRegressionCalculator {
    a: f32,
    b: f32,
}

pub struct Point(f32, f32);

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Point(x, y)
    }
}

impl LinearRegressionCalculator {
    pub fn default() -> Self {
        LinearRegressionCalculator { a: 0.0, b: 0.0 }
    }
    // calculate calculates a and b parameters of linear function.
    //
    // b = [n*sum(x*y) - (sum(x)*sum(y)] / [n*sum(x^2) - (sum(x))^2]
    // a = [n*sum(y) - b*sum(x)]/n
    pub fn calculate(&mut self, data: Vec<Point>) {
        let n = data.len() as f32;

        let x_sum: f32 = data.iter().map(|p| p.0).sum();
        let y_sum: f32 = data.iter().map(|p| p.1).sum();
        let xy_sum: f32 = data.iter().map(|p| p.0 * p.1).sum();
        let x2_sum: f32 = data.iter().map(|p| p.0.powi(2)).sum();
        let x_sum2: f32 = x_sum.powi(2);

        let b = (y_sum * x2_sum - x_sum * xy_sum) / (n * x2_sum - x_sum2);
        let a = (n * xy_sum - x_sum * y_sum) / (n * x2_sum - x_sum2);

        self.a = a;
        self.b = b;
        println!("{}x + {}", a, b)
    }

    pub fn describe(self) -> (f32, f32) {
        (self.a, self.b)
    }
}
