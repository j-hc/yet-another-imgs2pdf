use std::error::Error;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;
use std::process::exit;

use printpdf::{image_crate, Image, Mm, PdfDocument};
use printpdf::{ImageTransform, PdfDocumentReference};

use clap::{App, Arg, ValueHint};

const INCH_PER_MM: f64 = 25.4;

struct PDFMerger {
    pdf: PdfDocumentReference,
}
impl PDFMerger {
    fn new(title: &str) -> Self {
        Self {
            pdf: PdfDocument::empty(title),
        }
    }

    fn append_image_page<T: Read>(
        &self,
        image: &mut BufReader<T>,
        dpi: f64,
        layer_name: &str,
        wh: (u16, u16),
    ) -> image_crate::ImageResult<()> {
        let mut img = image_crate::jpeg::JpegDecoder::new(image)?;
        let scaled_szs = img.scale(wh.0, wh.1)?;
        let page_w = Mm((scaled_szs.0 as f64 * INCH_PER_MM) / dpi);
        let page_h = Mm((scaled_szs.1 as f64 * INCH_PER_MM) / dpi);

        let (page_i, layer_i) = self.pdf.add_page(page_w, page_h, layer_name);
        let layer = self.pdf.get_page(page_i).get_layer(layer_i);

        Image::try_from(img)?.add_to_layer(
            layer,
            ImageTransform {
                dpi: Some(dpi),
                ..Default::default()
            },
        );
        Ok(())
    }

    fn save(self, sink: impl Write) -> Result<(), printpdf::Error> {
        self.pdf.save(&mut BufWriter::new(sink))
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about("Merge multiple images into a single pdf")
        .author("scrubjay55")
        .arg(
            Arg::new("dir")
                .help("Directory to folder of images")
                .required(true)
                .multiple_occurrences(false)
                .multiple_values(false)
                .conflicts_with("imgs")
                .value_hint(ValueHint::DirPath)
                .long("dir")
                .short('d'),
        )
        .arg(
            Arg::new("imgs")
                .help("Paths to multiple images seperated with a whitespace")
                .multiple_values(true)
                .value_hint(ValueHint::FilePath)
                .long("imgs")
                .short('i'),
        )
        .arg(
            Arg::new("out")
                .value_hint(ValueHint::FilePath)
                .required(true)
                .long("out")
                .short('o'),
        )
        .arg(Arg::new("dpi").default_value("100.0").long("dpi"))
        .arg(
            Arg::new("scale-width")
                .default_value("")
                .long("scale-width")
                .short('w')
                .default_value("720"),
        )
        .arg(
            Arg::new("scale-height")
                .default_value("")
                .long("scale-height")
                .short('h')
                .default_value("1280"),
        )
        .arg(
            Arg::new("auto-sort")
                .takes_value(false)
                .long("auto-sort")
                .short('s'),
        )
        .arg(
            Arg::new("pdf-title")
                .hide_default_value(true)
                .default_value("")
                .long("pdf-title")
                .short('t'),
        )
        .get_matches();

    let dpi = match matches.value_of("dpi").unwrap().parse::<f64>() {
        Ok(dpi) => dpi,
        Err(_) => {
            eprintln!("Value <dpi> could not be parsed as a float");
            exit(1)
        }
    };
    let width = match matches.value_of("scale-width").unwrap().parse::<u16>() {
        Ok(w) => w,
        Err(_) => {
            eprintln!("Value <scale-width> could not be parsed as an unsigned integer");
            exit(1)
        }
    };
    let height = match matches.value_of("scale-height").unwrap().parse::<u16>() {
        Ok(h) => h,
        Err(_) => {
            eprintln!("Value <scale-height> could not be parsed as an unsigned integer");
            exit(1)
        }
    };
    let mut out_path = PathBuf::from(matches.value_of("out").unwrap());
    if out_path.extension() != Some(OsStr::new("pdf")) {
        out_path.set_extension("pdf");
    }

    let p = PDFMerger::new(matches.value_of("pdf-title").unwrap());
    let mut imgs_iter = if let Some(imgs) = matches.values_of("imgs") {
        imgs.map(PathBuf::from).collect::<Vec<PathBuf>>()
    } else if let Some(f) = matches.value_of("dir") {
        match std::fs::read_dir(f) {
            Ok(rds) => rds
                .filter_map(|rd| rd.map(|de| de.path()).ok())
                .collect::<Vec<PathBuf>>(),
            Err(e) => {
                eprintln!("Could not read <dir> `{f}`: {e}");
                exit(1)
            }
        }
    } else {
        unreachable!();
    };
    if matches.value_of("auto-sort").is_some() {
        imgs_iter.sort();
    }

    for n in imgs_iter {
        let mut image_file = BufReader::new(File::open(n)?);
        p.append_image_page(&mut image_file, dpi, "", (width, height))?;
    }
    p.save(&mut File::create(&out_path)?)?;

    println!("Created the PDF successfully `{}`", out_path.display());
    Ok(())
}
