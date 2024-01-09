
extern crate image;


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
fn color_levels(mut input:Vec<u8>)->Vec<u8>{
    let in_hight:u8=30;
    let in_low:u8=0;
    for element in input.iter_mut() {
        *element = (*element).saturating_add(in_hight).saturating_sub(in_low);
    }

    let input =normalize_array(input);
    input
}
fn main() {
    // Укажите путь к вашему изображению
    let image_path = "/home/umzo/RustroverProjects/untitled/123/333.png";

    // Загрузка изображения
    let img = image::open(image_path).expect("Не удалось загрузить изображение");

    // Преобразование изображения в оттенки серого
    let mut gray_img = img.to_luma8();

    // Получение массива пикселей оттенков серого
    let gray_pixels: Vec<u8> = gray_img.clone().into_raw();
    let gray_pixels=color_levels(gray_pixels);

    gray_img.copy_from_slice(&gray_pixels);
    let _ = gray_img.save("output_path.png");


}