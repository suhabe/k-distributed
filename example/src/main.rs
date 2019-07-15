struct Transaction {
    value: u32
}

fn main() {
    exec(|task| {
        task.value = 5;
        println!("result {}", task.value);
    });
}

fn exec(task : fn(&mut Transaction)) {

    let mut trans = Transaction {
        value: 0
    };

    task(&mut trans);
}
