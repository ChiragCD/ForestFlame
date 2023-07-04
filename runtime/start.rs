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


/*
/// This function should trigger garbage collection and return the updated heap pointer (i.e., the new
/// value of `%r15`). See [`snek_try_gc`] for a description of the meaning of the arguments.
#[export_name = "\x01snek_gc"]
pub unsafe fn snek_gc(
    heap_ptr: *const u64,
    stack_base: *const u64,
    curr_rbp: *const u64,
    curr_rsp: *const u64,
) -> *const u64 {
    // print_heap();
    // snek_print_stack(stack_base, curr_rbp, curr_rsp);
    let mut base_set = HashSet::new();
    let mut seen_set = HashSet::new();
    gc_stack(stack_base, curr_rsp, &mut base_set, false);
    // println!("Got base set!");
    // println!("{:?}", base_set);
    for pointer in base_set.iter() {snek_str(*pointer, &mut HashSet::new(), &mut seen_set);}
    // println!("{:?}", seen_set);
    let mut new_addr = HEAP_START;
    let mut relocator_reversed = HashMap::new();
    let mut seen_list = seen_set.iter().map(|i| *i).collect::<Vec<u64>>();
    seen_list.sort();
    for pointer in seen_list.iter() {
        let old_addr = (*pointer - 1) as *mut u64;
        // println!("Moving {} to {}", old_addr as usize, new_addr as usize);
        old_addr.write(new_addr as u64);
        relocator_reversed.insert(new_addr as u64, old_addr as u64);
        new_addr = new_addr.add(old_addr.add(1).read() as usize + 2);
        // println!("Updating new addr to {} for size {}", new_addr as usize, old_addr.add(1).read() as usize);
    }
    let updated_heap_ptr = new_addr;
    for pointer in seen_list.iter() {
        let addr = (*pointer - 1) as *mut u64;
        let size = addr.add(1).read() as usize;
        for i in 0..size {
            let val = addr.add(2 + i).read() as u64;
            if val % 4 == 1 && val != 1 {
                // println!("Updating a ref! {} {} {}", addr as u64, i, size);
                let loc = (val - 1) as *const u64;
                addr.add(2 + i).write(loc.read() + 1);
            }
        }
    }
    gc_stack(stack_base, curr_rsp, &mut base_set, true);
    let mut locations = relocator_reversed.keys().map(|i| *i as u64).collect::<Vec<u64>>();
    locations.sort();
    // println!("HI");
    // println!("{:?}", locations);
    for location in locations.iter() {
        // println!("{} from {}", location, relocator_reversed[&(location)]);
        let new_addr = *location as *mut u64;
        let old_addr = relocator_reversed[&(location)] as *const u64;
        let size = old_addr.add(1).read() as usize;
        new_addr.write(0);
        for i in 0..(size+1) {
            new_addr.add(1 + i).write(old_addr.add(1 + i).read());
        }
        // println!("Relocated {}!", size);
    }
    updated_heap_ptr
}
 */
