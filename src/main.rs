extern crate image;
use std::{fs};
use std::time::Instant;
use rayon::prelude::*;
use ndarray::{Array2};

fn normalize_array(input: Vec<u8>) -> Vec<u8> {

        // Найти минимальное и максимальное значения
    let min_value = *input.iter().min().unwrap_or(&0);
    let max_value = *input.iter().max().unwrap_or(&255);

    // Нормализовать массив
    let normalized_values: Vec<u8> = input
        .iter()
        .map(|&x| ((x - min_value) as f32 / (max_value - min_value) as f32 * 255.0) as u8)
        .collect();

    normalized_values
}
fn color_levels(mut input:Vec<u8>, in_high:u8, in_low:u8, gamma:f32, out_low:u8,out_high:u8,rows:usize, cols:usize)->Vec<u8>{
    let mut array = Array2::from_shape_vec((rows, cols), input).unwrap();

    array.mapv_inplace(|x| if x + in_high <= u8::MAX.into() { x + in_high } else { u8::MAX });
    array.mapv_inplace(|x| if x >= in_low { x - in_low } else { 0 });
    array.mapv_inplace(|x| {
        let result = (x as f32 * gamma).round() as u8;
        if result > u8::MAX {
            u8::MAX
        } else {
            result
        }
    });
    let max_value = array.fold(0, |acc, &x| acc.max(x));
    array.mapv_inplace(|x| ((x as f32 / max_value as f32) * 255.0).round() as u8);

    array.mapv_inplace(|x| if x + out_high <= u8::MAX.into() { x + out_high } else { u8::MAX });
    array.mapv_inplace(|x| if x >= out_low { x - out_low } else { 0 });
    let input :Vec<u8> = array.into_raw_vec();
    input
}
fn procces_images(path2:&str,output_path:&str){
    let img = image::open(path2).expect("Не удалось загрузить изображение");

    // Преобразование изображения в оттенки серого
    let mut gray_img = img.to_luma8();
    let width = img.width() as usize;
    let height = img.height() as usize;

    // Получение массива пикселей оттенков серого
    let gray_pixels: Vec<u8> = gray_img.clone().into_raw();
    let in_high: u8 = 0;
    let in_low: u8 = 0;
    let gamma: f32 = 1.0;
    let out_low: u8 = 0;
    let out_high: u8 = 255;
    let out_high2: u8 = 255 - out_high;
    let gray_pixels = color_levels(gray_pixels, in_high, in_low, gamma, out_low, out_high2,width,height);

    gray_img.copy_from_slice(&gray_pixels);
    let _ = gray_img.save(output_path);
}
fn main() {
    let input_directory = "123";
    let output_directory = "OUTPUT";
    let start = Instant::now();

    if let Ok(paths) = fs::read_dir(input_directory) {
        paths.par_bridge().for_each(|path| {
            if let Ok(entry) = path {
                let path2 = entry.path();
                if path2.is_file() && path2.extension().is_some() {
                    let output_path = format!(
                        "{}/{}",
                        output_directory,
                        path2.file_name().expect("Failed to get file name").to_string_lossy()
                    );
                    procces_images(path2.to_str().expect("Failed to convert path to string"), &output_path);
                }
            }
        });
    }

    let duration = start.elapsed();
    println!("Time elapsed in main() is: {:?}", duration);
}