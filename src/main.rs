use std::collections::HashMap;
use indicatif::{ProgressBar, ProgressStyle};
use std::{fs};
use image::GrayImage;
use rayon::prelude::*;
use ndarray::{Array2};
//use imageproc::filter::median_filter;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    low_input: u8,
    high_input: u8,
    low_output: u8,
    high_output: u8,
    gamma: f32,
    diapason_black: i16,
    diapason_white: i16,
    cenny: u8,
}

static CONFIG: Lazy<Config> = Lazy::new(|| load_config());

fn load_config() -> Config {
    // Указываем путь к JSON-файлу
    let file_path = "config.json";

    // Открываем файл
    let mut file = match File::open(&file_path) {
        Ok(file) => file,
        Err(err) => {
            panic!("Не удалось открыть файл {}: {}", file_path, err);
        }
    };

    // Создаем строку, в которую будем считывать содержимое файла
    let mut file_content = String::new();

    // Считываем содержимое файла в строку
    match file.read_to_string(&mut file_content) {
        Ok(_) => {
            // Распарсим JSON-строку с использованием serde_json
            match serde_json::from_str(&file_content) {
                Ok(config) => config,
                Err(err) => {
                    panic!("Не удалось распарсить JSON: {}", err);
                }
            }
        }
        Err(err) => {
            panic!("Не удалось считать содержимое файла {}: {}", file_path, err);
        }
    }
}

fn color_levels(mut image: GrayImage, scores: HashMap<&str, u8>, gamma: f32) -> GrayImage {
    let width = image.width() as usize;
    let height = image.height() as usize;
    let input: Vec<u8> = image.clone().into_raw();
    let mut array = Array2::from_shape_vec((width, height), input).unwrap();

    array.mapv_inplace(|x| x.saturating_add(255 - scores["in_high"]));
    array.mapv_inplace(|x| x.saturating_sub(scores["in_low"]));
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

    array.mapv_inplace(|x| x.saturating_add(255 - scores["out_high"]));
    array.mapv_inplace(|x| x.saturating_sub(scores["out_low"]));
    let input_vec: Vec<u8> = array.into_raw_vec();
    image.copy_from_slice(&input_vec);
    image
}

fn process_images(path2: &str, output_path: &str) {
    let gray_img = image::open(path2).expect("Не удалось загрузить изображение").to_luma8();
    let config = &*CONFIG;
    let gamma: f32 = 1.0;
    let mut scores = HashMap::new();
    scores.insert("in_high", config.high_input);
    scores.insert("in_low", config.low_input);
    scores.insert("out_low", config.low_output);
    scores.insert("out_high", config.high_output);

    let gray_img = color_levels(gray_img, scores, gamma);
    //let gray_img = median_filter(&gray_img, 2, 2);

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