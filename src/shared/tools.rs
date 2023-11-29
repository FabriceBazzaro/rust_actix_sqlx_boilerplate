use rand::Rng;

pub fn generate_string_number(size: u8) -> String {
    let mut str: String = "".to_owned();
    let mut rng = rand::thread_rng();

    for _ in 0..size {
        str.push_str(&rng.gen_range(0..10).to_string());
    }
    str
}
