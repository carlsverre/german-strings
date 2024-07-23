use std::ptr::null;

const PREFIX_LEN: usize = 4;
const SHORT_LEN: usize = 12;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct ShortString {
    len: u32,
    buf: [u8; SHORT_LEN],
}
static_assertions::assert_eq_size!(ShortString, &str);

impl ShortString {
    #[inline]
    fn prefix(&self) -> &str {
        let len = PREFIX_LEN.min(self.len as usize);
        unsafe {
            let prefix = self.buf.get_unchecked(..len);
            std::str::from_utf8_unchecked(prefix)
        }
    }

    #[inline]
    fn str(&self) -> &str {
        // SAFETY: `self.len` is always less than or equal to `SHORT_LEN`
        // and `self.buf` is always a valid UTF-8 string.
        debug_assert!(self.len as usize <= SHORT_LEN);
        unsafe {
            let str = self.buf.get_unchecked(..self.len as usize);
            std::str::from_utf8_unchecked(str)
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct LongString {
    len: u32,
    prefix: [u8; 4],
    heap: *const u8,
}
static_assertions::assert_eq_size!(LongString, &str);

impl LongString {
    #[inline]
    fn str(&self) -> &str {
        // SAFETY: `self.len` is always greater than `SHORT_LEN`
        // and `self.heap` is always a valid UTF-8 string.
        debug_assert!(self.len as usize > SHORT_LEN);
        unsafe {
            let slice = std::slice::from_raw_parts(self.heap, self.len as usize);
            std::str::from_utf8_unchecked(slice)
        }
    }
}

#[repr(C)]
pub union GermanString {
    short: ShortString,
    long: LongString,
}
static_assertions::assert_eq_size!(GermanString, &str);

impl Drop for GermanString {
    fn drop(&mut self) {
        if !self.is_short() {
            unsafe {
                let long = &mut self.long;
                if !long.heap.is_null() {
                    drop(Box::from_raw(long.heap as *mut u8));
                    long.heap = null();
                }
            }
        }
    }
}

impl GermanString {
    pub fn new(s: &str) -> Self {
        let len: u32 = s.len().try_into().expect("length too long");
        if len <= SHORT_LEN as u32 {
            let mut short = ShortString {
                len,
                buf: [0; SHORT_LEN],
            };
            short.buf[..len as usize].copy_from_slice(s.as_bytes());
            Self { short }
        } else {
            let mut long = LongString {
                len,
                prefix: [0; PREFIX_LEN],
                heap: std::ptr::null(),
            };
            long.prefix.copy_from_slice(&s.as_bytes()[..PREFIX_LEN]);
            // copy str to the heap and leak into a ptr
            let heap = Box::leak(s.as_bytes().to_vec().into_boxed_slice());
            long.heap = heap.as_ptr();
            Self { long }
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        // SAFETY: the length is in the same location for both variants
        unsafe { self.short.len as usize }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn is_short(&self) -> bool {
        self.len() <= SHORT_LEN
    }

    #[inline]
    pub fn prefix(&self) -> &str {
        // SAFETY: the prefix is in the same location for both variants
        unsafe { self.short.prefix() }
    }

    #[inline]
    pub fn str(&self) -> &str {
        if self.is_short() {
            unsafe { self.short.str() }
        } else {
            unsafe { self.long.str() }
        }
    }

    #[inline]
    pub fn starts_with(&self, needle: &str) -> bool {
        // early return if the needle doesn't match the interned prefix
        unsafe {
            let len = PREFIX_LEN.min(needle.len());
            let needle_prefix = needle.get_unchecked(..len);
            if !self.prefix().starts_with(needle_prefix) {
                return false;
            }
        }

        // if needle is longer than the prefix, check the rest of the string
        if needle.len() > PREFIX_LEN {
            self.str().starts_with(needle)
        } else {
            true
        }
    }
}

impl From<String> for GermanString {
    fn from(str: String) -> Self {
        let len: u32 = str.len().try_into().expect("length too long");
        if len <= SHORT_LEN as u32 {
            let mut short = ShortString {
                len,
                buf: [0; SHORT_LEN],
            };
            short.buf[..len as usize].copy_from_slice(str.as_bytes());
            Self { short }
        } else {
            let mut prefix = [0; PREFIX_LEN];
            prefix.copy_from_slice(&str.as_bytes()[..PREFIX_LEN]);
            let heap = Box::leak(str.into_bytes().into_boxed_slice());
            Self {
                long: LongString {
                    len,
                    prefix,
                    heap: heap.as_ptr(),
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_german_string() {
        let s = GermanString::new("");
        assert_eq!(s.len(), 0);
        assert!(s.is_empty());
        assert!(s.is_short());
        assert_eq!(s.prefix(), "");
        assert_eq!(s.str(), "");
        assert!(!s.starts_with("a"));
        assert!(s.starts_with(""));

        let s = GermanString::new("Hallo, Welt!");
        assert_eq!(s.len(), 12);
        assert!(s.is_short());
        assert_eq!(s.prefix(), "Hall");
        assert_eq!(s.str(), "Hallo, Welt!");
        assert!(s.starts_with(""));
        assert!(s.starts_with("H"));
        assert!(s.starts_with("Hallo"));
        assert!(s.starts_with("Hallo, Welt!"));
        assert!(!s.starts_with("Hallo, Welt! "));
        assert!(!s.starts_with("F"));

        let s = GermanString::new("Hello, World!");
        assert_eq!(s.len(), 13);
        assert!(!s.is_short());
        assert_eq!(s.prefix(), "Hell");
        assert_eq!(s.str(), "Hello, World!");
        assert!(s.starts_with(""));
        assert!(s.starts_with("H"));
        assert!(s.starts_with("Hello"));
        assert!(s.starts_with("Hello, World!"));
        assert!(!s.starts_with("Hello, World! "));
        assert!(!s.starts_with("F"));

        let s: String = "Hallo, Welt!".into();
        let s: GermanString = s.into();
        assert_eq!(s.len(), 12);
        assert!(s.is_short());
        assert_eq!(s.prefix(), "Hall");
        assert_eq!(s.str(), "Hallo, Welt!");
        assert!(s.starts_with(""));
        assert!(s.starts_with("H"));
        assert!(s.starts_with("Hallo"));
        assert!(s.starts_with("Hallo, Welt!"));
        assert!(!s.starts_with("Hallo, Welt! "));
        assert!(!s.starts_with("F"));
    }
}
