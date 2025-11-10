use html_sanitizer::parse_elements;
use ratatui::style::Style;

use crate::comments::spans;

#[test]
fn test_mark_up() {
    let test = "&quot;Samsung Family Hub™ for 2025 Update Elevates the Smart Home Ecosystem\nThe software update includes a more unified user experience across connected devices, enhancements to AI Vision Inside™, expanded Knox Security and more&quot;<p>In plain English now, Samgung will put advertising on your face but mostly important is that they will know and sell the data about what you buy.<p>They call it Smart Home. Yeah, &quot;smart&quot;. They are the smart ones, not those who buy this **.";

    let elements = parse_elements(test);

    let spans = spans(elements, Style::default(), None);
    assert_eq!(spans.len(), 7);
}

#[test]
fn test_mark_up_adjacent_escaped() {
    let test = r#"From the author&#x27;s Reddit post &lt;<a href="https:&#x2F;&#x2F;www.reddit.com&#x2F;r&#x2F;vintagecomputing&#x2F;comments&#x2F;1ot83o4&#x2F;you_can_boot_68k_hpux_and_parisc_hpux_from_the&#x2F;" rel="nofollow">https:&#x2F;&#x2F;www.reddit.com&#x2F;r&#x2F;vintagecomputing&#x2F;comments&#x2F;1ot83o4&#x2F;y...</a>&gt;:<p>&gt;I’ve got my HP 9000 Model 340 booting over the network from an HP 9000 Model 705 in Cluster Server mode and I’ve learned some very unsettling things about HP-UX and its filesystem.<p>&gt;Boot-up video at the end of the blog, where I play a bit of the original version of Columns."#;

    let elements = parse_elements(test);

    dbg!(&elements);

    let spans = spans(elements, Style::default(), None);

    dbg!(&spans);
}
