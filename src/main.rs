//! # A tool for translating textures to new UV mappings on similar objects

use anyhow::Result;
use clap::Parser;
use image::{io::Reader, ImageBuffer, Rgb};
use nalgebra::{Matrix2, Matrix3, Vector2, Vector3};

type Image = ImageBuffer<Rgb<u8>, Vec<u8>>;
type Triangle = (Vector2<f64>, Vector2<f64>, Vector2<f64>);

#[derive(Debug)]
struct Position {
    x: u32,
    y: u32,
}

/// A tool for translating textures to new UV mappings on similar objects
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    input_uv: String,
    output_uv: String,
    // map_file: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("Loading UVs");
    let result = Reader::open(args.input_uv)?;
    let input_image = result.decode()?;
    let input_image = input_image.to_rgb8();

    println!("Finding Triangles");
    let positions = find_markers(input_image);
    println!("{positions:?}");

    let input_triangle = make_triangle(&positions[0], &positions[1], &positions[2]);
    let output_triangle = make_triangle(&positions[0], &positions[1], &positions[2]);

    println!("Building Matrices");
    get_transform(input_triangle, output_triangle);

    // Save
    // println!("Saving output image");
    // let output_image = DynamicImage::from(img).to_rgb8();
    // output_image.save(args.map_file)?;

    Ok(())
}

/// Returns the position of each point
///
/// The order considers the RGB colors as a weight BGR where R is least significant
/// and B most significant. Look at get_precendence for the exact definition.
fn find_markers(image: Image) -> Vec<Position> {
    let (width, height) = image.dimensions();
    let ignore_color = Rgb { 0: [0; 3] };

    struct Marker<'a> {
        pixel: &'a Rgb<u8>,
        position: Position,
    }

    // Find all markers
    let mut markers = Vec::new();

    for x in 0..width {
        for y in 0..height {
            let pixel = image.get_pixel(x, y);

            if pixel != &ignore_color {
                let marker = Marker {
                    pixel,
                    position: Position { x, y },
                };

                markers.push(marker);
            }
        }
    }

    // Put them in order
    markers.sort_by(|a, b| {
        let a = get_precedence(&a.pixel);
        let b = get_precedence(&b.pixel);

        a.cmp(&b)
    });

    // Discard color information
    markers.into_iter().map(|marker| marker.position).collect()
}

fn get_precedence(pixel: &Rgb<u8>) -> u32 {
    let r = pixel.0[0] as u32;
    let g = pixel.0[1] as u32;
    let b = pixel.0[2] as u32;

    r + 256 * g + 256 * 256 * b
}

fn make_triangle(a: &Position, b: &Position, c: &Position) -> Triangle {
    let a = Vector2::new(a.x as f64, a.y as f64);
    let b = Vector2::new(b.x as f64, b.y as f64);
    let c = Vector2::new(c.x as f64, c.y as f64);

    (a, b, c)
}

fn pad_matrix(matrix: Matrix2<f64>) -> Matrix3<f64> {
    Matrix3::new(
        matrix.m11, matrix.m12, 0.0, matrix.m21, matrix.m22, 0.0, 0.0, 0.0, 1.0,
    )
}

fn pad_vector(vector: Vector2<f64>) -> Vector3<f64> {
    Vector3::new(vector.x, vector.y, 1.0)
}

fn get_transform(input: Triangle, output: Triangle) -> Matrix3<f64> {
    todo!()
}
