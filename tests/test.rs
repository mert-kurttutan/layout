use layout::core::geometry::Point;

#[cfg(test)]
mod tests {

    use layout::backends::svg::SVGWriter;
    use layout::core::geometry::weighted_median;
    use layout::gv::parser::ast::{DotString, Stmt};
    use layout::gv::record::parse_record_string;
    use layout::gv::record::print_record;
    use layout::gv::DotParser;
    use layout::gv::GraphBuilder;
    use layout::gv::Lexer;
    use layout::gv::Token;
    use layout::std_shapes::shapes::RecordDef;

    fn parse_dot(program: &str) -> layout::gv::parser::ast::Graph {
        let mut parser = DotParser::new(program);
        match parser.process() {
            Ok(graph) => graph,
            Err(err) => {
                parser.print_error();
                panic!("Failed to parse program: {}", err);
            }
        }
    }

    fn render_dot(program: &str) -> String {
        let graph = parse_dot(program);
        let mut builder = GraphBuilder::new();
        builder.visit_graph(&graph);
        let mut visual_graph = builder.get();
        let mut svg = SVGWriter::new();
        visual_graph.do_it(false, false, false, &mut svg);
        svg.finalize()
    }

    fn text_y(svg: &str, label: &str) -> f64 {
        let label_idx = svg
            .find(&format!(">{}</tspan>", label))
            .unwrap_or_else(|| panic!("expected label {}", label));
        let y_idx = svg[..label_idx]
            .rfind(" y=\"")
            .unwrap_or_else(|| panic!("expected y coordinate for {}", label));
        let y_start = y_idx + 4;
        let y_end = svg[y_start..]
            .find('"')
            .map(|idx| y_start + idx)
            .expect("expected y coordinate terminator");
        svg[y_start..y_end]
            .parse()
            .unwrap_or_else(|_| panic!("expected numeric y for {}", label))
    }

    fn is_identifier(t: Token, target: &str) -> bool {
        match t {
            Token::Identifier(name) => target == name,
            _ => false,
        }
    }
    fn get_sample_program2() -> String {
        r##"/* ancestor graph from Caroline Bouvier Kennedy */
        graph G {
            I5 [shape=ellipse,color=red,style=bold,label="Caroline Bouvier Kennedy\nb. 27.11.1957 New York",image="images/165px-Caroline_Kennedy.jpg",labelloc=b];
            I1 [shape=box,color=blue,style=bold,label="John Fitzgerald Kennedy\nb. 29.5.1917 Brookline\nd. 22.11.1963 Dallas",image="images/kennedyface.jpg",labelloc=b];
            I6 [shape=box,color=blue,style=bold,label="John Fitzgerald Kennedy\nb. 25.11.1960 Washington\nd. 16.7.1999 over the Atlantic Ocean, near Aquinnah, MA, USA",image="images/180px-JFKJr2.jpg",labelloc=b];
            I7 [shape=box,color=blue,style=bold,label="Patrick Bouvier Kennedy\nb. 7.8.1963\nd. 9.8.1963"];
            I2 [shape=ellipse,color=red,style=bold,label="Jaqueline Lee Bouvier\nb. 28.7.1929 Southampton\nd. 19.5.1994 New York City",image="images/jacqueline-kennedy-onassis.jpg",labelloc=b];
            I8 [shape=box,color=blue,style=bold,label="Joseph Patrick Kennedy\nb. 6.9.1888 East Boston\nd. 16.11.1969 Hyannis Port",image="images/1025901671.jpg",labelloc=b];
            I10 [shape=box,color=blue,style=bold,label="Joseph Patrick Kennedy Jr\nb. 1915\nd. 1944"];
            I11 [shape=ellipse,color=red,style=bold,label="Rosemary Kennedy\nb. 13.9.1918\nd. 7.1.2005",image="images/rosemary.jpg",labelloc=b];
            I12 [shape=ellipse,color=red,style=bold,label="Kathleen Kennedy\nb. 1920\nd. 1948"];
            I13 [shape=ellipse,color=red,style=bold,label="Eunice Mary Kennedy\nb. 10.7.1921 Brookline"];
            I9 [shape=ellipse,color=red,style=bold,label="Rose Elizabeth Fitzgerald\nb. 22.7.1890 Boston\nd. 22.1.1995 Hyannis Port",image="images/Rose_kennedy.JPG",labelloc=b];
            I15 [shape=box,color=blue,style=bold,label="Aristotle Onassis"];
            I3 [shape=box,color=blue,style=bold,label="John Vernou Bouvier III\nb. 1891\nd. 1957",image="images/BE037819.jpg",labelloc=b];
            I4 [shape=ellipse,color=red,style=bold,label="Janet Norton Lee\nb. 2.10.1877\nd. 3.1.1968",image="images/n48862003257_1275276_1366.jpg",labelloc=b];
             I1 -- I5  [style=bold,color=blue]; 
             I1 -- I6  [style=bold,color=orange]; 
             I2 -- I6  [style=bold,color=orange]; 
             I1 -- I7  [style=bold,color=orange]; 
             I2 -- I7  [style=bold,color=orange]; 
             I1 -- I2  [style=bold,color=violet]; 
             I8 -- I1  [style=bold,color=blue]; 
             I8 -- I10  [style=bold,color=orange]; 
             I9 -- I10  [style=bold,color=orange]; 
             I8 -- I11  [style=bold,color=orange]; 
             I9 -- I11  [style=bold,color=orange]; 
             I8 -- I12  [style=bold,color=orange]; 
             I9 -- I12  [style=bold,color=orange]; 
             I8 -- I13  [style=bold,color=orange]; 
             I9 -- I13  [style=bold,color=orange]; 
             I8 -- I9  [style=bold,color=violet]; 
             I9 -- I1  [style=bold,color=red]; 
             I2 -- I5  [style=bold,color=red]; 
             I2 -- I15  [style=bold,color=violet]; 
             I3 -- I2  [style=bold,color=blue]; 
             I3 -- I4  [style=bold,color=violet]; 
             I4 -- I2  [style=bold,color=red]; 
            }
        "##
        .to_string()
    }

