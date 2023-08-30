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
    let matrix = get_transform(input_triangle, output_triangle);

    println!("Transformation matrix: {matrix:?}");

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

/// Pads a 2x2 transform matrix to a 3x3 one
fn pad_matrix(matrix: &Matrix2<f64>) -> Matrix3<f64> {
    Matrix3::new(
        matrix.m11, matrix.m12, 0.0, matrix.m21, matrix.m22, 0.0, 0.0, 0.0, 1.0,
    )
}

/// Chops the 3rd dimension off a 3d vector
// fn chop_vector(vector: &Vector3<f64>) -> Vector2<f64> {
//     Vector2::new(vector.x, vector.y)
// }

/// Adds a 3rd dimension 1 to a vector
fn pad_vector(vector: &Vector2<f64>) -> Vector3<f64> {
    Vector3::new(vector.x, vector.y, 1.0)
}

/// Gets the transformation matrix to go from an input triangle to output
fn get_transform(input: Triangle, output: Triangle) -> Matrix3<f64> {
    // Find translation matrix
    let translation_vector = output.0 - input.0;
    let translation_matrix = Matrix3::new_translation(&translation_vector);

    // Find rotation matrix
    let input_01 = input.1 - input.0;
    let output_01 = output.1 - output.0;

    let input_01_angle = input_01.y.atan2(input_01.x);
    let output_01_angle = output_01.y.atan2(output_01.x);

    let angle_difference = output_01_angle - input_01_angle;
    let rotation_matrix = Matrix3::new_rotation(angle_difference);

    // Find change of basis matrix
    let rot_270_matrix = Matrix2::new(0.0, 1.0, 1.0, 0.0);
    let output_01_perpendicular = rot_270_matrix * output_01;

    let change_basis_matrix_2d = Matrix2::new(
        output_01.x,
        output_01_perpendicular.x,
        output_01.y,
        output_01_perpendicular.y,
    )
    .try_inverse()
    .unwrap();

    let unchange_basis_matrix = pad_matrix(&Matrix2::new(
        output_01.x,
        output_01_perpendicular.x,
        output_01.y,
        output_01_perpendicular.y,
    ));

    let change_basis_matrix = pad_matrix(&change_basis_matrix_2d);

    // Change triangle bases
    let m = change_basis_matrix * rotation_matrix * translation_matrix;

    let input_1_in_01 = m * pad_vector(&input.1);
    let input_2_in_01 = m * pad_vector(&input.2);
    let output_1_in_01 = m * pad_vector(&output.1);
    let output_2_in_01 = m * pad_vector(&output.2);

    // Find scale/shear matrix
    let scale_01 = output_1_in_01.x / input_1_in_01.x;
    let scale_01_perpendicular = output_2_in_01.y / input_2_in_01.y;
    let shear_01 = (output_2_in_01.x - input_2_in_01.x * scale_01) / output_2_in_01.y;

    let scale_matrix_2d = Matrix2::new(scale_01, shear_01, 0.0, scale_01_perpendicular);
    let scale_matrix = pad_matrix(&scale_matrix_2d);

    // Print out
    println!("Translation matrix: {translation_matrix:?}");
    println!("Rotation matrix: {rotation_matrix:?}");
    println!("Change of basis matrix: {change_basis_matrix:?}");
    println!("Scale/shear matrix {scale_matrix:?}");

    unchange_basis_matrix
        * scale_matrix
        * change_basis_matrix
        * rotation_matrix
        * translation_matrix
}
