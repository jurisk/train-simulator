use fastrand::Rng;

#[must_use]
pub fn generate_random_string(length: usize) -> String {
    let vowels: &[u8] = b"aeiou";
    let consonants: &[u8] = b"bcdfghjklmnpqrstvwxyz";

    let mut rng = Rng::new();
    let mut result = String::with_capacity(length);

    for i in 0 .. length {
        let char_set = if i % 2 == 0 { vowels } else { consonants };
        let c = choose(&mut rng, char_set).unwrap_or(b' ') as char;
        result.push(c);
    }
    result
}

pub fn choose<T: Copy>(rng: &mut Rng, items: &[T]) -> Option<T> {
    if items.is_empty() {
        None
    } else {
        let i = rng.usize(.. items.len());
        Some(items[i])
    }
}
