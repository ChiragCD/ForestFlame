const HEAP_SIZE: i64 = 1024 * 1024 / 8;

static mut HEAP_START: *const u64 = std::ptr::null();
static mut HEAP_END: *const u64 = std::ptr::null();

use std::env;
#[link(name = "our_code")]
extern "C" {
    // The \x01 here is an undocumented feature of LLVM that ensures
    // it does not add an underscore in front of the name.
    // Courtesy of Max New (https://maxsnew.com/teaching/eecs-483-fa22/hw_adder_assignment.html)
    #[link_name = "\x01our_code_starts_here"]
    fn our_code_starts_here(input: i64, heap_start: *const u64, heap_end: *const u64) -> u64;
    #[link_name = "\x01def_deref"]
    fn deref(ptr: i64) -> i64;
}

#[no_mangle]
#[export_name = "\x01snek_error"]
pub extern "C" fn snek_error(errcode : i64) {
    let s = match errcode {
        1 => "Failed to set up runtime",
        2 => "Ran out of memory",
        3 => "Type mismatch",
        4 => "Bad memory access",
        5 => "Truth value expected (invalid argument)",
        6 => "Numeric value expected (invalid argument)",
        7 => "Arithmetic error (overflow)",
        _ => "An unspecified error occurred",
    };
  eprintln!("Error - {errcode} - {s}");
  std::process::exit(1);
}

fn snek_print_array(ptr: i64) {
    let size = unsafe {deref(ptr+8)} / 8;
    print!("array - size {} -", size);
    for i in 0..size {
        print!(" ");
        snek_print_internal(unsafe {deref(ptr + 16 + i*8)});
    }
}

fn snek_print_link(ptr: i64) {
    print!("link - ");
    snek_print_internal(unsafe {deref(ptr + 8)});
    print!(" ");
    snek_print_internal(unsafe {deref(ptr + 16)});
}

fn snek_print_deref(ptr: i64) {
    print!("(");
    let val = unsafe {deref(ptr)};
    if val == 19 { snek_print_array(ptr); }
    else if val == 11 { snek_print_link(ptr); }
    else { snek_print_internal(val); }
    print!(")");
}

fn snek_print_internal(val: i64) {
    if val == 3 { print!("true"); }
    else if val == 1 { print!("false"); }
    else if val & 5 == 5 { snek_print_deref(val);}
    else if val % 2 == 0 { print!("{}", val >> 1); }
    else { print!("Unknown value: {}", val); }
}

#[no_mangle]
#[export_name = "\x01snek_print"]
pub extern "C" fn snek_print(val : u64) -> u64 {
  snek_print_internal(val as i64);
  println!();
  return val;
}

fn parse_arg(v : &Vec<String>) -> i64 {
  if v.len() < 2 { return 1 }
  let s = &v[1];
  if s == "true" { 3 }
  else if s == "false" { 1 }
  else { s.parse::<i64>().expect("Invalid - cannot accept as number!") << 1 }
}

#[no_mangle]
#[export_name = "\x01create_heap"]
pub extern "C" fn create_heap() -> *mut i64 {
    let mut heap: [i64; HEAP_SIZE as usize] = [0; HEAP_SIZE as usize];
    let heap_ptr = &mut heap as *mut i64;
    return heap_ptr;
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let input = parse_arg(&args);
    let heap_size = if args.len() >= 3 { &args[2] } else { "10000" };
    let heap_size = heap_size.parse::<usize>().unwrap();
    
    // Initialize heap
    let mut heap: Vec<u64> = Vec::with_capacity(heap_size);
    unsafe {
        HEAP_START = heap.as_mut_ptr();
        HEAP_END = HEAP_START.add(heap_size);
    }

    let i : u64 = unsafe { our_code_starts_here(input, HEAP_START, HEAP_END) };
    snek_print(i);
}