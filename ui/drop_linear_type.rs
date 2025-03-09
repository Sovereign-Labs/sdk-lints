use nearly_linear::{DropGuard, DropWarning};

fn drop_linear_type() {
    let mut x = DropGuard::new(0u8);
    //~^ drop_linear_type
    *x += 1;
}

fn consume_linear_type() {
    let mut x = DropGuard::new(0u8);
    *x += 1;
    x.done();
}

fn main() {
	consume_linear_type();
    drop_linear_type();
}