    #[test]
    fn simple() {
        let mut lexer = Lexer::from_string("a -> b");
        let t0 = lexer.next_token();
        let t1 = lexer.next_token();
        let t2 = lexer.next_token();
        println!("{:?}", t0);
        println!("{:?}", t1);
        println!("{:?}", t2);
        assert!(is_identifier(t0, "a"));
        assert!(matches!(t1, Token::ArrowRight));
        assert!(is_identifier(t2, "b"));
    }
    #[test]
    fn simple_number() {
        let mut lexer = Lexer::from_string("-12345");
        let t0 = lexer.next_token();
        let t1 = lexer.next_token();
        println!("{:?}", t0);
        println!("{:?}", t1);
        assert!(is_identifier(t0, "-12345"));
        assert!(matches!(t1, Token::EOF));
    }
    #[test]
    fn simple_float_number() {
        let mut lexer = Lexer::from_string("1.12");
        let t0 = lexer.next_token();
        let t1 = lexer.next_token();
        println!("{:?}", t0);
        println!("{:?}", t1);
        assert!(is_identifier(t0, "1.12"));
        assert!(matches!(t1, Token::EOF));
    }

    #[test]
    fn simple_program() {
        let mut lexer = Lexer::from_string("digraph { a -> b; } ");
        assert!(matches!(lexer.next_token(), Token::DigraphKW));
        assert!(matches!(lexer.next_token(), Token::OpenBrace));
        assert!(matches!(lexer.next_token(), Token::Identifier(_)));
        assert!(matches!(lexer.next_token(), Token::ArrowRight));
        assert!(matches!(lexer.next_token(), Token::Identifier(_)));
        assert!(matches!(lexer.next_token(), Token::Semicolon));
        assert!(matches!(lexer.next_token(), Token::CloseBrace));
        assert!(matches!(lexer.next_token(), Token::EOF));
    }

    #[test]
    fn catch_unterminated_str() {
        let mut lexer = Lexer::from_string("digraph { a -> b; \" } ");
        assert!(matches!(lexer.next_token(), Token::DigraphKW));
        assert!(matches!(lexer.next_token(), Token::OpenBrace));
        assert!(matches!(lexer.next_token(), Token::Identifier(_)));
        assert!(matches!(lexer.next_token(), Token::ArrowRight));
        assert!(matches!(lexer.next_token(), Token::Identifier(_)));
        assert!(matches!(lexer.next_token(), Token::Semicolon));
        assert!(matches!(lexer.next_token(), Token::Error(_)));
    }

    #[test]
    fn lex_program() {
        let program = get_sample_program2();
        let mut lexer = Lexer::from_string(&program[..]);
        let mut tok = lexer.next_token();
        let mut counter = 1;
        while !matches!(tok, Token::EOF) {
            println!("{:?}", tok);
            if let Token::Error(_) = tok {
                lexer.print_error();
                panic!();
            }

            tok = lexer.next_token();
            counter += 1;
        }
        assert_eq!(counter, 629);
    }

