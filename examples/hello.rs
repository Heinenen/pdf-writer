use pdf_writer::{
    ActionType, AnnotationType, BorderType, Content, Name, PdfWriter, Rect, Ref, Str,
    TextStr,
};

fn main() -> std::io::Result<()> {
    // Start writing with PDF version 1.7 header. The version is not
    // semantically important to the writer, but must be present in the output
    // document.
    let mut writer = PdfWriter::new(1, 7);

    // Make the output more readable by indenting things with 2 spaces.
    writer.set_indent(2);

    // Define some indirect reference ids we'll use.
    let catalog_id = Ref::new(1);
    let page_tree_id = Ref::new(2);
    let page_id = Ref::new(3);
    let font_id = Ref::new(4);
    let text_id = Ref::new(5);
    let font_name = Name(b"F1");

    // Write the document catalog with a reference to the page tree.
    writer.catalog(catalog_id).pages(page_tree_id);

    // Write the page tree with a single child page.
    writer.pages(page_tree_id).kids(vec![page_id]);

    // Write a page.
    let mut page = writer.page(page_id);

    // Set the size to A4 (measured in points) using `media_box` and set the
    // text object we'll write later as the page's contents.
    page.media_box(Rect::new(0.0, 0.0, 595.0, 842.0));
    page.parent(page_tree_id);
    page.contents(text_id);

    // We also create the annotations list here that allows us to have things
    // like links or comments on the page.
    let mut annotations = page.annotations();

    // Write a new annotation.
    let mut annotation = annotations.push();

    // Write the type, area, alt-text, and color for this annotation.
    annotation.subtype(AnnotationType::Link);
    annotation.rect(Rect::new(215.0, 730.0, 251.0, 748.0));
    annotation.contents(TextStr("Link to the Rust project web page"));
    annotation.color_rgb(0.0, 0.0, 1.0);

    // Write an action for the annotation, telling it where to link to. Actions
    // can be associated with annotations, outline objects, and more and allow
    // creating interactive PDFs (open links, play sounds...).
    annotation
        .action()
        .action_type(ActionType::Uri)
        .uri(Str(b"https://www.rust-lang.org/"));

    // Set border and style for the link annotation.
    annotation.border_style().width(3.0).style(BorderType::Underline);

    // We have to drop all the writers that page depends on in order here
    // because otherwise it would be mutably borrowed until the end of the
    // block.
    drop(annotation);
    drop(annotations);

    // We also need to specify which resources the page needs, which in our case
    // is only a font that we name "F1" (the specific name doesn't matter).
    page.resources().fonts().pair(font_name, font_id);

    drop(page);

    // Specify the font we want to use. Because Helvetica is one of the 14 base
    // fonts shipped with every PDF reader, we don't have to embed any font
    // data.
    writer.type1_font(font_id).base_font(Name(b"Helvetica"));

    // Write a line of text, with the font specified in the resource list
    // before, at a font size of 14.0, starting at coordinates (108.0, 734.0)
    // measured from the bottom left of the page.
    //
    // Because we haven't specified any encoding when writing the Type 1 font,
    // the standard encoding is used which happens to work with most ASCII
    // characters.
    let mut content = Content::new();
    content
        .text()
        .font(font_name, 14.0)
        .next_line(108.0, 734.0)
        .show(Str(b"Hello World from Rust!"));

    writer.stream(text_id, &content.finish());

    // Finish writing (this automatically creates the cross-reference table and
    // file trailer) and retrieve the resulting byte buffer.
    let buf: Vec<u8> = writer.finish(catalog_id);

    // Write the thing to a file.
    std::fs::write("target/hello.pdf", buf)
}
