use std::error::Error;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{stdout, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::exit;

use printpdf::{
    image_crate::{self, GenericImageView},
    Image, Mm, PdfDocument,
};
use printpdf::{ImageTransform, PdfDocumentReference};

use clap::{App, Arg, ArgGroup, ValueHint};

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

    fn append_image_page(
        &self,
        image: &Path,
        dpi: f64,
        layer_name: &str,
        wh: (u32, u32),
    ) -> image_crate::ImageResult<()> {
        let img = image_crate::open(image)?.resize(
            wh.0,
            wh.1,
            image_crate::imageops::FilterType::Nearest,
        );
        let (w, h) = img.dimensions();
        let page_w = Mm((w as f64 * INCH_PER_MM) / dpi);
        let page_h = Mm((h as f64 * INCH_PER_MM) / dpi);

        let (page_i, layer_i) = self.pdf.add_page(page_w, page_h, layer_name);
        let layer = self.pdf.get_page(page_i).get_layer(layer_i);

        Image::from_dynamic_image(&img).add_to_layer(
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
                .multiple_occurrences(false)
                .multiple_values(false)
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
        .arg(Arg::new("dpi").default_value("96.0").long("dpi"))
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
        .group(
            ArgGroup::new("input")
                .args(&["imgs", "dir"])
                .multiple(false)
                .required(true),
        )
        .get_matches();

    let dpi = match matches.value_of("dpi").unwrap().parse::<f64>() {
        Ok(dpi) => dpi,
        Err(_) => {
            eprintln!("Value <dpi> could not be parsed as a float");
            exit(1)
        }
    };
    let width = match matches.value_of("scale-width").unwrap().parse::<u32>() {
        Ok(w) => w,
        Err(_) => {
            eprintln!("Value <scale-width> could not be parsed as an int");
            exit(1)
        }
    };
    let height = match matches.value_of("scale-height").unwrap().parse::<u32>() {
        Ok(h) => h,
        Err(_) => {
            eprintln!("Value <scale-height> could not be parsed as an int");
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

    let tic = std::time::Instant::now();
    let imgs_len = imgs_iter.len();
    for (i, n) in imgs_iter.iter().enumerate() {
        if let Err(e) = p.append_image_page(n, dpi, "", (width, height)) {
            println!("Skipping `{}` because: {}", n.display(), e);
        }
        print!("Processing image {}/{}\r", i, imgs_len);
        stdout().flush().unwrap();
    }
    p.save(&mut File::create(&out_path)?)?;

    println!(
        "Successfully created the PDF `{}` in {:.2}s",
        out_path.display(),
        tic.elapsed().as_secs_f32()
    );
    Ok(())
}
