use fastrand::Rng;

#[must_use]
pub fn generate_random_string(length: usize, seed: u64) -> String {
    let vowels: &[u8] = b"aeiou";
    let consonants: &[u8] = b"bcdfghjklmnpqrstvwxyz";

    let mut rng = Rng::with_seed(seed);
    let mut result = String::with_capacity(length);

    for i in 0 .. length {
        let char_set = if i % 2 == 0 { vowels } else { consonants };
        let c = *choose_with_rng(&mut rng, char_set).unwrap_or(&b' ') as char;
        let c = if i == 0 { c.to_ascii_uppercase() } else { c };
        result.push(c);
    }
    result
}

pub fn choose_with_rng<'a, T>(rng: &'a mut Rng, items: &'a [T]) -> Option<&'a T> {
    if items.is_empty() {
        None
    } else {
        let i = rng.usize(.. items.len());
        items.get(i)
    }
}

pub fn choose<T>(items: &[T]) -> Option<&T> {
    if items.is_empty() {
        None
    } else {
        let i = fastrand::usize(.. items.len());
        items.get(i)
    }
}
