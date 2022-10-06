use std::ptr;
fn main() {
    let mut s = Vec::new();
    let sptr = s.as_ptr();
    
    if sptr.is_null() {
        println!("its null");
    } else {
        println!("something else");
    }
    
    s.push(1);
}