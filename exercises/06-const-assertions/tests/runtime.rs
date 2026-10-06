use const_assertions::{RingBuffer, hex_color};

#[test]
fn ring_buffer_wraps() {
    let mut ring = RingBuffer::<u32, 4>::new();
    assert!(ring.is_empty());
    for i in 0..6 {
        ring.push(i);
    }
    assert_eq!(ring.len(), 4);
    let contents: Vec<u32> = (0..ring.len()).map(|i| *ring.get(i).unwrap()).collect();
    assert_eq!(contents, [2, 3, 4, 5]);
    assert_eq!(ring.get(4), None);
}

#[test]
fn hex_colors() {
    assert_eq!(hex_color("#000000"), 0);
    assert_eq!(hex_color("#ffffff"), 0xffffff);
    assert_eq!(hex_color("#FF7F50"), 0xff7f50);
    assert_eq!(hex_color("#1a2B3c"), 0x1a2b3c);
}

#[test]
#[should_panic]
fn hex_color_rejects_garbage_at_runtime_too() {
    let input = String::from("#nope!!");
    hex_color(&input);
}
