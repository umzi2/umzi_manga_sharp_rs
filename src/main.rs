use std::collections::HashMap;
use indicatif::{ProgressBar, ProgressStyle};
use std::{fs};
use image::GrayImage;
use ndarray::Array2;
use imageproc::edges::canny;
use imageproc::filter::gaussian_blur_f32;
use std::env;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;

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
    if let Err(err) = file.read_to_string(&mut file_content) {
        panic!("Не удалось считать содержимое файла {}: {}", file_path, err);
    }

    // Распарсим JSON-строку с использованием serde_json
    match serde_json::from_str(&file_content) {
        Ok(config) => config,
        Err(err) => {
            panic!("Не удалось распарсить JSON: {}", err);
        }
    }
}

fn array2_to_gray_image(input: Array2<f32>) -> GrayImage {
    let max_pixel_value = u8::max_value() as f32;
    let scaled_input = input.mapv(|pixel| (pixel * max_pixel_value) as u8);
    let input_vec: Vec<u8> = scaled_input.iter().cloned().collect();
    GrayImage::from_vec(input.shape()[0] as u32, input.shape()[1] as u32, input_vec).unwrap()
}

fn gray_image_to_array2(input: GrayImage) -> Array2<f32> {
    let input_data: Vec<f32> = input.clone().into_raw().iter().map(|&pixel| f32::from(pixel) / u8::max_value() as f32).collect();
    Array2::from_shape_vec((input.width() as usize, input.height() as usize), input_data).unwrap()
}

fn threshold(input: Array2<f32>, threshold_value: f32) -> Array2<f32> {
    let mut output = Array2::zeros(input.raw_dim());

    // Применяем пороговое значение к каждому пикселю входного массива
    output.assign(&input.mapv(|pixel| if pixel > threshold_value { 1.0 } else { 0.0 }));

    output
}

fn color_levels_f32(image: GrayImage, scores: HashMap<&str, f32>) -> Array2<f32> {
    let width = image.width() as usize;
    let height = image.height() as usize;
    let input = image.iter().map(|&x| x as f32 / 255.0).collect();
    let mut array = Array2::from_shape_vec((width, height), input).unwrap();
    array.mapv_inplace(|x| f32::min(1.0, x + (1.0 - scores["in_high"])));
    array.mapv_inplace(|x| f32::max(0.0, x - scores["in_low"]));
    if scores["gamma"] != 1.0 {
        array.mapv_inplace(|x| f32::min(1.0, x * scores["gamma"]));
    }

    let min_value = array.iter().cloned().fold(f32::INFINITY, f32::min);
    let max_value = array.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

    array.mapv_inplace(|x| (x - min_value) / (max_value - min_value));
    if scores["out_high"] != 255.0 {
        array.mapv_inplace(|x| f32::min(1.0, x + (1.0 - scores["out_high"])));
    }
    if scores["out_low"] != 0.0 {
        array.mapv_inplace(|x| f32::max(0.0, x - scores["out_low"]));
    }
    array
}

fn masc(image: Array2<f32>, mask: Array2<f32>, add: bool, inverted: bool) -> Array2<f32> {
    let mut array_masc = mask.to_owned();

    if inverted {
        array_masc.mapv_inplace(|pixel| 1.0 - pixel);
    }

    let result = if add {
        add_f32_arrays(image, &array_masc)
    } else {
        sub_f32_arrays(image, &array_masc)
    };

    result
}

fn add_f32_arrays(mut image: Array2<f32>, mask: &Array2<f32>) -> Array2<f32> {
    image.zip_mut_with(mask, |pixel, &mask_pixel| *pixel += mask_pixel);
    image.to_owned()
}

fn sub_f32_arrays(mut image: Array2<f32>, mask: &Array2<f32>) -> Array2<f32> {
    image.zip_mut_with(mask, |pixel, &mask_pixel| *pixel -= mask_pixel);
    image.to_owned()
}

fn process_images(path2: &str, output_path: &str) {
    let gray_img = match image::open(path2) {
        Ok(image) => image.to_luma8(),
        Err(err) => {
            eprintln!("Не удалось загрузить изображение {}: {}", path2, err);
            return;
        }
    };

    let config = &*CONFIG;
    let mut scores = HashMap::new();
    scores.insert("in_high", config.high_input as f32 / 255.0);
    scores.insert("in_low", config.low_input as f32 / 255.0);
    scores.insert("out_low", config.low_output as f32 / 255.0);
    scores.insert("out_high", config.high_output as f32 / 255.0);
    scores.insert("gamma", config.gamma);

    let mut gray_array = color_levels_f32(gray_img, scores.clone());
    let gray_img = array2_to_gray_image(gray_array.clone());
    let gray_img_blur = gaussian_blur_f32(&gray_img, 1.4);
    let gray_img_blur = gray_image_to_array2(gray_img_blur);

    if config.cenny != 0 || config.diapason_white != -1 {
        if config.diapason_white != -1 {
            let white_masc = threshold(gray_img_blur.clone(), config.diapason_white as f32 / 255.0);
            gray_array = masc(gray_array.clone(), white_masc, true, false);
        }

        if config.cenny != 0 {
            let gray_img = array2_to_gray_image(gray_array.clone());
            let cenny = canny(&gray_img, 450.0, 500.0);
            let cenny = gray_image_to_array2(cenny);
            gray_array = masc(gray_array.clone(), cenny, false, false);
        }
    }

    if config.diapason_black != -1 {
        let img_threshold = threshold(gray_array.clone(), config.diapason_black as f32 / 255.0);
        let gray_img = array2_to_gray_image(img_threshold);
        let gray_img_blur = gaussian_blur_f32(&gray_img, 1.4);
        let blur_image = gray_image_to_array2(gray_img_blur);
        let black_masc = threshold(blur_image.clone(), 170.0 / 255.0);
        gray_array = masc(gray_array.clone(), black_masc, false, true);
    }

    let gray_img = array2_to_gray_image(gray_array.clone());

// Добавим проверку наличия расширения и установим его в ".png"
    let output_path_with_extension = if output_path.ends_with(".png") {
        output_path.to_owned()
    } else {
        format!("{}.png", output_path)
    };

    if let Err(err) = gray_img.save(&output_path_with_extension) {
        eprintln!("Ошибка при сохранении изображения {}: {}", output_path_with_extension, err);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let input_directory;
    let output_directory;

    if args.len() != 5 {
        input_directory = "123";
        output_directory = "OUTPUT";
    } else {
        input_directory = &args[2];
        output_directory = &args[4];
    }

    if let Err(_e) = fs::metadata(output_directory) {
        match fs::create_dir(output_directory) {
            Ok(_) => println!("Папка успешно создана: {}", output_directory),
            Err(e) => eprintln!("Ошибка при создании папки: {}", e),
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
    } else {
        eprintln!("Ошибка при чтении содержимого директории: {}", input_directory);
    }
}
