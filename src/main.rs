extern crate image;
use std::{cmp,fs};

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
fn color_levels(mut input:Vec<u8>, in_high:u8, in_low:u8, gamma:f32, out_low:u8,out_high:u8)->Vec<u8>{

    for element in input.iter_mut() {
        *element = (*element).saturating_add(in_high).saturating_sub(in_low);
        *element = cmp::min(255, ((*element as f32) * (gamma)) as u8)
    }

    let mut input =normalize_array(input);
    for element in input.iter_mut() {
        *element = (*element).saturating_add(out_high).saturating_sub(out_low);
    }
    input
}
fn procces_images(path2:&str,output_path:&str){
    let img = image::open(path2).expect("Не удалось загрузить изображение");

    // Преобразование изображения в оттенки серого
    let mut gray_img = img.to_luma8();

    // Получение массива пикселей оттенков серого
    let gray_pixels: Vec<u8> = gray_img.clone().into_raw();
    let in_high: u8 = 0;
    let in_low: u8 = 0;
    let gamma: f32 = 1.0;
    let out_low: u8 = 0;
    let out_high: u8 = 255;
    let out_high2: u8 = 255 - out_high;
    let gray_pixels = color_levels(gray_pixels, in_high, in_low, gamma, out_low, out_high2);

    gray_img.copy_from_slice(&gray_pixels);
    let _ = gray_img.save(output_path);
}
fn main() {
    // Укажите путь к вашему изображению

    let input_directory = "123";
    let output_directory = "OUTPUT"; // Создайте эту директорию заранее или измените путь на существующую

    if let Ok(paths) = fs::read_dir(input_directory) {
        for path in paths {
            if let Ok(entry) = path {
                let path2 = entry.path();
                if path2.is_file() && path2.extension().is_some() {
                    let output_path = format!(
                        "{}/{}",
                        output_directory,
                        path2.file_name().expect("Failed to get file name").to_string_lossy()
                    );
                    // Загрузка изображения
                    procces_images(path2.to_str().expect("Failed to convert path to string"),&output_path)

                }
            }
        }
    }
}