use pdf_writer::{IndirectGuard, Name, Null, Obj, PdfWriter, Ref, Str};

/// Test that `buf` is the same as the result of concatenating the strings.
macro_rules! test {
    ($buf:expr, $($expected:literal),* $(,)?) => {{
        let buf = $buf;
        let string = std::str::from_utf8(&buf).unwrap();
        assert_eq!(string, concat!($($expected),*));
    }}
}

/// Test how an object is written.
macro_rules! test_obj {
    (|$obj:ident| $write:expr, $($tts:tt)*) => {{
        test!(slice_obj(|$obj| { $write; }), $($tts)*)
    }}
}

/// Test how a primitive object is written.
macro_rules! test_primitive {
    ($value:expr, $($tts:tt)*) => {
        test_obj!(|obj| obj.primitive($value), $($tts)*);
    }
}

/// Return the slice of bytes written during the execution of `f`.
fn slice<F>(f: F) -> Vec<u8>
where
    F: FnOnce(&mut PdfWriter),
{
    let mut w = PdfWriter::new(1, 7);
    let start = w.len();
    f(&mut w);
    let end = w.len();
    let buf = w.finish(Ref::new(1));
    buf[start .. end].to_vec()
}

/// Return the slice of bytes written for an object.
fn slice_obj<F>(f: F) -> Vec<u8>
where
    F: FnOnce(Obj<IndirectGuard>),
{
    let buf = slice(|w| f(w.indirect(Ref::new(1))));
    buf[8 .. buf.len() - 9].to_vec()
}

#[test]
fn test_minimal() {
    let w = PdfWriter::new(1, 7);
    test!(
        w.finish(Ref::new(1)),
        "%PDF-1.7\n\n",
        "xref\n0 1\n0000000000 65535 f\r\n",
        "trailer\n",
        "<<\n/Size 1\n/Root 1 0 R\n>>\n",
        "startxref\n10\n%%EOF",
    );
}

#[test]
#[should_panic(expected = "unfinished object")]
#[allow(unused_must_use)]
fn test_object_unused() {
    let mut w = PdfWriter::new(1, 7);
    w.indirect(Ref::new(1));
    w.finish(Ref::new(1));
}

#[test]
fn test_primitive_objects() {
    test_primitive!(true, "true");
    test_primitive!(false, "false");
    test_primitive!(78, "78");
    test_primitive!(4.22, "4.22");
    test_primitive!(Str(b"hello"), "(hello)");
    test_primitive!(Name(b"Filter"), "/Filter");
    test_primitive!(Ref::new(7), "7 0 R");
    test_primitive!(Null, "null");
}

#[test]
fn test_arrays() {
    test_obj!(|obj| obj.array(), "[]");
    test_obj!(|obj| obj.array().item(12).item(Null), "[12 null]");
    test_obj!(|obj| obj.array().typed().items(vec![1, 2, 3]), "[1 2 3]");
    test_obj!(
        |obj| {
            let mut array = obj.array();
            array.obj().array().typed().items(vec![1, 2]);
            array.item(3);
        },
        "[[1 2] 3]",
    );
}

#[test]
fn test_dicts() {
    test_obj!(|obj| obj.dict(), "<<\n>>");
    test_obj!(
        |obj| obj.dict().pair(Name(b"Quality"), Name(b"Good")),
        "<<\n/Quality /Good\n>>",
    );
}
