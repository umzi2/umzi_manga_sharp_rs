use std::collections::HashMap;
use indicatif::{ProgressBar, ProgressStyle};
use std::{fs};
use image::GrayImage;
use rayon::prelude::*;
use ndarray::{Array2, Zip};
use imageproc::filter::{median_filter, gaussian_blur_f32};
use imageproc::edges::canny;
use std::env;
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

fn add_u8_arrays(array1: &Array2<u8>, array2: &Array2<u8>) -> Array2<u8> {
    // Используем Zip для сложения элементов массивов
    let result = Zip::from(array1)
        .and(array2)
        .map_collect(|&x, &y| x.saturating_add(y));

    // Возвращаем результат как новый массив
    result
}

fn sub_u8_arrays(array1: &Array2<u8>, array2: &Array2<u8>) -> Array2<u8> {
    // Используем Zip для сложения элементов массивов
    let result = Zip::from(array1)
        .and(array2)
        .map_collect(|&x, &y| x.saturating_sub(y));

    // Возвращаем результат как новый массив
    result
}

fn threshold(mut image: GrayImage, threshold_value: u8) -> GrayImage {
    let width = image.width() as usize;
    let height = image.height() as usize;
    let input: Vec<u8> = image.clone().into_raw();
    let mut array = Array2::from_shape_vec((width, height), input).unwrap();
    array.mapv_inplace(|pixel| if pixel > threshold_value { 255 } else { 0 });
    let input_vec: Vec<u8> = array.into_raw_vec();
    image.copy_from_slice(&input_vec);
    image
}

fn color_levels(mut image: GrayImage, scores: HashMap<&str, u8>, gamma: f32) -> GrayImage {
    let width = image.width() as usize;
    let height = image.height() as usize;
    let input: Vec<u8> = image.clone().into_raw();
    let mut array = Array2::from_shape_vec((width, height), input).unwrap();

    array.mapv_inplace(|x| x.saturating_add(255 - scores["in_high"]));
    array.mapv_inplace(|x| x.saturating_sub(scores["in_low"]));
    if gamma != 1.0 {
        array.mapv_inplace(|x| {
            let result = (x as f32 * gamma).round() as u8;
            if result > u8::MAX {
                u8::MAX
            } else {
                result
            }
        });
    }
    let max_value = array.fold(0, |acc, &x| acc.max(x));
    array.mapv_inplace(|x| ((x as f32 / max_value as f32) * 255.0).round() as u8);
    if scores["out_high"] != 255 {
        array.mapv_inplace(|x| x.saturating_add(255 - scores["out_high"]));
    }
    if scores["out_low"] != 0 {
        array.mapv_inplace(|x| x.saturating_sub(scores["out_low"]));
    }


    let input_vec: Vec<u8> = array.into_raw_vec();
    image.copy_from_slice(&input_vec);
    image
}

fn masc(mut image: GrayImage, mask: GrayImage, add: bool, inverted: bool) -> GrayImage {//mut image: GrayImage,mask: GrayImage
    let input: Vec<u8> = image.clone().into_raw();
    let input_masc: Vec<u8> = mask.clone().into_raw();
    let width = image.width() as usize;
    let height = image.height() as usize;
    let array = Array2::from_shape_vec((width, height), input).unwrap();
    let mut array_masc = Array2::from_shape_vec((width, height), input_masc).unwrap();
    if inverted {
        array_masc.mapv_inplace(|pixel| 255 - pixel);
    }
    let result;
    if add {
        result = add_u8_arrays(&array, &array_masc);
    } else {
        result = sub_u8_arrays(&array, &array_masc);
    }
    let input_vec: Vec<u8> = result.into_raw_vec();
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
    let mut gray_img = color_levels(gray_img, scores.clone(), gamma);
    if config.cenny != 0 || config.diapason_white != -1 {
        let gray_img_median = median_filter(&gray_img, 1, 1);

        if config.diapason_white != -1 {
            let gray_img_median = median_filter(&gray_img, 1, 1);
            let mut scores2 = HashMap::new();
            scores2.insert("in_high", 255 - config.diapason_white as u8);
            scores2.insert("in_low", 255 - config.diapason_white as u8);
            scores2.insert("out_low", 0);
            scores2.insert("out_high", 255);
            let white_masc = color_levels(gray_img_median.clone(), scores2, 1.0);
            gray_img = masc(gray_img, white_masc, true, false)
        }
        if config.cenny != 0 {
            let cenny = canny(&gray_img_median, 450.0, 500.0);
            gray_img = masc(gray_img, cenny, false, false);
        }
    }
    if config.diapason_black != -1 {
        let img_thereshold = threshold(gray_img.clone(), config.diapason_black as u8);
        let blur_image = gaussian_blur_f32(&img_thereshold, 1.0 as f32);
        let black_masc = threshold(blur_image.clone(), 170);
        // let black_masc=;
        gray_img = masc(gray_img, black_masc, false, true)
    }

    let _ = gray_img.save(output_path);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let input_directory;
    let output_directory;
    if args.len() != 5 {
        input_directory = "INPUT";
        output_directory = "OUTPUT"
    } else {
        input_directory = &args[2];
        output_directory = &args[4];
    }
    if !fs::metadata(output_directory).is_ok() {
        match fs::create_dir(output_directory) {
            Ok(_) => println!("Папка успешно создана: {}", output_directory),
            Err(e) => println!("Ошибка при создании папки: {}", e),
        }
    } else {
        println!("Папка уже существует: {}", output_directory);
    }

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
