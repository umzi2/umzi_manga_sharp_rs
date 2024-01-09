extern crate image;

use indicatif::{ProgressBar, ProgressStyle};
use std::{fs};
use rayon::prelude::*;
use ndarray::{Array2};

fn color_levels(input: Vec<u8>, in_high: u8, in_low: u8, gamma: f32, out_low: u8, out_high: u8, rows: usize, cols: usize) -> Vec<u8> {
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
    let input: Vec<u8> = array.into_raw_vec();
    input
}

fn process_images(path2: &str, output_path: &str) {
    let mut gray_img = image::open(path2).expect("Не удалось загрузить изображение").grayscale().to_luma8();

    // Преобразование изображения в оттенки серого
    let width = gray_img.width() as usize;
    let height = gray_img.height() as usize;

    // Получение массива пикселей оттенков серого
    let gray_pixels: Vec<u8> = gray_img.clone().into_raw();
    let in_high: u8 = 0;
    let in_low: u8 = 0;
    let gamma: f32 = 1.0;
    let out_low: u8 = 0;
    let out_high: u8 = 255;
    let out_high2: u8 = 255 - out_high;
    let gray_pixels = color_levels(gray_pixels, in_high, in_low, gamma, out_low, out_high2, width, height);

    gray_img.copy_from_slice(&gray_pixels);
    let _ = gray_img.save(output_path);
}

fn main() {
    let input_directory = "123";
    let output_directory = "OUTPUT";

    if let Ok(paths) = fs::read_dir(input_directory) {
        // Клонирование ReadDir для избежания перемещения
        let paths = paths.collect::<Vec<_>>();

        // Получение общего количества файлов для прогресс-бара
        let total_files = paths.len() as u64;

        // Создание прогресс-бара
        let progress_bar = ProgressBar::new(total_files);
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({eta})").expect("REASON")
                .progress_chars("##-"),
        );

        paths.par_iter().for_each(|path| {
            if let Ok(entry) = path {
                let path2 = entry.path();
                if path2.is_file() && path2.extension().is_some() {
                    let output_path = format!(
                        "{}/{}",
                        output_directory,
                        path2.file_name().expect("Failed to get file name").to_string_lossy()
                    );
                    process_images(path2.to_str().expect("Failed to convert path to string"), &output_path);

                    // Обновление прогресс-бара
                    progress_bar.inc(1);
                }
            }
        });

        // Завершение прогресс-бара
        progress_bar.finish();
    }
}