    #[test]
    fn parse_program0() {
        let mut parser = DotParser::new("graph { a -> b; b -> c;}");
        if let Result::Err(err) = parser.process() {
            parser.print_error();
            println!("Error: {}", err);
            panic!();
        }
    }

    #[test]
    fn parse_program1() {
        let mut parser = DotParser::new("graph { a -> b -> c; }");
        if let Result::Err(err) = parser.process() {
            parser.print_error();
            println!("Error: {}", err);
            panic!();
        }
    }

    #[test]
    fn parse_program2() {
        let program = get_sample_program2();
        let mut parser = DotParser::new(&program[..]);
        if let Result::Err(err) = parser.process() {
            parser.print_error();
            println!("Error: {}", err);
            panic!();
        }
    }

    #[test]
    fn parse_program_fail() {
        let mut parser = DotParser::new("graph { } s");
        if parser.process().is_err() {
            return;
        }
        panic!();
    }

    #[test]
    fn parse_html_node_label_with_entities() {
        let graph = parse_dot(r#"digraph { a [label=<A &amp; B &lt; C>]; }"#);
        let node = graph
            .list
            .list
            .iter()
            .find_map(|stmt| match stmt {
                Stmt::Node(node) => Some(node),
                _ => None,
            })
            .expect("expected node statement");
        let label = node
            .list
            .iter()
            .find(|(key, _)| key == "label")
            .expect("expected node label");

        match &label.1 {
            DotString::HtmlString(value) => {
                assert_eq!(value, "A &amp; B &lt; C");
            }
            other => panic!("expected HTML label, got {:?}", other),
        }
    }

    #[test]
    fn parse_html_edge_labels() {
        let graph = parse_dot(
            r#"digraph {
                a -> b [
                    label=<edge <B>label</B>>,
                    headlabel=<head>,
                    taillabel=<tail>
                ];
            }"#,
        );
        let edge = graph
            .list
            .list
            .iter()
            .find_map(|stmt| match stmt {
                Stmt::Edge(edge) => Some(edge),
                _ => None,
            })
            .expect("expected edge statement");

        for attr_name in ["label", "headlabel", "taillabel"] {
            let attr = edge
                .list
                .iter()
                .find(|(key, _)| key == attr_name)
                .unwrap_or_else(|| panic!("expected {}", attr_name));
            assert!(
                matches!(attr.1, DotString::HtmlString(_)),
                "expected {} to be parsed as HTML",
                attr_name
            );
        }
    }

