fn as_str(data: &u32) -> &str {
    let s = format!("{}", data);
    &s
}

fn main() {
    let x = 10;
    println!("{}", as_str(&x));
}