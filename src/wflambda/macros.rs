#![allow(unused_macros)]

macro_rules! max {
    ($x: expr) => ($x);
    ($x: expr, $($z: expr),+) => {{
        let y = max!($($z),*);
        if $x > y {
            $x
        } else {
            y
        }
    }}
}

macro_rules! min {
    ($x: expr) => ($x);
    ($x: expr, $($z: expr),+) => {{
        let y = min!($($z),*);
        if $x < y {
            $x
        } else {
            y
        }
    }}
}

macro_rules! abs {
    ($x: expr) => {{
        if $x >= 0 {
            $x
        } else {
            -$x
        }
    }};
}

macro_rules! ewavefront_diagonal {
    ($h: expr, $v: expr) => {{
        $h - $v
    }};
}

macro_rules! ewavefront_offset {
    ($h: expr, $v: expr) => {{
        $h
    }};
}

macro_rules! ewavefront_v {
    ($k: expr, $offset: expr) => {{
        $offset - $k
    }};
}

macro_rules! ewavefront_h {
    ($k: expr, $offset: expr) => {{
        $offset
    }};
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_max() {
        assert_eq!(4, max!(1, 2, 3, 4));
        assert_eq!(43, max!(9, 2, 43, 4))
    }

    #[test]
    fn test_min() {
        assert_eq!(1, min!(1, 2, 3, 4));
        assert_eq!(2, min!(9, 2, 43, 4))
    }
}