    #[test]
    fn parse_html_table_with_ports_and_rules() {
        parse_dot(
            r#"digraph {
                a [shape=plain label=<
                    <TABLE BORDER="0" CELLBORDER="1" CELLSPACING="0">
                        <TR><TD PORT="left">left</TD><VR/><TD PORT="right">right</TD></TR>
                        <HR/>
                        <TR><TD COLSPAN="2">bottom</TD></TR>
                    </TABLE>
                >];
                b [label="target"];
                a:right -> b;
            }"#,
        );
    }

    #[test]
    fn render_html_plain_shape_table() {
        let svg = render_dot(
            r#"digraph {
                a [shape=plain label=<
                    <TABLE BORDER="0" CELLBORDER="1" CELLSPACING="0">
                        <TR><TD>left</TD><TD>right</TD></TR>
                    </TABLE>
                >];
            }"#,
        );

        assert!(svg.contains("<svg"));
        assert!(svg.contains("<rect"));
        assert!(svg.contains(">left</tspan>"));
        assert!(svg.contains(">right</tspan>"));
        assert!(!svg.contains("<ellipse"));
    }

    #[test]
    fn render_html_image_label() {
        let svg = render_dot(
            r#"digraph {
                a [shape=plain label=<
                    <TABLE BORDER="1" CELLBORDER="1">
                        <TR><TD><IMG SRC="docs/sample.png" SCALE="TRUE"/></TD></TR>
                        <TR><TD>caption</TD></TR>
                    </TABLE>
                >];
            }"#,
        );

        assert!(svg.contains("<image"));
        assert!(svg.contains("href=\"docs/sample.png\""));
        assert!(svg.contains(">caption</tspan>"));
    }

    #[test]
    fn render_html_edge_label() {
        let svg = render_dot(
            r#"digraph {
                a [label="source"];
                b [label="target"];
                a -> b [label=<
                    <TABLE BORDER="0" CELLBORDER="1" CELLSPACING="0">
                        <TR><TD><B>HTML</B></TD><TD>edge label</TD></TR>
                    </TABLE>
                >];
            }"#,
        );

        assert!(svg.contains(">source</tspan>"));
        assert!(svg.contains(">target</tspan>"));
        assert!(svg.contains(">HTML</tspan>"));
        assert!(svg.contains(">edge label</tspan>"));
    }

    #[test]
    fn render_filled_cluster_uses_cluster_color() {
        let svg = render_dot(
            r#"digraph G {
                subgraph cluster_0 {
                    style=filled;
                    color=lightgrey;
                    node [style=filled,color=white];
                    a0 -> a1 -> a2 -> a3;
                    label = "process #1";
                }

                start -> a0;
                start -> b0;
                a3 -> a0;
                a3 -> end;
                b3 -> end;
            }"#,
        );

        assert!(svg.contains("fill=\"#d3d3d3ff\""));
        assert!(!svg.contains("fill=\"#000000ff\" \n            stroke-width=\"1\" stroke=\"#d3d3d3ff\""));
    }

    #[test]
    fn render_graph_and_cluster_labels() {
        let svg = render_dot(
            r#"digraph G {
                label="Graph Label";
                fontsize=22;

                subgraph cluster_0 {
                    label="Cluster Label";
                    style=filled;
                    fillcolor=lightgrey;
                    a -> b;
                }
            }"#,
        );

        assert!(svg.contains(">Graph Label</tspan>"));
        assert!(svg.contains(">Cluster Label</tspan>"));
    }

    #[test]
    fn render_bottom_graph_and_cluster_labels() {
        let svg = render_dot(
            r#"digraph G {
                label="Bottom Graph Label";
                labelloc=b;

                subgraph cluster_0 {
                    label="Bottom Cluster Label";
                    labelloc=b;
                    a -> b;
                }
            }"#,
        );

        assert!(text_y(&svg, "Bottom Graph Label") > text_y(&svg, "b"));
        assert!(text_y(&svg, "Bottom Cluster Label") > text_y(&svg, "b"));
    }

    #[test]
    fn render_html_entities() {
        let svg = render_dot(
            r#"digraph {
                a [shape=plain label=<
                    <TABLE BORDER="0" CELLBORDER="1" CELLSPACING="0">
                        <TR><TD>A &amp; B</TD></TR>
                        <TR><TD>1 &lt; 2 &gt; 0</TD></TR>
                        <TR><TD>&quot;quoted&quot; &apos;text&apos;</TD></TR>
                        <TR><TD>&lt;&gt;&amp;&quot;&apos;</TD></TR>
                        <TR><TD>prefix&amp;middle&lt;suffix&gt;</TD></TR>
                    </TABLE>
                >];
            }"#,
        );

        assert!(svg.contains(">A &amp; B</tspan>"));
        assert!(svg.contains(">1 &lt; 2 &gt; 0</tspan>"));
        assert!(svg.contains(">&quot;quoted&quot; &apos;text&apos;</tspan>"));
        assert!(svg.contains(">&lt;&gt;&amp;&quot;&apos;</tspan>"));
        assert!(svg.contains(">prefix&amp;middle&lt;suffix&gt;</tspan>"));
        assert!(!svg.contains("&amp;amp;"));
        assert!(!svg.contains("&amp;lt;"));
        assert!(!svg.contains("&amp;gt;"));
        assert!(!svg.contains("&amp;quot;"));
        assert!(!svg.contains("&amp;apos;"));
    }

    #[test]
    #[should_panic]
    fn render_html_unknown_entity_panics() {
        render_dot(r#"digraph { a [label=<unknown &madeup; entity>]; }"#);
    }

    #[test]
    #[should_panic]
    fn render_html_missing_entity_semicolon_panics() {
        render_dot(r#"digraph { a [label=<missing &amp semicolon>]; }"#);
    }

    #[test]
    #[should_panic]
    fn render_html_numeric_entity_panics() {
        render_dot(r#"digraph { a [label=<numeric &#65; entity>]; }"#);
    }

    #[test]
    fn render_quoted_label_non_ascii_entities() {
        let svg = render_dot(
            r#"digraph {
                a [label="forall: &#8704;"];
                b [label="hex forall: &#x2200;"];
                a -> b [
                    label="edge: &#8704;",
                    headlabel="head: &#8704;",
                    taillabel="tail: &#x2200;"
                ];
            }"#,
        );

        assert!(svg.contains("forall: \u{2200}</tspan>"));
        assert!(svg.contains("hex forall: \u{2200}</tspan>"));
        assert!(svg.contains("edge: \u{2200}</tspan>"));
        assert!(svg.contains("head: \u{2200}</tspan>"));
        assert!(svg.contains("tail: \u{2200}</tspan>"));
        assert!(!svg.contains("&amp;#8704;"));
        assert!(!svg.contains("&amp;#x2200;"));
    }

    #[test]
    fn render_head_and_tail_labels() {
        let svg = render_dot(
            r#"digraph {
                a [label="source"];
                b [label="target"];
                a -> b [
                    label="middle",
                    taillabel="near source",
                    headlabel="near target"
                ];
            }"#,
        );

        assert!(svg.contains(">middle</tspan>"));
        assert!(svg.contains(">near source</tspan>"));
        assert!(svg.contains(">near target</tspan>"));
    }

    #[test]
    fn render_html_head_and_tail_labels() {
        let svg = render_dot(
            r#"digraph {
                a [label="source"];
                b [label="target"];
                a -> b [
                    taillabel=<near <I>source</I>>,
                    headlabel=<near <B>target</B>>
                ];
            }"#,
        );

        assert!(svg.contains(">near </tspan>"));
        assert!(svg.contains(">source</tspan>"));
        assert!(svg.contains(">target</tspan>"));
        assert!(svg.contains("font-style=\"italic\""));
        assert!(svg.contains("font-weight=\"bold\""));
    }

    #[test]
    fn parse_record0() {
        let desc = "hello&#92;nworld |{ b |{c|<here> d|e}| f}| g | h";
        let res = parse_record_string(desc);
        print_record(&res, 0);
    }
    #[test]
    fn parse_record1() {
        let desc = "{InputLayer\n|{input:|output:}|{{[(?, ?)]}|{[(?, ?)]}}}";
        let res = parse_record_string(desc);
        print_record(&res, 0);
    }

    #[test]
    fn parse_record2() {
        let desc = "department: Dense\n|{input:|output:}|{{(?, 172)}|{(?, 4)}}";
        let res = parse_record_string(desc);
        print_record(&res, 0);
    }

    #[test]
    fn parse_record_port0() {
        let desc = "<f0> foo";
        let res = parse_record_string(desc);
        print_record(&res, 0);
        if let RecordDef::Array(arr) = res {
            assert_eq!(arr.len(), 1, "expecting one element");
            if let RecordDef::Text(label, port) = &arr[0] {
                assert_eq!(label, "foo");
                if let Option::Some(port) = port {
                    assert_eq!(port, "f0");
                } else {
                    panic!();
                }
            } else {
                panic!();
            }
        } else {
            panic!();
        }
    }

    #[test]
    fn test_median() {
        let k = weighted_median(&[1.]);
        assert_eq!(k, 1.);
        let k = weighted_median(&[2., 1.]);
        assert_eq!(k, 1.5);
        let k = weighted_median(&[99., 2., 1.]);
        assert_eq!(k, 2.);
        let k = weighted_median(&[90., 23., 0., 1., 3.]);
        assert_eq!(k, 3.);
        let k = weighted_median(&[0., 99., 30., 40.]);
        assert_eq!(k, 35.);
    }

    #[test]
    fn test_median_range() {
        for i in 2..10 {
            let data: Vec<f64> = (1..i).map(|x: usize| x as f64).collect();
            println!("{:?}", data);
            let _ = weighted_median(&data);
        }
    }
}

#[test]
fn test_rotate() {
    fn almost(a: f64, b: f64) {
        let abs_difference = (b - a).abs();
        assert!(abs_difference < 1e-10);
    }
    // 180'
    let p = Point::new(1.0, 0.0);
    let r = p.rotate(180_f64.to_radians());
    almost(r.x, -1.);
    almost(r.y, 0.);

    //90'
    let p = Point::new(1.0, 0.0);
    let r = p.rotate(90_f64.to_radians());
    almost(r.x, 0.);
    almost(r.y, 1.);

    //45'
    let p = Point::new(1.0, 0.0);
    let r = p.rotate(45_f64.to_radians());
    almost(r.x, 1. / 2_f64.sqrt());
    almost(r.y, 1. / 2_f64.sqrt());

    //Rotate around a point.
    let p = Point::new(101.0, 100.0);
    let c = Point::new(100., 100.);
    let r = p.rotate_around(c, 45_f64.to_radians());
    almost(r.x, 100. + 1. / 2_f64.sqrt());
    almost(r.y, 100. + 1. / 2_f64.sqrt());
}
