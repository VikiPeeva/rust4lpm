pub mod petri_net;
pub mod dfg;

pub trait DotRenderable {
    fn to_dot(&self) -> String;

    fn render(&self) {
        use layout::gv;
        use layout::backends::svg::SVGWriter;

        let dot = self.to_dot();
        let mut parser = gv::DotParser::new(&dot);
        let graph = parser.process().expect("failed to parse DOT");

        let mut builder = gv::GraphBuilder::new();
        builder.visit_graph(&graph);
        let mut vg = builder.get();

        let mut svg = SVGWriter::new();
        vg.do_it(false, false, false, &mut svg);

        let tmp = std::env::temp_dir().join(format!("{}.svg", uuid::Uuid::new_v4()));
        layout::core::utils::save_to_file(tmp.to_str().
            unwrap(), &svg.finalize())
            .expect("failed to write SVG");

        open::with(tmp, "firefox").unwrap();
    }
}