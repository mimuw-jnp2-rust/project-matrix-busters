use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct DftMetadata {
    height: u32,
    width: u32,
}

#[derive(Serialize, Deserialize)]
struct DftPoint {
    re: f32,
    im: f32,
}

#[derive(Serialize, Deserialize)]
struct DftSource {
    metadata: DftMetadata,
    points: Vec<DftPoint>,
}

#[derive(Serialize, Deserialize)]
struct DftEpicycle {
    re: f32,
    im: f32,
    freq: f32,
    amp: f32,
    phase: f32,
}

#[derive(Serialize, Deserialize)]
struct DftResult {
    metadata: DftMetadata,
    epicycles: Vec<DftEpicycle>,
}

const MISSING_FILE: &str = "Missing file: ";
const INVALID_FILE: &str = "Invalid file: ";

// read file `filename` and return a DftSource using serde-json
fn read_source(filename: &str) -> Result<DftSource, String> {
    let file = std::fs::File::open(filename).map_err(|_| MISSING_FILE.to_string() + filename)?;
    let reader = std::io::BufReader::new(file);
    let source: DftSource =
        serde_json::from_reader(reader).map_err(|_| INVALID_FILE.to_string() + filename)?;
    Ok(source)
}

fn dft_algorithm(source: DftSource) -> Result<DftResult, String> {
    let DftSource { metadata, points } = source;
    let n = points.len();
    let mut epicycles = Vec::with_capacity(n);
    for k in 0..n {
        let mut re = 0.;
        let mut im = 0.;
        for (i, point) in points.iter().enumerate() {
            let angle = 2. * std::f32::consts::PI * k as f32 * i as f32 / n as f32;
            re += point.re * angle.cos() + point.im * angle.sin();
            im += point.im * angle.cos() - point.re * angle.sin();
        }
        re /= n as f32;
        im /= n as f32;
        let freq = k as f32;
        let amp = (re * re + im * im).sqrt();
        let phase = im.atan2(re);
        epicycles.push(DftEpicycle {
            re,
            im,
            freq,
            amp,
            phase,
        });
    }
    Ok(DftResult {
        metadata,
        epicycles,
    })
}

fn calculate_n(expected_points: usize, actual_points: usize) -> usize {
    if actual_points < expected_points {
        return actual_points;
    }
    return actual_points / expected_points;
}

fn take_every_nth<T>(source: Vec<T>, n: usize) -> Vec<T> {
    source.into_iter().step_by(n).collect()
}

fn save_result(result: DftResult, filename: &str) -> Result<(), String> {
    let file = std::fs::File::create(filename).map_err(|_| MISSING_FILE.to_string() + filename)?;
    let writer = std::io::BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &result)
        .map_err(|_| INVALID_FILE.to_string() + filename)?;
    Ok(())
}

fn main() -> Result<(), String> {
    println!("Generating DFT data...");

    const EXPECTED_POINTS: usize = 1000;
    const SOURCE_FILE: &str = "assets/dft_source.json";
    const RESULT_FILE: &str = "assets/dft_result.json";

    let source = read_source(SOURCE_FILE)?;
    let number_of_points = source.points.len();
    let source = DftSource {
        points: take_every_nth(
            source.points,
            calculate_n(EXPECTED_POINTS, number_of_points),
        ),
        ..source
    };
    let result = dft_algorithm(source)?;
    save_result(result, RESULT_FILE)
}
